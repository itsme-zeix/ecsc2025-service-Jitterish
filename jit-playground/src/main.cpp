#include <iostream>

import parser;
import CodegenContext;
import AsmWriter;
import JitRunner;

std::string assemble(std::string code) {
    std::ofstream f("output.s");
    f << code;
    f.close();

    system("gcc -g output.s -shared -o ./output.so");
    return "output.so";
}

int main() {
    auto ast = parse_string("print(print(1 + 2 + 3 + 4 + 5));");
    ast->print(std::cout);

    std::cout << "\n----------------------------------------" << std::endl << std::endl;

    std::stringstream ss;
    CodegenContext ctx(ss);
    ctx.start();
    ast->codegen(ctx);
    ctx.end();
    std::cout << ss.str() << "\n";

    auto assembled = assemble(ss.str());

    std::cout << "\n----------------------------------------" << std::endl << std::endl;
    JitRunner r;
    r.run(assembled);

    return 0;
}
