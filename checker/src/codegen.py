import json
import random
import string
from abc import abstractmethod, ABC
from contextlib import contextmanager
from dataclasses import dataclass
from typing import Any, NamedTuple, Generator, TypeAlias, Type

Value: TypeAlias = int | str | list | dict | None
Expr = NamedTuple('Expr', [('expr', str), ('value', Value)])
Function = NamedTuple('Function', [('name', str), ('code', str), ('params', list[Expr]),
                                   ('value', Value), ('output', list[Value])])
Stmt = NamedTuple('Stmt', [('code', list[str]), ('ctx', 'Context'), ('output', list[Value])])

INT_MASK = 0xffffffffffffff


def expr_value(v: Any) -> Value:
    if isinstance(v, Expr):
        return v.value
    if isinstance(v, dict):
        return {expr_value(k): expr_value(v) for k, v in v.items()}
    if isinstance(v, list):
        return [expr_value(_) for _ in v]
    return v


@dataclass
class Collection:
    name: str
    items: list[dict[Expr, Expr]]
    ident_key: Expr
    flag_key: Expr
    flag_item: dict[Expr, Expr]

    def to_ndjson(self) -> str:
        return '\n'.join(json.dumps(expr_value(item)) for item in self.items)


@dataclass
class Context:
    vars: dict[str, Value]


class ExprGen(ABC):
    def __init__(self, gen: 'CodeGenerator') -> None:
        self.gen = gen

    def applicable(self, ctx: Context) -> bool:
        return True

    @abstractmethod
    def generate(self, ctx: Context) -> Expr:
        pass


class ExprGenGroup(ExprGen):
    def __init__(self, gens: list[ExprGen]) -> None:
        super().__init__(gens[0].gen)
        self.gens = gens

    def generate(self, ctx: Context) -> Expr:
        if self.gen.current_depth > self.gen.MAX_EXPR_DEPTH:
            return self.gens[0].generate(ctx)
        with self.gen.depth():
            gens = [_ for _ in self.gens if _.applicable(ctx)]
            return random.choice(gens).generate(ctx)


class VarUse(ExprGen):
    TYPE = str

    def applicable(self, ctx: Context) -> bool:
        return any(type(_) == self.TYPE for _ in ctx.vars.values())

    def generate(self, ctx: Context) -> Expr:
        v = [(k, v) for k, v in ctx.vars.items() if type(v) == self.TYPE]
        name, value = random.choice(v)
        return Expr(name, value)


class IntConst(ExprGen):
    def generate(self, ctx: Context) -> Expr:
        i = random.randint(0, 1000)
        return Expr(str(i), i)


class IntOp(ExprGen):
    def generate(self, ctx: Context) -> Expr:
        expr1 = self.gen.int_expr.generate(ctx)
        expr2 = self.gen.int_expr.generate(ctx)
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
        expr = self.gen.int_expr.generate(ctx)
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
        expr1 = self.gen.str_expr.generate(ctx)
        expr2 = self.gen.str_expr.generate(ctx) if random.randint(0, 2) else self.gen.int_expr.generate(ctx)
        return Expr(f'({expr1.expr} + {expr2.expr})', expr1.value + str(expr2.value))


class BoolConst(ExprGen):
    def generate(self, ctx: Context) -> Expr:
        c = random.choice((True, False))
        return Expr(json.dumps(c), c)


class BoolUnary(ExprGen):
    def generate(self, ctx: Context) -> Expr:
        expr = self.gen.bool_expr.generate(ctx)
        return Expr(f'!{expr.expr}', not expr.value)


class BoolDefined(ExprGen):
    def generate(self, ctx: Context) -> Expr:
        options = [(_, True) for _ in ctx.vars]
        options += [(str(random.randint(0, 1000)), True), (f'({random.randint(0, 1000)} + true)', False)]
        e, r = random.choice(options)
        return Expr(f'defined {e}', r)


