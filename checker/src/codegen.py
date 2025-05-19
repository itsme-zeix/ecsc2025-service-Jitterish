import json
import random
import re
import string
from abc import abstractmethod, ABC
from contextlib import contextmanager
from dataclasses import dataclass, field
from functools import wraps
from types import UnionType
from typing import Any, Generator, TypeAlias, Type, get_args, Callable, TypeVar, Hashable

Value: TypeAlias = int | str | list | dict | None
T = TypeVar('T', bound=Hashable)


@dataclass(frozen=True)
class Expr:
    expr: str
    value: Value
    output: tuple[Value, ...] = field(default_factory=lambda: ())

    def to_json(self) -> dict:
        return {
            'expr': self.expr,
            'value': self.value,
            'output': list(self.output),
        }

    @classmethod
    def from_json(cls, d: dict) -> 'Expr':
        return Expr(d['expr'], d['value'], tuple(d['output']))


@dataclass(frozen=True)
class Stmt:
    code: list[str]
    ctx: 'Context'
    output: list[Value] = field(default_factory=list)


@dataclass(frozen=True)
class Function:
    name: str
    code: str
    params: list[Expr]
    value: Value
    output: list[Value] = field(default_factory=list)

    def as_script(self) -> str:
        params = ' '.join(json.dumps(_.value) for _ in self.params)
        code = f'// EXECUTE {self.name} {params}\n\n' + self.code + '\n'
        code += f'\n// Result: {json.dumps(self.value)}'
        for o in self.output:
            code += f'\n// output: {json.dumps(o)}'
        return code


INT_MASK = 0xffffffffffffff


def expr_value(v: Any) -> Value:
    if isinstance(v, Expr):
        return v.value
    if isinstance(v, dict):
        return {expr_value(k): expr_value(v) for k, v in v.items()}
    if isinstance(v, list):
        return [expr_value(_) for _ in v]
    return v


def make_unique(c: Callable[[], T]) -> Callable[[], T]:
    used = set()

    @wraps(c)
    def wrapper() -> T:
        while True:
            s = c()
            if s not in used:
                used.add(s)
                return s

    return wrapper


@dataclass
class Collection:
    name: str
    items: list[dict[Expr, Expr]]
    ident_key: Expr
    flag_key: Expr
    flag_item: dict[Expr, Expr]

    def to_ndjson(self) -> str:
        return '\n'.join(json.dumps(expr_value(item)) for item in self.items)

    @property
    def private(self) -> bool:
        return 'private' in self.name

    def to_json(self) -> dict:
        return {
            'name': self.name,
            'items': [[(k.to_json(), v.to_json()) for k, v in item.items()] for item in self.items],
            'ident_key': self.ident_key.to_json(),
            'flag_key': self.flag_key.to_json(),
            'flag_item': [(k.to_json(), v.to_json()) for k, v in self.flag_item.items()],
        }

    @classmethod
    def from_json(cls, d: dict) -> 'Collection':
        return cls(
            name=d['name'],
            items=[{Expr.from_json(k): Expr.from_json(v) for k, v in item} for item in d['items']],
            ident_key=Expr.from_json(d['ident_key']),
            flag_key=Expr.from_json(d['flag_key']),
            flag_item={Expr.from_json(k): Expr.from_json(v) for k, v in d['flag_item']},
        )


@dataclass
class Context:
    vars: dict[str, Value] = field(default_factory=dict)
    funcs: dict[str, Function] = field(default_factory=dict)

    def update_var(self, name: str, value: Value) -> 'Context':
        return Context(vars=self.vars | {name: value}, funcs=self.funcs)


class IdentifierGenerator:
    FIRST_CHAR = string.ascii_letters + "____"
    ALLOWED_CHARS = string.ascii_letters + string.digits + "__"
    CANDIDATES = [
        'undefined',
        'NaN',
        'Infinity',
        'NUL',

        'system',
        'execve',
        'exec',
        'shell_exec',
        'eval',
        'puts',

        'SELECT',
        'INSERT',
        'UPDATE',

        'ProcessBuilder',
    ]

    def __init__(self) -> None:
        self.used = set()
        self.candidates = list(self.CANDIDATES)

    def get(self) -> str:
        while True:
            if len(self.candidates) > 1 and random.randint(0, 100) < 8:
                s = random.choice(self.candidates)
                self.candidates.remove(s)
            else:
                l = random.randint(3, 12)
                s = random.choice(self.FIRST_CHAR) + ''.join(random.choice(self.ALLOWED_CHARS) for _ in range(l - 1))
            if s not in self.used:
                self.used.add(s)
                return s


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


