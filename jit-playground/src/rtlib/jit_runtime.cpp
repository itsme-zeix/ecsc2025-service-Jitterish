#include "jit_runtime.h"
#include <iostream>
#include <nlohmann/json.hpp>

using namespace nlohmann;

#define API_FUNCTION __attribute__((visibility ("default"))) __attribute__((used))

extern "C" {
API_FUNCTION Value jit_rt_print(Value v);
API_FUNCTION Value jit_rt_constant(const char *data, size_t len);

API_FUNCTION Value jit_rt_add(Value v1, Value v2);
API_FUNCTION Value jit_rt_sub(Value v1, Value v2);
API_FUNCTION Value jit_rt_mul(Value v1, Value v2);
API_FUNCTION Value jit_rt_div(Value v1, Value v2);
API_FUNCTION Value jit_rt_not(Value v);
API_FUNCTION Value jit_rt_equal(Value v1, Value v2);
API_FUNCTION Value jit_rt_lt(Value v1, Value v2);
API_FUNCTION Value jit_rt_getproperty(Value v1, Value v2);
API_FUNCTION Value jit_rt_setproperty(Value object, Value key, Value v);
API_FUNCTION bool jit_rt_bool(Value v);
}


void printValue(Value v) {
    switch (v.type()) {
        case ValueType::UNDEFINED:
            std::cout << "undefined";
            break;
        case ValueType::NULLVALUE:
            std::cout << "null";
            break;
        case ValueType::BOOL:
            std::cout << (v.data<bool>() ? "true" : "false");
            break;
        case ValueType::INT:
            std::cout << v.integer();
            break;
        case ValueType::STRING:
            std::cout << '"' << v.str() << '"'; // TODO escaping
            break;
        case ValueType::ARRAY: {
            char c = '[';
            for (auto v2: *v.data<ArrayData *>()) {
                std::cout << c;
                printValue(v2);
                c = ',';
            }
            std::cout << ']';
            break;
        }
        case ValueType::OBJECT: {
            char c = '{';
            for (auto kv: *v.data<ObjectData *>()) {
                std::cout << c << '"' << kv.first << "\":";  // TODO escaping
                printValue(kv.second);
                c = ',';
            }
            std::cout << '}';
            break;
        }
        default:
            std::cout << "<not implemented>";
            break;
    }
}

Value jit_rt_print(Value v) {
    printValue(v);
    std::cout << "\n";
    return {};
}

Value json_to_value(const json &j) {
    if (j.is_null()) return {ValueType::NULLVALUE, 0};
    if (j.is_boolean()) return {ValueType::BOOL, j.template get<bool>()};
    if (j.is_number()) return {ValueType::INT, j.template get<uint64_t>()};
    if (j.is_string()) return {ValueType::STRING, new StringData(j.template get<std::string>())};
    if (j.is_array()) {
        auto data = new ArrayData();
        for (auto item: j) {
            data->push_back(json_to_value(item));
        }
        return {ValueType::ARRAY, data};
    }
    if (j.is_object()) {
        auto data = new ObjectData();
        for (auto item: j.items()) {
            (*data)[item.key()] = json_to_value(item.value());
        }
        return {ValueType::OBJECT, data};
    }
    return {};
}

Value jit_rt_constant(const char *data, size_t len) {
    auto result = json::parse(std::string(data, len));
    return json_to_value(result);
}

Value jit_rt_add(Value v1, Value v2) {
    if (v1.is(ValueType::INT) && v2.is(ValueType::INT)) {
        return {ValueType::INT, v1.integer() + v2.integer()};
    }
    if (v1.is(ValueType::STRING) || v2.is(ValueType::STRING)) {
        return {ValueType::STRING, new std::string(v1.str() + v2.str())};
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

Value jit_rt_not(Value v) {
    return {ValueType::BOOL, !v.is_true()};
}

Value jit_rt_equal(Value v1, Value v2) {
    if (v1.type() != v2.type())
        return {ValueType::BOOL, false};
    if (v1.type() == ValueType::STRING)
        return {ValueType::BOOL, v1.str() == v2.str()};
    return {ValueType::BOOL, v1.data<uintptr_t>() == v2.data<uintptr_t>()};
}

Value jit_rt_lt(Value v1, Value v2) {
    if (v1.is(ValueType::INT) && v2.is(ValueType::INT)) {
        return {ValueType::BOOL, v1.integer() < v2.integer()};
    }
    return {};
}

Value jit_rt_getproperty(Value v1, Value v2) {
    if (v1.is(ValueType::STRING) && v2.is(ValueType::INT)) {
        return {ValueType::INT, v1.data<StringData *>()->at(v2.integer())};
    }
    if (v1.is(ValueType::ARRAY) && v2.is(ValueType::INT)) {
        return v1.data<ArrayData *>()->at(v2.integer());
    }
    if (v1.is(ValueType::OBJECT)) {
        auto data = v1.data<ObjectData *>();
        auto it = data->find(v2.str());
        if (it != data->end()) {
            return it->second;
        }
    }
    return {};
}

Value jit_rt_setproperty(Value object, Value key, Value v) {
    if (object.is(ValueType::STRING) && key.is(ValueType::INT)) {
        key.data<StringData *>()->at(key.integer()) = v.integer();
    } else if (object.is(ValueType::ARRAY) && key.is(ValueType::INT)) {
        key.data<ArrayData *>()->at(key.integer()) = v;
    } else if (object.is(ValueType::OBJECT)) {
        (*object.data<ObjectData *>())[key.str()] = v;
    }
    return v;
}

bool jit_rt_bool(Value v) {
    return v.is_true();
}

void *jit_functions[] = {
        (void *) jit_rt_constant,
        (void *) jit_rt_add,
        (void *) jit_rt_sub,
        (void *) jit_rt_mul,
        (void *) jit_rt_div,
        (void *) jit_rt_equal,
        (void *) jit_rt_lt,
        (void *) jit_rt_getproperty,
        (void *) jit_rt_setproperty,
        (void *) jit_rt_bool,
        (void *) jit_rt_print
};