class BoolBinary(ExprGen):
    def generate(self, ctx: Context) -> Expr:
        op = random.randint(0, 3)
        expr1 = self.gen.bool_expr.generate(ctx)
        expr2 = self.gen.bool_expr.generate(ctx) if random.randint(0, 100) <= 80 or op >= 2 else self.gen.int_expr.generate(ctx)
        return [
            Expr(f'({expr1.expr} && {expr2.expr})', bool(expr1.value and expr2.value)),
            Expr(f'({expr1.expr} || {expr2.expr})', bool(expr1.value or expr2.value)),
            Expr(f'({expr1.expr} == {expr2.expr})', bool(expr1.value) == bool(expr2.value)),
            Expr(f'({expr1.expr} != {expr2.expr})', bool(expr1.value) != bool(expr2.value)),
        ][op]


class BoolComparison(ExprGen):
    def generate(self, ctx: Context) -> Expr:
        expr1 = self.gen.int_expr.generate(ctx)
        expr2 = self.gen.int_expr.generate(ctx)
        return random.choice([
            Expr(f'({expr1.expr} >= {expr2.expr})', expr1.value >= expr2.value),
            Expr(f'({expr1.expr} <= {expr2.expr})', expr1.value <= expr2.value),
            Expr(f'({expr1.expr} > {expr2.expr})', expr1.value > expr2.value),
            Expr(f'({expr1.expr} < {expr2.expr})', expr1.value < expr2.value),
        ])


class BoolVarUse(VarUse):
    TYPE = bool


class ArrayConstant(ExprGen):
    def generate(self, ctx: Context) -> Expr:
        a = []
        if random.randint(0, 100) <= 50:
            a.append(random.randint(0, 1000))
        if random.randint(0, 100) <= 50:
            a.append(self.gen.ident())
        return Expr(json.dumps(a), a)


class ObjectConstant(ExprGen):
    def generate(self, ctx: Context) -> Expr:
        a = {}
        if random.randint(0, 100) <= 50:
            a[self.gen.ident()] = random.randint(0, 1000)
        if random.randint(0, 100) <= 50:
            a[self.gen.ident()] = self.gen.ident()
        return Expr(json.dumps(a), a)


class PickArray(ExprGen):
    def __init__(self, gen: 'CodeGenerator', type: Type) -> None:
        super().__init__(gen)
        self.type = type

    def candidates(self, ctx: Context) -> list[tuple[str, int, Value]]:
        result = []
        for k, v in ctx.vars.items():
            if isinstance(v, list):
                for i, v2 in enumerate(v):
                    if type(v2) == self.type:
                        result.append((k, i, v2))
        return result
    def applicable(self, ctx: Context) -> bool:
        return len(self.candidates(ctx)) > 0

    def generate(self, ctx: Context) -> Expr:
        cands = self.candidates(ctx)
        var, i, v = random.choice(cands)
        return Expr(f'{var}[{i}]', v)


class PickObject(ExprGen):
    def __init__(self, gen: 'CodeGenerator', type: Type) -> None:
        super().__init__(gen)
        self.type = type

    def candidates(self, ctx: Context) -> list[tuple[str, str, Value]]:
        result = []
        for var, v in ctx.vars.items():
            if isinstance(v, dict):
                for k, v2 in v.items():
                    if type(v2) == self.type:
                        result.append((var, k, v2))
        return result
    def applicable(self, ctx: Context) -> bool:
        return len(self.candidates(ctx)) > 0

    def generate(self, ctx: Context) -> Expr:
        cands = self.candidates(ctx)
        var, k, v = random.choice(cands)
        return Expr(f'{var}[{json.dumps(k)}]', v)


class StmtGen(ABC):
    def __init__(self, gen: 'CodeGenerator') -> None:
        self.gen = gen

    def applicable(self, ctx: Context) -> bool:
        return True

    @abstractmethod
    def generate(self, ctx: Context) -> Stmt:
        pass


class StmtGenGroup(StmtGen):
    def __init__(self, gens: list[StmtGen]) -> None:
        super().__init__(gens[0].gen)
        self.gens = gens

    def generate(self, ctx: Context) -> Stmt:
        if self.gen.current_depth > self.gen.MAX_EXPR_DEPTH:
            return self.gens[0].generate(ctx)
        with self.gen.depth():
            gens = [_ for _ in self.gens if _.applicable(ctx)]
            return random.choice(gens).generate(ctx)


class VarDefine(StmtGen):
    def generate(self, ctx: Context) -> Stmt:
        name = self.gen.ident()
        expr = self.gen.any_expr.generate(ctx)
        ctx = Context(vars=ctx.vars | {name: expr})
        return Stmt([f'{name} = {expr.expr};'], ctx, [])


