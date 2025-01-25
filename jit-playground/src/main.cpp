#include <iostream>

import parser;

int main() {
    auto ast = parse_string("1+2*3-4");
    ast->print(std::cout);
    return 0;
}
