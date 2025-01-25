module;

#include <string>
#include <iostream>
#include "../rtlib/jit_runtime.h"

export module AsmWriter;

// call arguments: RDI, RSI, RDX, RCX, R8, R9
// callee saves RBX, RSP, RBP, and R12–R15
// caller saves rax, rcx, rdx, ...

export enum Register {
    None = 0,
    RAX,
    RBX,
    RCX,
    RDX,
    RSI,
    RDI
};

static const char *reg[] = {
        "INVALID-REGISTER",
        "rax",
        "rbx",
        "rcx",
        "rdx",
        "rsi",
        "rdi"
};

export class AsmWriter {
    std::ostream &out;

public:
    explicit AsmWriter(std::ostream &out) : out(out) {}

    Register callRuntime(std::string name, Register arg1, Register arg2 = Register::None) {
        out << "movq " << reg[arg1] << " " << reg[Register::RDI] << "\n";
        if (arg2 != Register::None)
            out << "movq " << reg[arg2] << " " << reg[Register::RSI] << "\n";
        out << "call " << name << "\n";
        return Register::RAX;
    }

    Register constantInt(int i) {
        out << "movq " << Value(ValueType::INT, i).raw() << " " << reg[Register::RBX] << "\n";
        return Register::RBX;
    }
};
