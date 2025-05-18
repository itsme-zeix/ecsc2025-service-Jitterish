import json
import subprocess
from pathlib import Path
from tempfile import TemporaryDirectory
from typing import Any

from codegen import Expr, CodeGenerator, Context, Function, expr_value


def find_compiler() -> Path:
    for p in ('cmake-build-debug', 'build', ''):
        candidate = JIT_ROOT / p / 'jit'
        if candidate.exists():
            print(f'Compiler: {candidate}')
            return candidate
    raise Exception('Compiler not found, please build it first!')


JIT_ROOT: Path = Path(__file__).absolute().parent.parent.parent / 'service' / 'jit'
COMPILER: Path = find_compiler()


class Compiler:
    def compile(self, code: str, output: Path) -> None:
        p = subprocess.Popen([str(COMPILER), '/dev/stdin', str(output)], stdin=subprocess.PIPE)
        p.communicate(input=code.encode(), timeout=5)
        p.wait()
        if p.returncode != 0:
            raise Exception(f'Compilation failed with code {p.returncode}')

    def run_prog(self, code: str, function: str = 'jit_entrypoint', param: Any = None,
                 collections: dict[str, str | list] = {}) -> str:
        with TemporaryDirectory() as tmpdir:
            tmpdir = Path(tmpdir)
            for n, c in collections.items():
                if isinstance(c, list):
                    c = '\n'.join(json.dumps(_) for _ in c) + '\n'
                (tmpdir / f'{n}.ndjson').write_text(c)

            self.compile(code, tmpdir / 'code.so')
            cmd: list[str] = [str(tmpdir / 'code.so'), function, json.dumps(param)]
            return subprocess.check_output(cmd, cwd=tmpdir).decode().strip()


class CompilerTester:
    def __init__(self, compiler: Compiler) -> None:
        self.compiler = compiler

    def test_expr(self, expr: Expr) -> None:
        result = self.compiler.run_prog(f'func jit_entrypoint() {{ return {expr.expr}; }}')
        result = json.loads(result)
        if result != expr.value:
            print(expr.expr)
            raise Exception(f'Expected {expr.value}, got {result}')

    def test_func(self, func: Function, collections: dict[str, str]) -> Any:
        try:
            result = self.compiler.run_prog(func.code, func.name, collections=collections)
            result = json.loads(result)
            if result != expr_value(func.value):
                raise Exception(f'Expected {func.value}, got {result}')
            return result
        except:
            print(func.code)
            raise


def test_expressions() -> None:
    compiler = Compiler()
    tester = CompilerTester(compiler)
    gen = CodeGenerator({})
    for _ in range(10):
        ctx = Context({})
        code = gen.random_str_expr(ctx)
        print(code)
        tester.test_expr(code)
    print('test_expressions completed.')


def test_queries() -> None:
    compiler = Compiler()
    tester = CompilerTester(compiler)
    gen = CodeGenerator({})
    for _ in range(10):
        coll = gen.collection('FLAG{abc}')
        qry = gen.flag_query(coll)
        flg = tester.test_func(qry, {coll.name: coll.to_ndjson()})
        # print(flg)
    print('test_queries completed.')


def main() -> None:
    test_queries()


if __name__ == '__main__':
    main()
