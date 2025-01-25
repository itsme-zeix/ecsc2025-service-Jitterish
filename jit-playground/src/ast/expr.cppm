module;

#include <vector>
#include <memory>
#include <string>
#include <iostream>

export module expr;

import ast;
import CodegenContext;

export class Expr : public Node {
};

// variable/function name
export class NamedExpr : public Node {
protected:
    std::string name;
public:
    explicit NamedExpr(std::string name) : name(std::move(name)) {}

    void print(std::ostream &out) const override {
        out << name;
    }
};

export class BinaryOp : public Expr {
protected:
    std::string op;
public:
    BinaryOp(std::string op, const NodeRef &left, const NodeRef &right) : op(std::move(op)) {
        children.push_back(left);
        children.push_back(right);
    }

    void print(std::ostream &out) const override {
        out << "(" << *children[0] << " " << op << " " << *children[1] << ")";
    }

    Register codegen(CodegenContext &ctx) override {
        if (op == "+") {
            auto lft = children[0]->codegen(ctx);
            auto rght = children[1]->codegen(ctx);
            auto result = ctx.callRuntime("jit_rt_add", lft, rght);
            return result;
        }
        return Expr::codegen(ctx);
    }
};

export class Call : public Expr {
public:
    explicit Call(const NodeRef &callee, std::vector<NodeRef> params) {
        children.push_back(callee);
        children.insert(children.end(), params.begin(), params.end());
    }

    void print(std::ostream &out) const override {
        out << *children[0] << "(";
        for (size_t i = 1; i < children.size(); i++) {
            if (i > 1) out << ", ";
            out << *children[i];
        }
        out << ")";
    }

    Register codegen(CodegenContext &ctx) override {
        auto arg = children[1]->codegen(ctx);
        auto result = ctx.callRuntime("jit_rt_print", arg);
        return result;
    }
};

export class ConstantInt : public Expr {
protected:
    int i;
public:
    explicit ConstantInt(int i) : i(i) {}

    void print(std::ostream &out) const override {
        out << i;
    }

    Register codegen(CodegenContext &ctx) override {
        return ctx.constantInt(i);
    }
};

export class ConstantString : public Expr {
protected:
    std::string s;
public:
    explicit ConstantString(std::string s) : s(std::move(s)) {}

    void print(std::ostream &out) const override {
        out << s;
    }
};
