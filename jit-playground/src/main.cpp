#include <iostream>

import parser;
import CodegenContext;
import AsmWriter;

int main() {
    auto ast = parse_string("print(1 + 2);");
    ast->print(std::cout);

    std::cout << "\n----------------------------------------" << std::endl<< std::endl;

    CodegenContext ctx;
    ast->codegen(ctx);

    return 0;
}
