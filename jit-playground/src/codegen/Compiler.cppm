module;

#include <string>
#include <ranges>
#include "antlr4-runtime.h"
#include "../generated/JitterishLexer.h"
#include "../generated/JitterishParser.h"
#include "../generated/JitterishBaseVisitor.h"

export module Compiler;

import CodegenContext;

using namespace antlr4;
using namespace generated_parser;


class Variables : public JitterishBaseVisitor {
    int stackCount = 0;
    int paramCount = 0;
public:
    std::map<std::string, int> offsets;

    int stackSize() const { return stackCount * 8; }

    std::any visitAssignToNamed(JitterishParser::AssignToNamedContext *ctx) override {
        auto name = ctx->NAME()->getText();
        if (offsets.find(name) == offsets.end()) {
            offsets[name] = (++stackCount) * -8;
        }
        return {};
    }

    void addParam(std::string name) {
        offsets[name] = (++paramCount) * 8;
    }
};


class CodegenVisitor : public JitterishBaseVisitor {
    std::stringstream ss;

    CodegenContext codegen;

    Variables vars;

protected:
    std::any defaultResult() override {
        return std::any(Register::None);
    }

    std::any aggregateResult(std::any aggregate, std::any nextResult) override {
        // default visitor methods should just pass through any used register
        auto r1 = any_cast<Register>(aggregate);
        auto r2 = any_cast<Register>(nextResult);
        if (r1 == Register::None) return r2;
        if (r2 == Register::None) return r1;
        // we must handle all cases with multiple used registers manually
        throw std::runtime_error("not implemented");
    }

    template<class T>
    Register compile(T *ctx) {
        return std::any_cast<Register>(ctx->accept(this));
    }

public:
    CodegenVisitor() : codegen(ss) {}

    std::string str() const { return ss.str(); }

    std::any visitMain(JitterishParser::MainContext *ctx) override {
        codegen.start();
        for (auto f: ctx->func()) {
            compile(f);
        }
        codegen.end();
        return Register::None;
    }

    std::any visitFunc(JitterishParser::FuncContext *ctx) override {
        vars = Variables();
        vars.visit(ctx->body);
        for (auto arg: ctx->NAME())
            std::cout << "DEBUG: ARG = " << arg->getText() << "\n";
        for (auto arg: ctx->NAME())
            vars.addParam(arg->getText());
        codegen.startFunction(ctx->name->getText(), vars.stackSize());
        compile(ctx->body);
        codegen.endFunction();
        return Register::None;
    }

    std::any visitReturnStmt(JitterishParser::ReturnStmtContext *ctx) override {
        auto reg = compile(ctx->expr());
        codegen.returnFunction(reg);
        codegen.releaseRegister(reg);
        return Register::None;
    }

    std::any visitExprStmt(JitterishParser::ExprStmtContext *ctx) override {
        codegen.releaseRegister(compile(ctx->expr()));
        return Register::None;
    }

    std::any visitBinaryOp(JitterishParser::BinaryOpContext *ctx) override {
        auto op = ctx->op->getText();
        auto lft = compile(ctx->left);
        auto rght = compile(ctx->rght);
        if (op == "+") {
            return codegen.callRuntime("jit_rt_add", lft, rght);
        } else if (op == "-") {
            return codegen.callRuntime("jit_rt_sub", lft, rght);
        } else if (op == "*") {
            return codegen.callRuntime("jit_rt_mul", lft, rght);
        } else if (op == "/") {
            return codegen.callRuntime("jit_rt_div", lft, rght);
        }
        throw std::runtime_error("unknown operator: " + op);
    }

    std::any visitCallExpr(JitterishParser::CallExprContext *ctx) override {
        if (ctx->callee->getText() == "print")
            return codegen.callRuntime("jit_rt_print", compile(ctx->expr(0)));

        auto saved = codegen.pushTakenRegistersBeforeCall();
        for (auto param: std::ranges::reverse_view(ctx->expr())) {
            codegen.pushCallArg(compile(param));
        }

        auto result = codegen.call(ctx->callee->getText());

        for (auto param: ctx->expr()) {
            codegen.releaseRegister(codegen.popCallArg());
        }
        codegen.popTakenRegistersAfterCall(saved);
        return result;
    }

    std::any visitConstantInt(JitterishParser::ConstantIntContext *ctx) override {
        return codegen.constantInt(stoi(ctx->getText()));
    }

    // === Variables ===

    std::any visitNamed(JitterishParser::NamedContext *ctx) override {
        auto name = ctx->NAME()->getText();
        return codegen.stackLoad(vars.offsets.at(name));
    }

    std::any visitAssignment(JitterishParser::AssignmentContext *ctx) override {
        auto value = compile(ctx->rght);
        if (auto named = dynamic_cast<JitterishParser::AssignToNamedContext *>(ctx->left)) {
            codegen.stackStore(vars.offsets.at(named->NAME()->getText()), value);
            return value;
        } else {
            throw std::runtime_error("not implemented");
        }
    }

};


export std::string compile_string(const std::string &s) {
    ANTLRInputStream input(s);
    JitterishLexer lexer(&input);
    CommonTokenStream tokens(&lexer);
    tokens.fill();
    JitterishParser parser(&tokens);
    tree::ParseTree *tree = parser.main();

    CodegenVisitor v;
    tree->accept(&v);
    return v.str();
}
