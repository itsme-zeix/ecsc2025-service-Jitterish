module;

#include "antlr4-runtime.h"
#include "generated/JitterishLexer.h"
#include "generated/JitterishParser.h"
#include "generated/JitterishVisitor.h"

import ast;

export module parser;

using namespace antlr4;
using namespace generated_parser;


/**
 * Convert overly verbose ANTLR stuff into our own slim AST
 */
class AstGeneratingVisitor : public JitterishVisitor {
public:
    std::any visitMain(JitterishParser::MainContext *context) override {
        return context->children.at(0)->accept(this);
    }

    std::any visitNumber(JitterishParser::NumberContext *context) override {
        return NodeRef(new ConstantInt(stoi(context->getText())));
    }

    std::any visitRawExpr(JitterishParser::RawExprContext *context) override {
        return context->children.at(0)->accept(this);
    }

    std::any visitBinaryOp(JitterishParser::BinaryOpContext *context) override {
        auto left = context->children.at(0)->accept(this);
        auto op = context->children.at(1)->getText();
        auto right = context->children.at(2)->accept(this);
        return NodeRef(new BinaryOp(op, std::any_cast<NodeRef>(left), std::any_cast<NodeRef>(right)));
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
