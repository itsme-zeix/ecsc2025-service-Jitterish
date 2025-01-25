#include "jit_runtime.h"
#include <iostream>

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

Value __print__(Value v) {
    print(v);
    std::cout << "\n";
    return {};
}

Value __add__(Value v1, Value v2) {
    if (v1.is(ValueType::INT) && v2.is(ValueType::INT)) {
        return Value(ValueType::INT, v1.integer() + v2.integer());
    }
    return {};
}
