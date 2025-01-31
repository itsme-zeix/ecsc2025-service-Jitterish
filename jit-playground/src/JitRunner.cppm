module;

#include <dlfcn.h>
#include <cstdio>
#include <string>
#include <iostream>
#include "rtlib/jit_runtime.h"

export module JitRunner;

export class JitRunner {
    void *handle;
public:
    explicit JitRunner(const std::string filename) {
        (void) jit_functions;  // import runtime library
        handle = dlopen(filename.c_str(), RTLD_LAZY);
        fprintf(stderr, "Loaded: %p\n", handle);
    }

    ~JitRunner() {
        if (handle) {
            dlclose(handle);
        }
    }

    Value call(const std::string function) {
        auto ref = dlsym(handle, function.c_str());
        auto fptr = (Value (*)()) ref;
        fprintf(stderr, "%s: %p\n", function.c_str(), fptr);
        auto result = fptr();
        fprintf(stderr, "Jit returned from %s.\n", function.c_str());
        return result;
    }

    static void run(std::string filename) {
        JitRunner r(filename);
        printValue(r.call("jit_entrypoint"));
        std::cout << std::endl;
    }
};
