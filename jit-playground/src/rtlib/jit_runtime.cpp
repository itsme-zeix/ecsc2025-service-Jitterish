#include "jit_runtime.h"
#include <iostream>

#define API_FUNCTION __attribute__((visibility ("default"))) __attribute__((used))

extern "C" {
API_FUNCTION Value jit_rt_print(Value v);

API_FUNCTION Value jit_rt_add(Value v1, Value v2);
API_FUNCTION Value jit_rt_sub(Value v1, Value v2);
API_FUNCTION Value jit_rt_mul(Value v1, Value v2);
API_FUNCTION Value jit_rt_div(Value v1, Value v2);
}


static void print(Value v) {
    switch (v.type()) {
        case ValueType::UNDEFINED:
            std::cout << "undefined";
            break;
        case ValueType::INT:
            std::cout << v.integer();
            break;
        default:
            std::cout << "<not implemented>";
            break;
    }
}

Value jit_rt_print(Value v) {
    print(v);
    std::cout << "\n";
    return {};
}

Value jit_rt_add(Value v1, Value v2) {
    if (v1.is(ValueType::INT) && v2.is(ValueType::INT)) {
        return {ValueType::INT, v1.integer() + v2.integer()};
    }
    return {};
}

Value jit_rt_sub(Value v1, Value v2) {
    if (v1.is(ValueType::INT) && v2.is(ValueType::INT)) {
        return {ValueType::INT, v1.integer() - v2.integer()};
    }
    return {};
}

Value jit_rt_mul(Value v1, Value v2) {
    if (v1.is(ValueType::INT) && v2.is(ValueType::INT)) {
        return {ValueType::INT, v1.integer() * v2.integer()};
    }
    return {};
}

Value jit_rt_div(Value v1, Value v2) {
    if (v1.is(ValueType::INT) && v2.is(ValueType::INT)) {
        return {ValueType::INT, v1.integer() / v2.integer()};
    }
    return {};
}

void *jit_functions[] = {
        (void *) jit_rt_add,
        (void *) jit_rt_sub,
        (void *) jit_rt_mul,
        (void *) jit_rt_div,
        (void *) jit_rt_print
};
