module;

#include <dlfcn.h>
#include <cstdio>
#include "rtlib/jit_runtime.h"
#include <string>

export module JitRunner;

export class JitRunner {
public:
    JitRunner() {
        (void) jit_functions;  // import runtime library
    }

    void run(std::string filename) {
        auto *handle = dlopen(filename.c_str(), RTLD_LAZY);
        fprintf(stderr, "Loaded: %p\n", handle);
        auto ref = dlsym(handle, "jit_entrypoint");
        auto fptr = (void (*)()) ref;
        fprintf(stderr, "Let's go! %p\n", fptr);
        fptr();
    }
};