class TypedGen(ExprGen, ABC):
    def __init__(self, gen: 'CodeGenerator', generated_type: Type | UnionType) -> None:
        super().__init__(gen)
        if isinstance(generated_type, UnionType):
            self.generated_types = tuple(get_args(generated_type))
        else:
            self.generated_types = (generated_type,)

    def match(self, value: Value) -> bool:
        return self.generated_types == () or any(type(value) == t for t in self.generated_types)


class VarUse(TypedGen):
    def applicable(self, ctx: Context) -> bool:
        return any(self.match(_) for _ in ctx.vars.values())

    def generate(self, ctx: Context) -> Expr:
        variables = [(k, v) for k, v in ctx.vars.items() if self.match(v)]
        name, value = random.choice(variables)
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
        return Expr(f'({expr1.expr} {op} {expr2.expr})', result, expr1.output + expr2.output)


class IntUnaryOp(ExprGen):
    def generate(self, ctx: Context) -> Expr:
        expr = self.gen.int_expr.generate(ctx)
        return Expr(f'-{expr.expr}', (-expr.value) & INT_MASK, expr.output)


class StringConst(ExprGen):
    STRING_CHARS = string.ascii_letters + string.digits + "._-:/*+%$"
    CANDIDATES = [
        "<script>",
        "/bin/sh",
        "/bin/bash",
        "/etc/passwd",
        "ncat",
        "wget",
        "curl",
        "%n%n%n",
        "%p%p%p%p%p",
        "<img src=x onerror=alert(2) />",
        "&lt;script&gt;alert(&#39;1&#39;);&lt;/script&gt;",
        "1;DROP TABLE users",
        "' OR 1=1 -- 1",
        "`touch /tmp/blns.fail`",
        "DROP TABLE",
        "Kernel.exec('ls -al /')",
        "$HOME",
        "$ENV{'HOME'}",
        "%d",
        "file:///",
        "{% print 'x' * 64 * 1024**3 %}",
        "{{ ''.__class__.__mro__[2].__subclasses__()[40]('/etc/passwd').read() }}",
    ]

    @classmethod
    def raw(cls) -> str:
        if random.randint(0, 100) <= 5:
            return random.choice(cls.CANDIDATES)
        return ''.join(random.choice(cls.STRING_CHARS) for _ in range(random.randint(0, 12)))

    def generate(self, ctx: Context) -> Expr:
        s = self.raw()
        return Expr(json.dumps(s), s)


class StringAdd(ExprGen):
    def generate(self, ctx: Context) -> Expr:
        expr1 = self.gen.str_expr.generate(ctx)
        expr2 = self.gen.str_expr.generate(ctx) if random.randint(0, 2) else self.gen.int_expr.generate(ctx)
        return Expr(f'({expr1.expr} + {expr2.expr})', expr1.value + str(expr2.value), expr1.output + expr2.output)


class BoolConst(ExprGen):
    def generate(self, ctx: Context) -> Expr:
        c = random.choice((True, False))
        return Expr(json.dumps(c), c)


class BoolUnary(ExprGen):
    def generate(self, ctx: Context) -> Expr:
        expr = self.gen.bool_expr.generate(ctx)
        return Expr(f'!{expr.expr}', not expr.value, expr.output)


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
            Expr(f'({expr1.expr} && {expr2.expr})', bool(expr1.value and expr2.value), expr1.output + expr2.output),
            Expr(f'({expr1.expr} || {expr2.expr})', bool(expr1.value or expr2.value), expr1.output + expr2.output),
            Expr(f'({expr1.expr} == {expr2.expr})', bool(expr1.value) == bool(expr2.value), expr1.output + expr2.output),
            Expr(f'({expr1.expr} != {expr2.expr})', bool(expr1.value) != bool(expr2.value), expr1.output + expr2.output),
        ][op]


class BoolComparison(ExprGen):
    def generate(self, ctx: Context) -> Expr:
        expr1 = self.gen.int_expr.generate(ctx)
        expr2 = self.gen.int_expr.generate(ctx)
        return random.choice([
            Expr(f'({expr1.expr} >= {expr2.expr})', expr1.value >= expr2.value, expr1.output + expr2.output),
            Expr(f'({expr1.expr} <= {expr2.expr})', expr1.value <= expr2.value, expr1.output + expr2.output),
            Expr(f'({expr1.expr} > {expr2.expr})', expr1.value > expr2.value, expr1.output + expr2.output),
            Expr(f'({expr1.expr} < {expr2.expr})', expr1.value < expr2.value, expr1.output + expr2.output),
        ])


