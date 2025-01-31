#include <iostream>

import parser;
import CodegenContext;
import JitRunner;
import direct_codegen;

std::string assemble(std::string code) {
    std::ofstream f("output.s");
    f << code;
    f.close();

    system("gcc -g output.s -shared -o ./output.so");
    return "output.so";
}

int main() {
    auto program = "print(print(1 + 2 + 3 + 4 + 5));";
    auto ast = parse_string(program);
    ast->print(std::cout);

    std::cout << "\n----------------------------------------" << std::endl << std::endl;

    /*
    std::stringstream ss;
    CodegenContext ctx(ss);
    ctx.start();
    ast->codegen(ctx);
    ctx.end();
    std::cout << ss.str() << "\n";

    auto assembled = assemble(ss.str());
     */

    auto assembly = compile_string(program);
    std::cout << assembly << std::endl;
    auto assembled = assemble(assembly);


    std::cout << "\n----------------------------------------" << std::endl << std::endl;
    JitRunner r;
    r.run(assembled);

    return 0;
}
