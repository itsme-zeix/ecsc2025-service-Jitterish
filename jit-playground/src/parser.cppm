module;

#include "antlr4-runtime.h"
#include "generated/JitterishLexer.h"
#include "generated/JitterishParser.h"
#include "generated/JitterishVisitor.h"

export module parser;

import ast;
import stmt;
import expr;

using namespace antlr4;
using namespace generated_parser;


/**
 * Convert overly verbose ANTLR stuff into our own slim AST
 */
class AstGeneratingVisitor : public JitterishVisitor {
    template<class T>
    NodeRef ast(T *ctx) {
        return std::any_cast<NodeRef>(ctx->accept(this));
    }

    template<class T>
    std::vector<NodeRef> ast(const std::vector<T *> &ctx) {
        std::vector<NodeRef> result;
        for (auto x: ctx) {
            result.push_back(std::any_cast<NodeRef>(x->accept(this)));
        }
        return result;
    }

public:
    std::any visitMain(JitterishParser::MainContext *context) override {
        return ast(context->children.at(0));
    }

    std::any visitRawExpr(JitterishParser::RawExprContext *context) override {
        return ast(context->children.at(0));
    }

    std::any visitRawPrimaryExpr(JitterishParser::RawPrimaryExprContext *context) override {
        return ast(context->expr());
    }

    std::any visitConstantInt(JitterishParser::ConstantIntContext *context) override {
        return NodeRef(new ConstantInt(stoi(context->getText())));
    }

    std::any visitConstantString(JitterishParser::ConstantStringContext *context) override {
        return NodeRef(new ConstantString(context->getText()));
    }

    std::any visitBinaryOp(JitterishParser::BinaryOpContext *context) override {
        auto op = context->op->getText();
        return NodeRef(new BinaryOp(op, ast(context->left), ast(context->rght)));
    }

    std::any visitCallExpr(JitterishParser::CallExprContext *context) override {
        auto call = new Call(ast(context->primary_expr()), ast(context->expr()));
        return NodeRef(call);
    }

    std::any visitNamed(JitterishParser::NamedContext *context) override {
        return NodeRef(new NamedExpr(context->getText()));
    }

    std::any visitExprStmt(JitterishParser::ExprStmtContext *context) override {
        return NodeRef(new ExprStmt(ast(context->expr())));
    }

    std::any visitBlockStmt(JitterishParser::BlockStmtContext *context) override {
        return NodeRef(new BlockStmt(ast(context->stmt())));
    }
};


export NodeRef parse_string(const std::string &s) {
    ANTLRInputStream input(s);
    JitterishLexer lexer(&input);
    CommonTokenStream tokens(&lexer);
    tokens.fill();
    JitterishParser parser(&tokens);
    tree::ParseTree *tree = parser.main();

    // std::cout << tree->toStringTree(&parser, true) << std::endl << std::endl;

    AstGeneratingVisitor v;
    return std::any_cast<NodeRef>(tree->accept(&v));
}
