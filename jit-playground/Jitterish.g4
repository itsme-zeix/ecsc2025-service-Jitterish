grammar Jitterish;

main: func* EOF;

constant_expr
    : STRING_LITERAL
    | INT
    | 'null' | 'true' | 'false'
    | '[' (constant_expr ','?) * ']'
    | '{' (STRING_LITERAL ':' constant_expr ','?)* '}'
    ;

primary_expr
    : '(' expr ')'  # RawPrimaryExpr
    | left=primary_expr '[' rght=expr ']'  # PropertyGet
    // | left=primary_expr '.' right=NAME  # PropertyGetString
    | NAME  # Named
    | INT  # ConstantInt
    | constant_expr  # ConstantExpr
    ;

assignable_expr
    : NAME  # AssignToNamed
    | left=primary_expr '[' right=expr ']'  # AssignToProperty
    // add "x[y]" and "x.y" later
    ;

expr: left=expr op=('*' | '/') rght=expr  # BinaryOp
    | left=expr op=('+' | '-') rght=expr  # BinaryOp
    | left=expr op=('==' | '!=' | '<' | '>' | '<=' | '>=') rght=expr  # BinaryOp
    | left=assignable_expr '=' rght=expr  # Assignment
    | callee=NAME '(' (expr (',' expr)* ','?)? ')'  # CallExpr
    | primary_expr  # RawExpr
    ;

stmt: expr ';'  # ExprStmt
    | stmtBlock  # BlockStmt
    | 'return' expr ';'  # ReturnStmt
    ;

stmtBlock: '{' stmt* '}';

func: 'func' name=NAME '(' (arg=NAME ','?)* ')' body=stmtBlock;


// lexer rules
fragment DIGIT: [0-9];
fragment LETTER: [a-zA-Z];
fragment LETTERU: [a-zA-Z_];
INT: DIGIT+;
STRING_LITERAL: '"'.*?'"';
NAME: LETTERU(LETTERU | DIGIT)*;
WS: [\t\r\n ]+ -> skip;