class ArrayConstant(ExprGen):
    def generate(self, ctx: Context) -> Expr:
        a = []
        if random.randint(0, 100) <= 50:
            a.append(random.randint(0, 1000))
        if random.randint(0, 100) <= 50:
            a.append(StringConst.raw())
        return Expr(json.dumps(a), a)


class ObjectConstant(ExprGen):
    def generate(self, ctx: Context) -> Expr:
        a = {}
        if random.randint(0, 100) <= 50:
            a[self.gen.field()] = random.randint(0, 1000)
        if random.randint(0, 100) <= 50:
            a[self.gen.field()] = StringConst.raw()
        return Expr(json.dumps(a), a)


class PickArray(TypedGen):
    def candidates(self, ctx: Context) -> list[tuple[str, int, Value]]:
        result = []
        for k, v in ctx.vars.items():
            if isinstance(v, list):
                for i, v2 in enumerate(v):
                    if self.match(v2):
                        result.append((k, i, v2))
        return result

    def applicable(self, ctx: Context) -> bool:
        return len(self.candidates(ctx)) > 0

    def generate(self, ctx: Context) -> Expr:
        cands = self.candidates(ctx)
        var, i, v = random.choice(cands)
        return Expr(f'{var}[{i}]', v)


class PickObject(TypedGen):
    def candidates(self, ctx: Context) -> list[tuple[str, str, Value]]:
        result = []
        for var, v in ctx.vars.items():
            if isinstance(v, dict):
                for k, v2 in v.items():
                    if self.match(v2):
                        result.append((var, k, v2))
        return result

    def applicable(self, ctx: Context) -> bool:
        return len(self.candidates(ctx)) > 0

    def generate(self, ctx: Context) -> Expr:
        cands = self.candidates(ctx)
        var, k, v = random.choice(cands)
        return Expr(f'{var}[{json.dumps(k)}]', v)


class Call(TypedGen):
    def applicable(self, ctx: Context) -> bool:
        return any(self.match(f.value) for f in ctx.funcs.values())

    def generate(self, ctx: Context) -> Expr:
        with self.gen.depth():
            candidates = [f for f in ctx.funcs.values() if self.match(f.value)]
            f: Function = random.choice(candidates)
            output: list[Value] = []
            for p in f.params:
                output += list(p.output)
            return Expr(f'{f.name}({", ".join(a.expr for a in f.params)})', f.value, tuple(output + f.output))


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
        name = self.gen.var_names.get()
        expr = self.gen.any_expr.generate(ctx)
        ctx = ctx.update_var(name, expr.value)
        return Stmt([f'{name} = {expr.expr};'], ctx, list(expr.output))


class VarAssign(StmtGen):
    def applicable(self, ctx: Context) -> bool:
        return len(ctx.vars) > 0

    def generate(self, ctx: Context) -> Stmt:
        name = random.choice(list(ctx.vars.keys()))
        expr = self.gen.any_expr_no_recursion.generate(ctx)
        ctx = ctx.update_var(name, expr.value)
        return Stmt([f'{name} = {expr.expr};'], ctx, list(expr.output))


class ArrayAppend(StmtGen):
    def applicable(self, ctx: Context) -> bool:
        return any(isinstance(_, list) for _ in ctx.vars.values())

    def generate(self, ctx: Context) -> Stmt:
        name, old_value = random.choice(list((n, v) for n, v in ctx.vars.items() if isinstance(v, list)))
        expr = self.gen.any_expr_no_recursion.generate(ctx)
        ctx = ctx.update_var(name, old_value + [expr.value])
        return Stmt([f'{name}[+] = {expr.expr};'], ctx, list(expr.output))


class ArrayAssign(StmtGen):
    def applicable(self, ctx: Context) -> bool:
        return any(isinstance(_, list) and len(_) > 0 for _ in ctx.vars.values())

    def generate(self, ctx: Context) -> Stmt:
        name, old_value = random.choice(list((n, v) for n, v in ctx.vars.items() if isinstance(v, list) and len(v) > 0))
        expr = self.gen.any_expr_no_recursion.generate(ctx)
        i = random.randint(0, len(old_value) - 1)
        new_value = old_value[:]
        new_value[i] = expr.value
        ctx = ctx.update_var(name, new_value)
        return Stmt([f'{name}[{i}] = {expr.expr};'], ctx, list(expr.output))


