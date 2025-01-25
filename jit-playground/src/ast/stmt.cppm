module;

#include <iostream>
#include <vector>
#include <memory>

export module stmt;

import ast;
import expr;
import CodegenContext;

export class Stmt : public Node {
};

export class ExprStmt : public Stmt {
public:
    explicit ExprStmt(const NodeRef &expr) {
        children.push_back(expr);
    }

    void print(std::ostream &out) const override {
        out << *children[0] << ";";
    }

    Register codegen(CodegenContext &ctx) override {
        auto reg = children[0]->codegen(ctx);
        ctx.releaseRegister(reg);
        // TODO free value?
        return Register::None;
    }

};

export class BlockStmt : public Stmt {
public:
    explicit BlockStmt(const std::vector<NodeRef> &stmts) {
        children.assign(stmts.begin(), stmts.end());
    }

    void print(std::ostream &out) const override {
        out << "{\n";
        for (const auto &c: children)
            out << "\t" << *c << "\n";
        out << "}";
    }

    Register codegen(CodegenContext &ctx) override {
        for (auto &c: children) {
            c->codegen(ctx);
        }
        return Register::None;
    }
};