class VarAssign(StmtGen):
    def applicable(self, ctx: Context) -> bool:
        return len(ctx.vars) > 0

    def generate(self, ctx: Context) -> Stmt:
        name = random.choice(list(ctx.vars.keys()))
        expr = self.gen.any_expr.generate(ctx)
        ctx = Context(vars=ctx.vars | {name: expr})
        return Stmt([f'{name} = {expr.expr};'], ctx, [])


class ArrayAppend(StmtGen):
    def applicable(self, ctx: Context) -> bool:
        return any(isinstance(_, list) for _ in ctx.vars.values())

    def generate(self, ctx: Context) -> Stmt:
        name, old_value = random.choice(list((n, v) for n, v in ctx.vars.items() if isinstance(v, list)))
        expr = self.gen.any_expr.generate(ctx)
        ctx = Context(vars=ctx.vars | {name: old_value + [expr.value]})
        return Stmt([f'{name}[+] = {expr.expr};'], ctx, [])


class ArrayAssign(StmtGen):
    def applicable(self, ctx: Context) -> bool:
        return any(isinstance(_, list) and len(_) > 0 for _ in ctx.vars.values())

    def generate(self, ctx: Context) -> Stmt:
        name, old_value = random.choice(list((n, v) for n, v in ctx.vars.items() if isinstance(v, list)))
        expr = self.gen.any_expr.generate(ctx)
        i = random.randint(0, len(old_value) - 1)
        new_value = old_value[:]
        new_value[i] = expr.value
        ctx = Context(vars=ctx.vars | {name: new_value})
        return Stmt([f'{name}[{i}] = {expr.expr};'], ctx, [])


class ObjectAssign(StmtGen):
    def applicable(self, ctx: Context) -> bool:
        return any(isinstance(_, dict) for _ in ctx.vars.values())

    def generate(self, ctx: Context) -> Stmt:
        name, old_value = random.choice(list((n, v) for n, v in ctx.vars.items() if isinstance(v, dict)))
        expr = self.gen.any_expr.generate(ctx)
        if random.randint(0, 100) < 30 or len(old_value) == 0:
            key = self.gen.ident()
        else:
            key = random.choice(list(old_value.keys()))
        ctx = Context(vars=ctx.vars | {name: {**old_value, key: expr.value}})
        return Stmt([f'{name}[{json.dumps(key)}] = {expr.expr};'], ctx, [])


class PrintVar(StmtGen):
    def applicable(self, ctx: Context) -> bool:
        return len(ctx.vars) > 0

    def generate(self, ctx: Context) -> Stmt:
        name, value = random.choice(list(ctx.vars.items()))
        return Stmt([f'print({name});'], ctx, [value])


