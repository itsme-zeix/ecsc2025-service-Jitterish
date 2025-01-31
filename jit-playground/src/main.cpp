#include <iostream>

import CodegenContext;
import JitRunner;
import Compiler;

std::string assemble(std::string code) {
    std::ofstream f("output.s");
    f << code;
    f.close();

    system("gcc -g output.s -shared -o ./output.so");
    return "output.so";
}

int main() {
    auto program = "print(print(1 + 2 + 3 + 4 + 5));";

    std::cout << "\n----------------------------------------" << std::endl << std::endl;

    auto assembly = compile_string(program);
    std::cout << assembly << std::endl;
    auto assembled = assemble(assembly);


    std::cout << "\n----------------------------------------" << std::endl << std::endl;
    JitRunner r;
    r.run(assembled);

    return 0;
}
