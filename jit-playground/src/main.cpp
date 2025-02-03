#include <iostream>

import CodegenContext;
import JitRunner;
import Compiler;

std::string assemble(std::string code) {
    std::ofstream f("output.s");
    f << code;
    f.close();

    int rc = system("gcc -fPIC -g output.s -shared -o ./output.so");
    if (rc != 0)
        throw std::runtime_error("gcc failed");
    return "./output.so";
}

std::string load_file(const char* filename) {
    std::ifstream ifs(filename);
    std::ostringstream oss;
    oss << ifs.rdbuf();
    return oss.str();
}

int main(int argc, const char* argv[]) {
    // auto program = "print(print(1 + 2 + 3 + 4 + 5));";
    // auto program = "func jit_entrypoint(){ a = 1 + 2 * (1+2) - 4; print(a); print(a+1); }";
    // auto program = "func helper(a, b) { return a+b; } func jit_entrypoint(){ return 4 + helper(1, 2) + helper(7, 1) + 3; }";
    // auto program = "func jit_entrypoint() { system(1); }";
    // auto program = R"(func jit_entrypoint() { return [1, 2+3, "4", {"a": 15, "b": "c"+"d"}, 1 && 2, 0 || !false, 5 + -3]; })";
    std::string program = R"(func jit_entrypoint() { a = []; print(len a); a[+] = 1; print(len a); return a; })";
    if (argc > 1) {
        program = load_file(argv[1]);
    }

    std::cout << "\n----------------------------------------" << std::endl << std::endl;

    auto assembly = compile_string(program);
    std::cout << assembly << std::endl;
    auto assembled = assemble(assembly);


    std::cout << "\n----------------------------------------" << std::endl << std::endl;
    JitRunner::run(assembled);

    return 0;
}
