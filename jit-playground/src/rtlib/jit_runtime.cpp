#include "jit_values.h"
#include <iostream>
#include <nlohmann/json.hpp>
#include "storage.h"

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
API_FUNCTION Value jit_rt_and(Value v1, Value v2);
API_FUNCTION Value jit_rt_or(Value v1, Value v2);
API_FUNCTION Value jit_rt_default(Value v1, Value v2);
API_FUNCTION Value jit_rt_equal(Value v1, Value v2);
API_FUNCTION Value jit_rt_lt(Value v1, Value v2);
API_FUNCTION Value jit_rt_getproperty(Value v1, Value v2);
API_FUNCTION Value jit_rt_setproperty(Value object, Value key, Value v);
API_FUNCTION Value jit_rt_object_new();
API_FUNCTION Value jit_rt_array_new();
API_FUNCTION Value jit_rt_append(Value v1, Value v2);
API_FUNCTION Value jit_rt_len(Value v);
API_FUNCTION bool jit_rt_bool(Value v);
API_FUNCTION Value jit_rt_query(const char *collection, JitFunction filter, JitFunction map, JitFunction reduce, size_t limit, Value param);
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
            std::cout << json(v.str());
            break;
        case ValueType::ARRAY: {
            std::cout << '[';
            std::string c;
            for (auto v2: *v.data<ArrayData *>()) {
                std::cout << c;
                printValue(v2);
                c = ',';
            }
            std::cout << ']';
            break;
        }
        case ValueType::OBJECT: {
            std::cout << '{';
            std::string c;
            for (const auto &kv: *v.data<ObjectData *>()) {
                std::cout << c << json(kv.first) << ":";
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

Value jsonToValue(const json &j) {
    if (j.is_null()) return {ValueType::NULLVALUE, 0};
    if (j.is_boolean()) return {ValueType::BOOL, j.template get<bool>()};
    if (j.is_number()) return {ValueType::INT, j.template get<uint64_t>()};
    if (j.is_string()) return {ValueType::STRING, new StringData(j.template get<std::string>())};
    if (j.is_array()) {
        auto data = new ArrayData();
        for (const auto &item: j) {
            data->push_back(jsonToValue(item));
        }
        return {ValueType::ARRAY, data};
    }
    if (j.is_object()) {
        auto data = new ObjectData();
        for (const auto &item: j.items()) {
            (*data)[item.key()] = jsonToValue(item.value());
        }
        return {ValueType::OBJECT, data};
    }
    return {};
}

Value jit_rt_constant(const char *data, size_t len) {
    auto result = json::parse(std::string(data, len));
    return jsonToValue(result);
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

Value jit_rt_and(Value v1, Value v2) {
    return {ValueType::BOOL, v1.is_true() && v2.is_true()};
}

Value jit_rt_or(Value v1, Value v2) {
    return {ValueType::BOOL, v1.is_true() || v2.is_true()};
}

Value jit_rt_default(Value v1, Value v2) {
    return v1.is_true() ? v1 : v2;
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
        object.data<StringData *>()->at(key.integer()) = (char) v.integer();
    } else if (object.is(ValueType::ARRAY) && key.is(ValueType::INT)) {
        object.data<ArrayData *>()->at(key.integer()) = v;
    } else if (object.is(ValueType::OBJECT)) {
        (*object.data<ObjectData *>())[key.str()] = v;
    }
    return v;
}

Value jit_rt_object_new() {
    return {ValueType::OBJECT, new ObjectData()};
}

Value jit_rt_array_new() {
    return {ValueType::ARRAY, new ArrayData()};
}

Value jit_rt_append(Value v1, Value v2) {
    if (v1.is(ValueType::ARRAY)) {
        v1.data<ArrayData *>()->push_back(v2);
    }
    if (v1.is(ValueType::STRING)) {
        v1.data<StringData *>()->append(v2.str());
    }
    return v1;
}

Value jit_rt_len(Value v) {
    if (v.is(ValueType::STRING)) {
        return {ValueType::INT, v.data<StringData *>()->size()};
    } else if (v.is(ValueType::ARRAY)) {
        return {ValueType::INT, v.data<ArrayData *>()->size()};
    } else if (v.is(ValueType::OBJECT)) {
        return {ValueType::INT, v.data<ObjectData *>()->size()};
    }
    return {};
}

bool jit_rt_bool(Value v) {
    return v.is_true();
}

Value jit_rt_query(const char *collection, JitFunction filter, JitFunction map, JitFunction reduce, size_t limit, Value param) {
    auto storage = DataStorage(std::string(collection) + ".json");
    if (!storage.isPublic()) {
        throw std::runtime_error("access denied");
    }

    Value result = reduce ? Value {} : jit_rt_array_new();

    while (auto jsonEntry = storage.nextEntry()) {
        Value entry = jsonToValue(*jsonEntry);
        if (filter && !callJitFunction(filter, entry, param).is_true()) {
            continue;
        }
        if (map) {
            entry = callJitFunction(map, entry, param);
        }
        if (reduce) {
            result = callJitFunction(reduce, entry, result, param);
        } else {
            result.data<ArrayData *>()->push_back(entry);
        }
        if ((--limit) == 0)
            break;
    }

    return result;
}

void *jit_functions[] = {
        (void *) jit_rt_constant,
        (void *) jit_rt_add,
        (void *) jit_rt_sub,
        (void *) jit_rt_mul,
        (void *) jit_rt_div,
        (void *) jit_rt_not,
        (void *) jit_rt_and,
        (void *) jit_rt_or,
        (void *) jit_rt_default,
        (void *) jit_rt_equal,
        (void *) jit_rt_lt,
        (void *) jit_rt_getproperty,
        (void *) jit_rt_setproperty,
        (void *) jit_rt_object_new,
        (void *) jit_rt_array_new,
        (void *) jit_rt_append,
        (void *) jit_rt_len,
        (void *) jit_rt_bool,
        (void *) jit_rt_print,
        (void *) jit_rt_query
};
