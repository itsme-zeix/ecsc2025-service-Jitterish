grammar Jitterish;

main: func* EOF;

primary_expr
    : '(' expr ')'  # RawPrimaryExpr
    | left=primary_expr '[' rght=expr ']'  # PropertyGet
    // | left=primary_expr '.' right=NAME  # PropertyGetString
    | NAME  # Named
    | INT  # ConstantInt
    | STRING_LITERAL  # ConstantString
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
INT: DIGIT+;
STRING_LITERAL: '"'.*?'"';
NAME: LETTER(LETTER | DIGIT | '_')*;
WS: [\t\r\n ]+ -> skip;
