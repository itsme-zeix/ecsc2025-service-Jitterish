#include <iostream>
#include <iterator>

#include "storage.h"

namespace {
class Line : std::string {
    friend std::istream &operator>>(std::istream &is, Line &line) {
        return std::getline(is, line);
    }
};
}

bool DataStorage::isPublic() {
    return !filename.contains("private");
}

std::optional<json> DataStorage::nextEntry() {
    std::string str;
    while (std::getline(f, str)) {
        try {
            return json::parse(str);
        } catch (const json::parse_error &e) {
            std::cerr << "[WARN] broken data in " << filename << std::endl;
        }
    }
    return {};
}