class ObjectAssign(StmtGen):
    def applicable(self, ctx: Context) -> bool:
        return any(isinstance(_, dict) for _ in ctx.vars.values())

    def generate(self, ctx: Context) -> Stmt:
        name, old_value = random.choice(list((n, v) for n, v in ctx.vars.items() if isinstance(v, dict)))
        expr = self.gen.any_expr_no_recursion.generate(ctx)
        if random.randint(0, 100) < 30 or len(old_value) == 0:
            key = self.gen.field()
        else:
            key = random.choice(list(old_value.keys()))
        ctx = ctx.update_var(name, {**old_value, key: expr.value})
        return Stmt([f'{name}[{json.dumps(key)}] = {expr.expr};'], ctx, list(expr.output))


class PrintVar(StmtGen):
    def applicable(self, ctx: Context) -> bool:
        return len(ctx.vars) > 0

    def generate(self, ctx: Context) -> Stmt:
        name, value = random.choice(list(ctx.vars.items()))
        return Stmt([f'print({name});'], ctx, [value])


class If(StmtGen):
    def generate(self, ctx: Context) -> Stmt:
        with self.gen.depth():
            gen = (self.gen.stmt_basic if self.gen.current_depth >= self.gen.MAX_EXPR_DEPTH else self.gen.stmt_full)
            expr = self.gen.bool_expr.generate(ctx)
            code = ['if ' + expr.expr + ' {']
            output: list[Value] = list(expr.output)
            stmt1 = gen.generate(ctx)
            code += ['    ' + _ for _ in stmt1.code]
            if expr.value:
                ctx = stmt1.ctx
                output += stmt1.output
            if random.randint(0, 100) <= 50:
                code.append('} else {')
                stmt2 = gen.generate(ctx)
                code += ['    ' + _ for _ in stmt2.code]
                if not expr.value:
                    ctx = stmt2.ctx
                    output += stmt2.output
            code.append('}')
        return Stmt(code, ctx, list(output))


class While(StmtGen):
    def generate(self, ctx: Context) -> Stmt:
        with self.gen.depth():
            gen = (self.gen.stmt_basic if self.gen.current_depth >= self.gen.MAX_EXPR_DEPTH else self.gen.stmt_full)
            expr = self.gen.bool_expr.generate(ctx)
            var = self.gen.var_names.get()
            code = [f'{var} = {expr.expr};', f'while {var} {{']
            ctx.update_var(var, expr.value)
            output: list[Value] = list(expr.output)
            stmt = gen.generate(ctx)
            code += ['    ' + _ for _ in stmt.code]
            if expr.value:
                code.append(f'{var} = false;')
                ctx = stmt.ctx.update_var(var, False)
                output += stmt.output
            code.append('}')
        return Stmt(code, ctx, list(output))


class TestExpressions(TypedGen):
    CONSTANTS = [
        Expr('(1+2*3+4-2)', 9),
        Expr('(1 && 2)', True),
        Expr('(1 && 0)', False),
        Expr('([!false, 5+-3, "a"+"b", "c"+7, 2*(1+1)])', [True, 2, "ab", "c7", 4]),
        Expr('([len {"a":0}, len [1,2], len "abc"])', [1, 2, 3]),
        Expr('([ [5,11,3][1], {"a":1,"b":2}["a"] ])', [11, 1]),
        Expr('([1==1,1!=1,1!=2, 1<2, 1>2, 1<=2, 1<=1, 1>=2])', [True, False, True, True, False, True, True, False]),
        Expr('(defined 1)', True),
        Expr('(defined {}["a"])', False),
        Expr('(-6 + 7)', 1),
        Expr('["a", {}, {"a":"b"}, null, 2, true, false]', ["a", {}, {"a": "b"}, None, 2, True, False]),
    ]

    def applicable(self, ctx: Context) -> bool:
        return any(self.match(_.value) for _ in self.CONSTANTS)

    def generate(self, ctx: Context) -> Expr:
        return random.choice([_ for _ in self.CONSTANTS if self.match(_.value)])


