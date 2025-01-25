module;

#include <utility>
#include <vector>
#include <memory>

export module ast;


export class Node;

export typedef std::shared_ptr<Node> NodeRef;

export class Node {
protected:
    std::vector<NodeRef> children;
public:
    virtual ~Node() = default;

    virtual void print(std::ostream &out) const = 0;
};

std::ostream &operator<<(std::ostream &out, const Node &n) {
    n.print(out);
    return out;
}


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
};

export class Call : public Expr {
public:
    explicit Call(const NodeRef &callee) {
        children.push_back(callee);
    }

    void addParam(const NodeRef &param) {
        children.push_back(param);
    }

    void print(std::ostream &out) const override {
        out << *children[0] << "(";
        for (size_t i = 1; i < children.size(); i++) {
            if (i > 1) out << ", ";
            out << *children[i];
        }
        out << ")";
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
};

export class BlockStmt : public Stmt {
public:
    explicit BlockStmt(const std::vector<NodeRef> &stmts) {
        children.assign(stmts.begin(), stmts.end());
    }

    void print(std::ostream &out) const override {
        out << "{\n";
        for (const auto& c: children)
            out << "\t" << *c << "\n";
        out << "}";
    }
};