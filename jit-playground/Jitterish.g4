grammar Jitterish;

main: func* query* EOF;


primary_expr
    : '(' expr ')'  # RawPrimaryExpr
    | op=('!' | '-' | 'len') expr  # UnaryOp
    | left=primary_expr '[' rght=expr ']'  # PropertyGet
    | NAME  # Named
    | INT  # ConstantInt
    | ('null' | 'true' | 'false' | STRING_LITERAL)  # ConstantExpr
    | '[' (expr ','?) * ']'  # ArrayExpr
    | '{' (STRING_LITERAL ':' expr ','?)* '}'  # ObjectExpr
    ;

assignable_expr  // left side of an assignment
    : NAME  # AssignToNamed
    | left=primary_expr '[' right=expr ']'  # AssignToProperty
    | left=primary_expr '[' '+' ']'  # AssignAppend
    ;

expr: left=expr op=('*' | '/') rght=expr  # BinaryOp
    | left=expr op=('+' | '-') rght=expr  # BinaryOp
    | left=expr op=('==' | '!=' | '<' | '>' | '<=' | '>=') rght=expr  # BinaryOp
    | left=expr op=('||' | '&&' | '?:') rght=expr  # BinaryOp
    | left=assignable_expr '=' rght=expr  # Assignment
    | callee=NAME '(' (expr (',' expr)* ','?)? ')'  # CallExpr
    | primary_expr  # RawExpr
    ;

stmt: expr ';'  # ExprStmt
    | stmtBlock  # BlockStmt
    | 'return' expr ';'  # ReturnStmt
    | 'if' expr caseTrue=stmt ('else' caseFalse=stmt)?  # IfStmt
    | 'while' expr stmt  # WhileStmt
    ;

stmtBlock: '{' stmt* '}';

func: 'func' name=NAME '(' (arg=NAME ','?)* ')' body=stmtBlock;

query:
    'query' name=NAME
    'on' collection=NAME
    ('filter' filter=NAME)?
    ('map' map=NAME)?
    ('reduce' reduce=NAME)?
    ('limit' limit=INT)?
    ';';


// lexer rules
fragment DIGIT: [0-9];
fragment LETTER: [a-zA-Z_];
INT: DIGIT+;
STRING_LITERAL: '"'.*?'"';
NAME: LETTER(LETTER | DIGIT)*;
WS: [\t\r\n ]+ -> skip;
COMMENT: '/*' .*? '*/' -> skip;
LINE_COMMENT: '//' ~[\r\n]* -> skip;
