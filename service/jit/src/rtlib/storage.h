#ifndef JIT_PLAYGROUND_STORAGE_H
#define JIT_PLAYGROUND_STORAGE_H

#include <fstream>
#include <nlohmann/json.hpp>

using namespace nlohmann;



class DataStorage {
    std::string filename;
    std::ifstream f;
public:
    explicit DataStorage(const std::string &filename) : filename(filename), f(filename) {}

    bool isPublic();

    std::optional<json> nextEntry();
};

#endif //JIT_PLAYGROUND_STORAGE_H
