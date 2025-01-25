module;

#include <iostream>
import AsmWriter;

export module CodegenContext;

export class CodegenContext {
public:
    AsmWriter writer = AsmWriter(std::cout);

};
