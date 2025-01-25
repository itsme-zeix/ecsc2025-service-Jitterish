#include <iostream>

import parser;

int main() {
    auto ast = parse_string("print(1 + 2);");
    ast->print(std::cout);
    return 0;
}
