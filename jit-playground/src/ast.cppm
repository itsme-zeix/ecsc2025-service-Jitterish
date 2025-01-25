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

export class ConstantInt : public Expr {
protected:
    int i;
public:
    explicit ConstantInt(int i) : i(i) {}

    void print(std::ostream &out) const override {
        out << i;
    }
};
