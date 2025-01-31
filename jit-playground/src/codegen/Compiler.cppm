module;

#include <string>
#include "antlr4-runtime.h"
#include "../generated/JitterishLexer.h"
#include "../generated/JitterishParser.h"
#include "../generated/JitterishBaseVisitor.h"

export module Compiler;

import CodegenContext;

using namespace antlr4;
using namespace generated_parser;


struct Variables : public JitterishBaseVisitor {
    std::map<std::string, int> offsets;

    int stackSize() { return offsets.size() * 8; }

    std::any visitAssignToNamed(JitterishParser::AssignToNamedContext *ctx) override {
        auto name = ctx->NAME()->getText();
        if (offsets.find(name) == offsets.end()) {
            offsets[name] = offsets.size() * 8;
        }
        return {};
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

        vars = Variables();
        vars.visit(ctx->stmt());
        codegen.startFunction("jit_entrypoint", vars.stackSize());

        compile(ctx->stmt());

        codegen.endFunction(vars.stackSize());
        codegen.end();
        return Register::None;
    }

    std::any visitBlockStmt(JitterishParser::BlockStmtContext *ctx) override {
        for (auto stmt: ctx->stmt()) {
            codegen.releaseRegister(compile(stmt));
        }
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
        auto arg = compile(ctx->expr(0));
        return codegen.callRuntime("jit_rt_print", arg);
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
        if (auto named = dynamic_cast<JitterishParser::AssignToNamedContext*>(ctx->left)) {
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
