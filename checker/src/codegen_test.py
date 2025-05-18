import json
import subprocess
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory
from typing import Any
from unittest import TestCase

from .codegen import Expr, CodeGenerator, Context, Function, expr_value


def find_compiler() -> Path:
    for p in ('cmake-build-debug', 'build', ''):
        candidate = JIT_ROOT / p / 'jit'
        if candidate.exists():
            print(f'Compiler: {candidate}')
            return candidate
    raise Exception('Compiler not found, please build it first!')


JIT_ROOT: Path = Path(__file__).absolute().parent.parent.parent / 'service' / 'jit'
COMPILER: Path = find_compiler()


def read_json_lines(s: str) -> list[Any]:
    return [json.loads(l) for l in s.strip().split('\n') if l]


class Compiler:
    def compile(self, code: str, output: Path) -> None:
        p = subprocess.Popen([str(COMPILER), '/dev/stdin', str(output)], stdin=subprocess.PIPE)
        p.communicate(input=code.encode(), timeout=5)
        p.wait()
        if p.returncode != 0:
            raise Exception(f'Compilation failed with code {p.returncode}')

    def run_prog(self, code: str, function: str = 'jit_entrypoint', param: Any = None,
                 collections: dict[str, str | list] = {}) -> str:
        with TemporaryDirectory(dir='/dev/shm') as tmpdir:
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

    def run_prog(self, *args, **kwargs) -> tuple[list[Any], Any]:
        lines = self.compiler.run_prog(*args, **kwargs)
        raw_result = read_json_lines(lines)
        return raw_result[:-1], raw_result[-1]

    def test_expr(self, expr: Expr) -> None:
        output, result = self.run_prog(f'func jit_entrypoint() {{ return {expr.expr}; }}')
        if result != expr.value:
            print(expr.expr)
            raise Exception(f'Expected {expr.value}, got {result}')
        assert output == []

    def test_func(self, func: Function, collections: dict[str, str]) -> Any:
        try:
            param = None if len(func.params) == 0 else func.params[0].value
            output, result = self.run_prog(func.code, func.name, param=param, collections=collections)
            if result != expr_value(func.value):
                raise Exception(f'Expected {func.value}, got {result}')
            expected_output = expr_value(func.output)
            if output != expected_output:
                raise Exception(f'Expected output {expected_output}, got {output}')
            return result
        except:
            print(func.code)
            raise


class CodegenTestCases(TestCase):
    ITERATIONS = 25
    CHUNKS = 1000

    def setUp(self):
        self.compiler = Compiler()
        self.tester = CompilerTester(self.compiler)
        self.gen = CodeGenerator({})

    def test_expressions_int(self) -> None:
        for _ in range(self.ITERATIONS):
            ctx = Context({})
            code = self.gen.int_expr.generate(ctx)
            self.tester.test_expr(code)

    def test_expressions_str(self) -> None:
        for _ in range(self.ITERATIONS):
            ctx = Context({})
            code = self.gen.str_expr.generate(ctx)
            self.tester.test_expr(code)

    def test_expressions_bool(self) -> None:
        for _ in range(self.ITERATIONS):
            ctx = Context({})
            code = self.gen.bool_expr.generate(ctx)
            self.tester.test_expr(code)

    def _test_expr_chunks(self, gen) -> None:
        code = 'func jit_entrypoint() {\n'
        exprs = []
        for _ in range(self.CHUNKS):
            ctx = Context({})
            expr = gen.generate(ctx)
            exprs.append(expr)
            code += 'print(' + expr.expr + ');\n'
        code += 'return 0; }'
        output, result = self.tester.run_prog(code)
        if len(output) != len(exprs):
            raise Exception('Wrong number of output lines')
        for e, o in zip(exprs, output):
            if e.value != o:
                raise Exception(f'Expected {e.value}, got {o} for: {e.expr}')

    def test_bool_chunked(self) -> None:
        self._test_expr_chunks(self.gen.bool_expr)

    def test_str_chunked(self) -> None:
        self._test_expr_chunks(self.gen.str_expr)

    def test_int_chunked(self) -> None:
        self._test_expr_chunks(self.gen.int_expr)

    def test_functions(self) -> None:
        for _ in range(self.ITERATIONS):
            func = self.gen.function_any(max_params=1)
            print(func.code)
            self.tester.test_func(func, {})

    def test_queries(self) -> None:
        for _ in range(self.ITERATIONS):
            coll = self.gen.collection('FLAG{abc}')
            qry = self.gen.flag_query(coll)
            flg = self.tester.test_func(qry, {coll.name: coll.to_ndjson()})
            # print(flg)


if __name__ == '__main__':
    unittest.main()
