#ifndef JIT_PLAYGROUND_JIT_RUNTIME_H
#define JIT_PLAYGROUND_JIT_RUNTIME_H

#include <nlohmann/json.hpp>
#include "jit_values.h"

Value jsonToValue(const nlohmann::json &j);

#endif //JIT_PLAYGROUND_JIT_RUNTIME_H