class CodeGenerator:
    MAX_EXPR_DEPTH = 5

    def __init__(self, collections: dict[str, list[dict[str, Any]]]) -> None:
        self.collections = collections
        self.current_depth = 0
        self.str_expr = ExprGenGroup([StringConst(self), StringVarUse(self), StringAdd(self),
                                      PickArray(self, str), PickObject(self, str)])
        self.int_expr = ExprGenGroup([IntConst(self), IntVarUse(self), IntOp(self), IntUnaryOp(self),
                                      PickArray(self, int), PickObject(self, int)])
        self.bool_expr = ExprGenGroup([BoolConst(self), BoolUnary(self), BoolDefined(self),
                                       BoolBinary(self), BoolComparison(self), BoolVarUse(self),
                                       PickArray(self, bool), PickObject(self, bool)])
        self.any_expr = ExprGenGroup([self.str_expr, self.int_expr, self.bool_expr, ArrayConstant(self), ObjectConstant(self)])
        self.stmt_basic = StmtGenGroup([VarDefine(self), VarAssign(self), ArrayAppend(self), ArrayAssign(self), ObjectAssign(self)])
        self.stmt_with_output = StmtGenGroup(self.stmt_basic.gens + [PrintVar(self)])

    def ident(self) -> str:
        s = random.choice(string.ascii_letters + '_')
        return s + ''.join(random.choice(string.ascii_letters + '_') for _ in range(random.randint(6, 11)))

    def collection(self, flag: str) -> Collection:
        name = ''.join(random.choice(string.ascii_lowercase) for _ in range(random.randint(6, 12)))
        items = [{} for _ in range(random.randint(1, 5))]

        ctx = Context({})
        # "key"
        ident_key = self.str_expr.generate(ctx)
        for item in items:
            item[ident_key] = self.str_expr.generate(ctx)
        # "data"
        flag_key = self.str_expr.generate(ctx)
        for i, item in enumerate(items):
            item[flag_key] = self.str_expr.generate(ctx) if i > 0 else Expr(json.dumps(flag), flag)
        # more things?
        if random.randint(1, 100) <= 65:
            k = self.str_expr.generate(ctx)
            for item in items:
                item[k] = self.int_expr.generate(ctx)
        if random.randint(1, 100) <= 65:
            k = self.str_expr.generate(ctx)
            for item in items:
                item[k] = self.json_expr(ctx)
        flag_item = items[0]
        random.shuffle(items)
        return Collection(name, items=items, ident_key=ident_key, flag_key=flag_key, flag_item=flag_item)

    def query_filter_func(self, collection: Collection, ident: Expr) -> Function:
        name = self.ident()
        var = self.ident()
        code = f'func {name}({var}) {{\n'
        code += '    return ' + self.equality_test(f'{var}[{collection.ident_key.expr}]', ident) + ';\n'
        code += '}'
        return Function(name, code, [], True, [])

    def query_selector_func(self, collection: Collection) -> Function:
        name = self.ident()
        var = self.ident()
        code = f'func {name}({var}) {{\n'
        code += f'    return {var}[{collection.flag_key.expr}];\n'
        code += '}'
        return Function(name, code, [], collection.flag_item[collection.flag_key].value, [])

    def flag_query(self, collection: Collection) -> Function:
        name = self.ident()
        code = f'query {name} on {collection.name}'
        data = collection.items
        if random.randint(1, 100) <= 70:
            filter = self.query_filter_func(collection, collection.flag_item[collection.ident_key])
            code = filter.code + '\n\n' + code
            code += f' filter {filter.name}'
            data = [collection.flag_item]
        if random.randint(1, 100) <= 70:
            selector = self.query_selector_func(collection)
            code = selector.code + '\n\n' + code
            code += f' map {selector.name}'
            data = [_[collection.flag_key] for _ in data]
        if len(data) == 1 and random.randint(1, 100) <= 70:
            l = random.randint(1, 10)
            code += f' limit {l}'
            data = data[:l]
        code += ';'
        return Function(name, code, [], data, [])

    @contextmanager
    def depth(self) -> Generator[None, Any, None]:
        self.current_depth += 1
        yield
        self.current_depth -= 1

    def json_expr(self, ctx: Context) -> Expr:
        c = random.choice(['true', 'false', 'null',
                           '[]', f'[{random.randint(0, 1000)}]',
                           '{}', f'{{"{self.ident()}": {random.randint(0, 1000)}}}'])
        return Expr(c, json.loads(c))

    def equality_test(self, expr1: str, expr2: Expr) -> str:
        if random.randint(1, 100) <= 50:
            return f'{expr1} == {expr2.expr}'
        else:
            return f'!({expr1} != {expr2.expr})'

    def function_any(self, max_params: int = 5, with_print: bool = True) -> Function:
        name = self.ident()
        output = []
        ctx = Context({})
        param_values = [self.any_expr.generate(ctx) for _ in range(random.randint(0, max_params))]
        param_names = [self.ident() for _ in range(len(param_values))]
        for n, v in zip(param_names, param_values):
            ctx.vars[n] = v.value
        code = f'func {name}({",".join(param_names)}) {{\n'

        for _ in range(10):
            stmt = (self.stmt_with_output if with_print else self.stmt_basic).generate(ctx)
            code += ''.join(f'    {s}\n' for s in stmt.code)
            ctx = stmt.ctx
            output += stmt.output

        result = self.any_expr.generate(ctx)
        code += '    return ' + result.expr + ';\n'
        code += '}\n'
        return Function(name, code, param_values, result.value, output)

    # TODO bool, objects, arrays, null, undefined
    # TODO include/use the unit tests?
    # TODO calls
    # TODO variables should be used more often in expressions
    # TODO if while etc
    # TODO use "system" etc as identifiers
