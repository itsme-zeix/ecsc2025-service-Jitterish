#include <iostream>

import parser;
import CodegenContext;
import AsmWriter;

int main() {
    // auto ast = parse_string("print(print(1 + 2) + print(3));");
    auto ast = parse_string("print(1 + 2 + 3 + 4);");
    ast->print(std::cout);

    std::cout << "\n----------------------------------------" << std::endl<< std::endl;

    CodegenContext ctx(std::cout);
    ast->codegen(ctx);

    return 0;
}
