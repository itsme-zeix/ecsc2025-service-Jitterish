#ifndef JIT_PLAYGROUND_STORAGE_H
#define JIT_PLAYGROUND_STORAGE_H

#include <fstream>
#include <iterator>
#include <nlohmann/json.hpp>

using namespace nlohmann;

namespace {
class Line : std::string {
    friend std::istream &operator>>(std::istream &is, Line &line) {
        return std::getline(is, line);
    }
};
}

class DataStorage {
    std::ifstream f;
public:
    explicit DataStorage(const std::string &filename) : f(filename) {}

    bool isPublic() {
        return true;  // TODO implement
    }

    std::optional<json> nextEntry() {
        std::string str;
        if (std::getline(f, str)) {
            return json::parse(str);
        }
        return {};
    }
};

#endif //JIT_PLAYGROUND_STORAGE_H
