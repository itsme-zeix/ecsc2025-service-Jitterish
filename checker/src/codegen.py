import json
import random
import string
from abc import abstractmethod, ABC
from contextlib import contextmanager
from dataclasses import dataclass
from typing import Any, NamedTuple, Generator

Expr = NamedTuple('Expr', [('expr', str), ('value', Any)])


INT_MASK = 0xffffffffffffff


@dataclass
class Context:
    vars: dict[str, Expr]


class ExprGen(ABC):
    def __init__(self, gen: 'CodeGenerator') -> None:
        self.gen = gen

    def applicable(self, ctx: Context) -> bool:
        return True

    @abstractmethod
    def generate(self, ctx: Context) -> Expr:
        pass


class VarUse(ExprGen):
    TYPE = str

    def applicable(self, ctx: Context) -> bool:
        return any(isinstance(_, self.TYPE) for _ in ctx.vars.values())

    def generate(self, ctx: Context) -> Expr:
        v = [(k, v) for k, v in ctx.vars.items() if isinstance(v, self.TYPE)]
        name, value = random.choice(v)
        return Expr(name, value)


class IntConst(ExprGen):
    def generate(self, ctx: Context) -> Expr:
        i = random.randint(0, 1000)
        return Expr(str(i), i)


class IntOp(ExprGen):
    def generate(self, ctx: Context) -> Expr:
        expr1 = self.gen.random_int_expr(ctx)
        expr2 = self.gen.random_int_expr(ctx)
        options = [
            ('+', (expr1.value + expr2.value) & INT_MASK),
            ('-', (expr1.value - expr2.value) & INT_MASK),
            ('*', (expr1.value * expr2.value) & INT_MASK),
        ]
        if expr2.value > 0:
            options.append(('/', (expr1.value // expr2.value) & INT_MASK))
        op, result = random.choice(options)
        return Expr(f'({expr1.expr} {op} {expr2.expr})', result)


class IntUnaryOp(ExprGen):
    def generate(self, ctx: Context) -> Expr:
        expr = self.gen.random_int_expr(ctx)
        return Expr(f'-{expr.expr}', (-expr.value) & INT_MASK)


class IntVarUse(VarUse):
    TYPE = int


class StringConst(ExprGen):
    STRING_CHARS = string.ascii_letters + string.digits + "._-:"

    def generate(self, ctx: Context) -> Expr:
        s = ''.join(random.choice(self.STRING_CHARS) for _ in range(random.randint(0, 12)))
        return Expr(json.dumps(s), s)


class StringVarUse(VarUse):
    TYPE = str


class StringAdd(ExprGen):
    def generate(self, ctx: Context) -> Expr:
        expr1 = self.gen.random_str_expr(ctx)
        expr2 = self.gen.random_str_expr(ctx) if random.randint(0, 2) else self.gen.random_int_expr(ctx)
        return Expr(f'({expr1.expr} + {expr2.expr})', expr1.value + str(expr2.value))


class CodeGenerator:
    MAX_EXPR_DEPTH = 5

    def __init__(self, collections: dict[str, list[dict[str, Any]]]) -> None:
        self.collections = collections
        self.depth = 0
        self.gens_str: list[ExprGen] = [StringConst(self), StringVarUse(self), StringAdd(self)]
        self.gens_int: list[ExprGen] = [IntConst(self), IntVarUse(self), IntOp(self), IntUnaryOp(self)]

    @contextmanager
    def _depth(self) -> Generator[None, Any, None]:
        self.depth += 1
        yield
        self.depth -= 1

    def random_int_expr(self, ctx: Context) -> Expr:
        if self.depth > self.MAX_EXPR_DEPTH:
            return self.gens_int[0].generate(ctx)
        with self._depth():
            gens = [_ for _ in self.gens_int if _.applicable(ctx)]
            return random.choice(gens).generate(ctx)

    def random_str_expr(self, ctx: Context) -> Expr:
        if self.depth > self.MAX_EXPR_DEPTH:
            return self.gens_str[0].generate(ctx)
        with self._depth():
            gens = [_ for _ in self.gens_str if _.applicable(ctx)]
            return random.choice(gens).generate(ctx)
