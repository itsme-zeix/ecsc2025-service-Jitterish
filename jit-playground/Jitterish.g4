grammar Jitterish;

main: stmt EOF;

primary_expr
    : '(' expr ')'  # RawPrimaryExpr
    | NAME  # Named
    | INT  # ConstantInt
    | STRING_LITERAL  # ConstantString
    ;

expr: left=expr op=('*' | '/') rght=expr  # BinaryOp
    | left=expr op=('+' | '-') rght=expr  # BinaryOp
    | callee=primary_expr '(' (expr (',' expr)* ','?)? ')'  # CallExpr
    | primary_expr  # RawExpr
    ;

stmt: expr ';'  # ExprStmt
    | '{' stmt* '}'  # BlockStmt
    ;


// lexer rules
fragment DIGIT: [0-9];
fragment LETTER: [a-zA-Z];
INT: DIGIT+;
STRING_LITERAL: '"'.*?'"';
NAME: LETTER(LETTER | DIGIT)*;
WS: [\t\r\n ]+ -> skip;
