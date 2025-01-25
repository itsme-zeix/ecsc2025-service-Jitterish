grammar Jitterish;

main: expr EOF;
number: INT;
expr: expr op=(MUL | DIV) expr  # BinaryOp
    | expr op=(ADD | SUB) expr  # BinaryOp
    | '(' expr ')'  # RawExpr
    | number  # RawExpr
    ;


// lexer rules
fragment DIGIT: [0-9];
fragment LETTER: [a-zA-Z];
INT: DIGIT+;
STRING_LITERAL: '"'.*?'"';
NAME: LETTER(LETTER | DIGIT)*;
IDENTIFIER: [a-zA-Z0-9]+;
MUL: '*';
DIV: '/';
ADD: '+';
SUB: '-';
WS: [trn]+ -> skip;
