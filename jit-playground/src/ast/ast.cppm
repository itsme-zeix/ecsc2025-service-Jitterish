module;

#include <vector>
#include <memory>

export module ast;

import CodegenContext;
import AsmWriter;


export class Node;

export typedef std::shared_ptr<Node> NodeRef;

export class Node {
protected:
    std::vector<NodeRef> children;
public:
    virtual ~Node() = default;

    virtual void print(std::ostream &out) const = 0;

    virtual Register codegen(CodegenContext &ctx) {
        throw std::runtime_error("codegen not implemented");
    }
};

export std::ostream &operator<<(std::ostream &out, const Node &n) {
    n.print(out);
    return out;
}
