#include <iostream>
#include <filesystem>

import CodegenContext;
import JitRunner;
import Compiler;


std::string runtime_library_path() {
    // libjit_rt is in the same directory as the compiler binary
    return std::filesystem::absolute(program_invocation_name).parent_path().string();
}

void assemble(std::string code, std::string output) {
    auto runtime_path = runtime_library_path();

    auto assembly_file = output + ".s";
    std::ofstream f(assembly_file);
    f << code;
    f.close();

    // or: clang-18 -fuse-ld=lld
    std::string cmd = "cc";
    cmd += " -fPIC -g -pie -Wl,-export-dynamic";
    cmd += " -L" + runtime_path + " -Wl,-rpath," + runtime_path;
    cmd += " -ljit_rt";
    cmd += " " + assembly_file + " -o \"" + output + "\"";
    // std::cerr << cmd << std::endl;
    int rc = system(cmd.c_str());
    if (rc != 0)
        throw std::runtime_error("cc failed");
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

void file_to_stdout(std::string filename) {
    std::ifstream ifs(filename);
    std::cout << ifs.rdbuf();
}

int demo_main(int argc, const char *argv[]) {
    std::string program = R"(func jit_entrypoint() { return 1; })";
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
    std::string output = argv[2];
    if (output == "-") {
        assemble(assembly, "/tmp/tmp.so");
        file_to_stdout("/tmp/tmp.so");
    } else {
        assemble(assembly, output);
    }
    return 0;
}

int main(int argc, const char *argv[]) {
    try {
        return real_main(argc, argv);
    } catch (const std::runtime_error &e) {
        std::cerr << e.what() << std::endl;
        std::cout << e.what() << std::endl;
        return 1;
    }
}
