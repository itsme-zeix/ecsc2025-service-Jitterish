#ifndef JIT_PLAYGROUND_JIT_RUNTIME_H
#define JIT_PLAYGROUND_JIT_RUNTIME_H

#include <cstdint>

#define DATA_MASK 0x00ffffffffffffff
#define TYPE_MASK 0xff00000000000000

enum class ValueType : uint64_t {
    UNDEFINED = 0,
    INT = 1ul << 56,
    STRING = 2ul << 56,
    ARRAY = 3ul << 56,
    OBJECT = 4ul << 56,
    // ... bool? float? null?
};

struct Value {
private:
    uint64_t container; // 8bit type + 56bit data

public:
    Value() : container(0) {}  // "undefined"

    template<class T>
    Value(ValueType type, T data) : container(((uint64_t) type) | (DATA_MASK & (uint64_t) data)) {}

    uint64_t raw() const { return container; }

    ValueType type() const {
        return (ValueType) (TYPE_MASK & container);
    }

    bool is(ValueType t) const { return type() == t; }

    template<class T>
    T data() const {
        return (T) (DATA_MASK & container);
    }

    uint64_t integer() const { return data<uint64_t>(); }

};


#define API_FUNCTION extern "C"

API_FUNCTION
Value __print__(Value v);

API_FUNCTION
Value __add__(Value v1, Value v2);

#endif //JIT_PLAYGROUND_JIT_RUNTIME_H