class TestStatements(StmtGen):
    CONSTANTS = [
        lambda ctx: Stmt(['a=[3,3,3];', 'a[0]=1;', 'a[1]=2;', 'a[2]=3;'], ctx.update_var('a', [1, 2, 3])),
        lambda ctx: Stmt(['a={"a":0};', 'a["a"]=1;', 'a["b"]=2;'], ctx.update_var('a', {"b": 2, "a": 1})),
        lambda ctx: Stmt(['a=[1];', 'a[+]=2;', 'a[+]=3;'], ctx.update_var('a', [1, 2, 3])),
        lambda ctx: Stmt(['a=[[], [], []];', 'a[0][+]=1;', 'a[1]=a[0];'], ctx.update_var('a', [[1], [1], []])),
        lambda ctx: Stmt(['i = 5; s = 0;', 'while i > 0 { s = s + i;  i = i - 1; }'],
                         ctx.update_var('s', 15).update_var('i', 0)),
    ]

    def generate(self, ctx: Context) -> Stmt:
        return random.choice(self.CONSTANTS)(ctx)


class CodeGenerator:
    MAX_EXPR_DEPTH = 5

    def __init__(self) -> None:
        self.current_depth = 0
        self.func_names = IdentifierGenerator()
        self.var_names = IdentifierGenerator()
        self.str_expr = ExprGenGroup([StringConst(self), VarUse(self, str), StringAdd(self),
                                      PickArray(self, str), PickObject(self, str),
                                      Call(self, str), TestExpressions(self, str)])
        self.int_expr = ExprGenGroup([IntConst(self), VarUse(self, int), IntOp(self), IntUnaryOp(self),
                                      PickArray(self, int), PickObject(self, int),
                                      Call(self, int), TestExpressions(self, int)])
        self.bool_expr = ExprGenGroup([BoolConst(self), BoolUnary(self), BoolDefined(self),
                                       BoolBinary(self), BoolComparison(self), VarUse(self, bool),
                                       PickArray(self, bool), PickObject(self, bool),
                                       Call(self, bool), TestExpressions(self, bool)])
        self.any_expr_no_recursion = ExprGenGroup([
            self.str_expr, self.int_expr, self.bool_expr,
            ArrayConstant(self), ObjectConstant(self),
            Call(self, Any), TestExpressions(self, Any),

            VarUse(self, str | int | bool | None),
        ])
        self.any_expr = ExprGenGroup(self.any_expr_no_recursion.gens[:-1] + [VarUse(self, Any)])
        # statements without recursion/output
        self.stmt_basic = StmtGenGroup([VarDefine(self), VarAssign(self),
                                        ArrayAppend(self), ArrayAssign(self), ObjectAssign(self),
                                        TestStatements(self)])
        # + calls/control structures
        self.stmt_full = StmtGenGroup(self.stmt_basic.gens + [If(self), While(self)])
        # + output
        self.stmt_with_output = StmtGenGroup(self.stmt_full.gens + [PrintVar(self)])

    def field(self) -> str:
        s = random.choice(string.ascii_letters + '_')
        return s + ''.join(random.choice(string.ascii_letters + '_') for _ in range(random.randint(6, 11)))

    def collection(self, flag: str, private: bool = False) -> Collection:
        name = ''.join(random.choice(string.ascii_lowercase) for _ in range(random.randint(6, 12)))
        if private:
            name = 'private' + name
        items = [{} for _ in range(random.randint(1, 5))]

        ctx = Context()
        # "key"
        ident_key = self.str_expr.generate(ctx)
        ident_key_gen = make_unique(lambda: self.str_expr.generate(ctx))
        for item in items:
            item[ident_key] = ident_key_gen()
        # "data"
        flag_key = self.str_expr.generate(ctx)
        for i, item in enumerate(items):
            item[flag_key] = self.str_expr.generate(ctx) if i > 0 else Expr(json.dumps(flag), flag)
        # more things?
        if random.randint(1, 100) <= 65:
            k = self.str_expr.generate(ctx)
            if k.value not in (flag_key.value, ident_key.value):
                for item in items:
                    item[k] = self.int_expr.generate(ctx)
        if random.randint(1, 100) <= 65:
            k = self.str_expr.generate(ctx)
            if k.value not in (flag_key.value, ident_key.value):
                for item in items:
                    item[k] = self.json_expr(ctx)
        flag_item = items[0]
        random.shuffle(items)
        return Collection(name, items=items, ident_key=ident_key, flag_key=flag_key, flag_item=flag_item)

    def query_filter_func(self, collection: Collection, ident: Expr) -> Function:
        name = self.func_names.get()
        var = self.var_names.get()
        code = f'func {name}({var}) {{\n'
        code += '    return ' + self.equality_test(f'{var}[{collection.ident_key.expr}]', ident) + ';\n'
        code += '}'
        return Function(name, code, [], True, [])

    def query_selector_func(self, collection: Collection) -> Function:
        name = self.func_names.get()
        var = self.var_names.get()
        code = f'func {name}({var}) {{\n'
        code += f'    return {var}[{collection.flag_key.expr}];\n'
        code += '}'
        return Function(name, code, [], collection.flag_item[collection.flag_key].value, [])

    def flag_query(self, collection: Collection) -> Function:
        name = self.func_names.get()
        code = f'query {name} on {collection.name}'
        data: list[Value] = expr_value(collection.items)
        if random.randint(1, 100) <= 70:
            filter_function = self.query_filter_func(collection, collection.flag_item[collection.ident_key])
            code = filter_function.code + '\n\n' + code
            code += f' filter {filter_function.name}'
            data = [expr_value(collection.flag_item)]
        if random.randint(1, 100) <= 70:
            selector_function = self.query_selector_func(collection)
            code = selector_function.code + '\n\n' + code
            code += f' map {selector_function.name}'
            data = [_[collection.flag_key.value] for _ in data]
        if len(data) == 1 and random.randint(1, 100) <= 70:
            l = random.randint(1, 10)
            code += f' limit {l}'
            data = data[:l]
        code += ';'
        return Function(name, code, [], value=data, output=[])

    @contextmanager
    def depth(self) -> Generator[None, Any, None]:
        self.current_depth += 1
        yield
        self.current_depth -= 1

    def json_expr(self, ctx: Context) -> Expr:
        c = random.choice(['true', 'false', 'null',
                           '[]', f'[{random.randint(0, 1000)}]',
                           '{}', f'{{"{self.field()}": {random.randint(0, 1000)}}}'])
        return Expr(c, json.loads(c))

    def equality_test(self, expr1: str, expr2: Expr) -> str:
        if random.randint(1, 100) <= 50:
            return f'{expr1} == {expr2.expr}'
        else:
            return f'!({expr1} != {expr2.expr})'

    def function_any(self, ctx: Context, max_params: int = 5, with_print: bool = True) -> Function:
        self.var_names = IdentifierGenerator()
        name = self.func_names.get()
        output = []
        ctx = Context({}, ctx.funcs)
        param_values = [self.any_expr.generate(ctx) for _ in range(random.randint(0, max_params))]
        param_names = [self.var_names.get() for _ in range(len(param_values))]
        for n, v in zip(param_names, param_values):
            ctx.vars[n] = v.value
        code = f'func {name}({",".join(param_names)}) {{\n'

        for _ in range(10):
            stmt = (self.stmt_with_output if with_print else self.stmt_full).generate(ctx)
            code += ''.join(f'    {s}\n' for s in stmt.code)
            ctx = stmt.ctx
            output += stmt.output

        result = self.any_expr.generate(ctx)
        output += result.output
        code += '    return ' + result.expr + ';\n'
        code += '}\n'
        return Function(name, code, param_values, result.value, output)

    def function_many(self, max_params: int = 5, max_additional_funcs: int = 9, with_print: bool = True) -> Function:
        ctx = Context()
        code = []
        for _ in range(random.randint(0, max_additional_funcs)):
            f = self.function_any(ctx, max_params=5, with_print=with_print)
            ctx.funcs[f.name] = f
            code.append(f.code)
        f = self.function_any(ctx, max_params=max_params, with_print=with_print)
        code.append(f.code)
        return Function(f.name, '\n\n'.join(code), f.params, f.value, f.output)

    @classmethod
    def uglify(cls, code: str) -> str:
        code = re.sub(r'".*?"', lambda m: m.group(0).replace(' ', '▅'), code)
        if random.randint(0, 100) <= 50:
            code = re.sub(r'\s+', ' ', code)
            code = re.sub(r' ([+*/=<>-]|&&|\|\||==|<=|>=) ', '\\1', code)
            code = re.sub('([;{}():,]) ', '\\1', code)
            code = re.sub(r' ([{(\[])', '\\1', code)
        else:
            rnd_whitespace = lambda m: ''.join(random.choice('   \r\n\t') for _ in range(random.randint(1, 4)))
            code = re.sub(r'\s+', rnd_whitespace, code)
        code = code.replace('▅', ' ')
        return code


def main() -> None:
    gen = CodeGenerator()
    func = gen.function_many(max_params=1)
    print(func.code)
    # print(CodeGenerator.uglify(func.code))
    print(f'// CALL {func.name} {" ".join(json.dumps(_.value) for _ in func.params)}')
    print(f'// => {json.dumps(func.value)}')


if __name__ == '__main__':
    main()
