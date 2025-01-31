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
    inline Value() : container(0) {}  // "undefined"

    template<class T>
    inline Value(ValueType type, T data) : container(((uint64_t) type) | (DATA_MASK & (uint64_t) data)) {}

    inline uint64_t raw() const { return container; }

    inline ValueType type() const {
        return (ValueType) (TYPE_MASK & container);
    }

    inline bool is(ValueType t) const { return type() == t; }

    template<class T>
    inline T data() const {
        return (T) (DATA_MASK & container);
    }

    inline uint64_t integer() const { return data<uint64_t>(); }

};

void printValue(Value v);

extern void* jit_functions[];

#endif //JIT_PLAYGROUND_JIT_RUNTIME_H
