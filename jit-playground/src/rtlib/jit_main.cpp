#include <iostream>
#include <dlfcn.h>
#include <nlohmann/json.hpp>

#include "jit_values.h"
#include "jit_runtime.h"


int main(int argc, const char* argv[]) {
    if (argc < 2 || argc > 3) {
        std::cerr << "USAGE: " << argv[0] << " <query/function> [<param-as-json>]" << std::endl;
        return 1;
    }

    Value param{};
    if (argc == 3) {
        param = jsonToValue(nlohmann::json::parse(argv[2]));
    }

    auto f = (JitFunction) dlsym(RTLD_DEFAULT, argv[1]);
    if (!f) {
        std::cerr << "\"" << argv[1] << "\" is neither a function nor a query" << std::endl;
        return 1;
    }
    auto result = callJitFunction(f, param);
    printValue(result);
    return 0;
}
