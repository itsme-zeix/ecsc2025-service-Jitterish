#include <iostream>

import CodegenContext;
import JitRunner;
import Compiler;

const std::string runtimeLibraryPath = "../cmake-build-debug";
const std::string runtimeLibraryPath2 = "/app";

void assemble(std::string code, std::string output) {
    std::ofstream f(output == "-" ? "tmp.s" : output + ".s");
    f << code;
    f.close();

    // or: clang-18 -fuse-ld=lld
    std::string cmd = "gcc";
    cmd += " -fPIC -g -pie -export-dynamic";
    if (!runtimeLibraryPath.empty())
        cmd += " -L" + runtimeLibraryPath + " -Wl,-rpath," + runtimeLibraryPath;
    if (!runtimeLibraryPath2.empty())
        cmd += " -L" + runtimeLibraryPath2 + " -Wl,-rpath," + runtimeLibraryPath2;
    cmd += " -ljit_rt";
    cmd += " " + output + ".s -o \"" + output + "\"";
    // std::cerr << cmd << std::endl;
    int rc = system(cmd.c_str());
    if (rc != 0)
        throw std::runtime_error("gcc failed");
}

std::string load_file(const char *filename) {
    std::ostringstream oss;
    if (std::string(filename) == "-") {
        oss << std::cin.rdbuf();
    } else {
        std::ifstream ifs(filename);
        oss << ifs.rdbuf();
    }
    return oss.str();
}

int demo_main(int argc, const char *argv[]) {
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
    assemble(assembly, "./output.so");


    std::cout << "\n----------------------------------------" << std::endl << std::endl;
    JitRunner::run("./output.so");

    return 0;
}

int real_main(int argc, const char *argv[]) {
    if (argc != 3) {
        std::cerr << "USAGE: " << argv[0] << " <INPUT> <OUTPUT>" << std::endl;
        return 1;
    }

    auto program = load_file(argv[1]);
    auto assembly = compile_string(program);
    assemble(assembly, argv[2]);
    return 0;
}

int main(int argc, const char *argv[]) {
    return real_main(argc, argv);
}
