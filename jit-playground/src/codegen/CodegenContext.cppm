module;

#include <iostream>
#include <ranges>
#include <stack>
#include <set>
#include "../rtlib/jit_runtime.h"

export module CodegenContext;

import AsmWriter;

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
    RDI,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,

    MAX_REGISTER_VALUE
};

struct RegisterInfo {
    const char *name;
    bool callerSaved;
};

static RegisterInfo infos[MAX_REGISTER_VALUE] = {
        {"INVALID-REGISTER", false},
        {"rax",              true},
        {"rbx",              false},
        {"rcx",              true},
        {"rdx",              true},
        {"rsi",              true},
        {"rdi",              true},
        {"r8",               true},
        {"r9",               true},
        {"r10",              true},
        {"r11",              true},
        {"r12",              false},
        {"r13",              false},
        {"r14",              false},
        {"r15",              false},
        //{"INVALID-REGISTER", false},
};

export std::ostream &operator<<(std::ostream &out, const Register &r) {
    out << infos[r].name;
    return out;
}


export class CodegenContext {
    std::ostream &out;
    std::stack<Register> freeRegisters;
    std::array<bool, MAX_REGISTER_VALUE> registerTaken;
public:
    explicit CodegenContext(std::ostream &out) : out(out) {
        for (int r = None; r < MAX_REGISTER_VALUE; r++) {
            releaseRegister((Register) r);
        }
    }

    Register takeRegister() {
        // TODO register selection might be simpler. For example: sort register enum, callee-saved before caller-saved
        // TODO rework register selection when we have a calling convention for our functions
        if (freeRegisters.empty()) {
            for (int r = Register::R11; r > Register::None; r--) {
                if (!registerTaken[r]) {
                    registerTaken[r] = true;
                    return (Register) r;
                }
            }
            throw std::runtime_error("no registers available");
        } else {
            auto reg = freeRegisters.top();
            freeRegisters.pop();
            registerTaken[reg] = true;
            return reg;
        }
    }

    void releaseRegister(Register r) {
        if (!infos[r].callerSaved)
            freeRegisters.push(r);
        registerTaken[r] = false;
    }

    // === asm utility functions ===

    std::stack<Register> pushTakenRegistersBeforeCall() {
        std::stack<Register> result;
        for (int i = 0; i < registerTaken.size(); i++) {
            if (registerTaken[i] && infos[i].callerSaved) {
                out << "pushq " << (Register) i << "\n";
                result.push((Register) i);
            }
        }
        return result;
    }

    void popTakenRegistersAfterCall(std::stack<Register> registers) {
        while (!registers.empty()) {
            out << "popq " << registers.top() << "\n";
            registers.pop();
        }
    }

    Register callRuntime(std::string name, Register arg1, Register arg2 = Register::None) {
        if (arg1 != Register::None && arg1 != Register::RDI) {
            out << "movq " << arg1 << " " << Register::RDI << "\n";
            releaseRegister(arg1);
        }
        if (arg2 != Register::None && arg2 != Register::RSI) {
            out << "movq " << arg2 << " " << Register::RSI << "\n";
            releaseRegister(arg2);
        }

        auto storedRegisters = pushTakenRegistersBeforeCall();
        out << "call " << name << "\n";

        // result register is rax by default, unless already taken
        auto result = Register::RAX;
        if (registerTaken[Register::RAX]) {
            result = takeRegister();
            out << "movq " << Register::RAX << " " << result << "\n";
        } else {
            registerTaken[Register::RAX] = true;
        }

        popTakenRegistersAfterCall(storedRegisters);

        return result;
    }

    Register constantInt(int i) {
        auto reg = takeRegister();
        out << "movq " << Value(ValueType::INT, i).raw() << " " << reg << "  ; = int " << i << "\n";
        return reg;
    }

};
