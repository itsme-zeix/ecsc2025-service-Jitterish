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


class ExpressionTestCases(TestCase):
    ITERATIONS = 20

    def setUp(self):
        self.compiler = Compiler()
        self.tester = CompilerTester(self.compiler)
        self.gen = CodeGenerator({})

    def test_expressions_int(self) -> None:
        for _ in range(self.ITERATIONS):
            ctx = Context({})
            code = self.gen.int_expr.generate(ctx)
            print(code)
            self.tester.test_expr(code)

    def test_expressions_str(self) -> None:
        for _ in range(self.ITERATIONS):
            ctx = Context({})
            code = self.gen.str_expr.generate(ctx)
            print(code)
            self.tester.test_expr(code)

    def test_expressions_bool(self) -> None:
        for _ in range(self.ITERATIONS):
            ctx = Context({})
            code = self.gen.bool_expr.generate(ctx)
            print(code)
            self.tester.test_expr(code)

    def test_queries(self) -> None:
        for _ in range(self.ITERATIONS):
            coll = self.gen.collection('FLAG{abc}')
            qry = self.gen.flag_query(coll)
            flg = self.tester.test_func(qry, {coll.name: coll.to_ndjson()})
            # print(flg)


if __name__ == '__main__':
    unittest.main()
