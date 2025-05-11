## Building:
```
mkdir build
cd build
// this will not work because of outdated tests that import an old cmake version
cmake ../ -G Ninja -DCMAKE_CXX_COMPILER_CLANG_SCAN_DEPS:FILEPATH=/usr/bin/clang-scan-deps -D CMAKE_CXX_COMPILER=/usr/bin/clang++    
// replace the cmake version
find . -name CMakeLists.txt -execdir sed -i "s/cmake_minimum_required(VERSION 2.8.12)/cmake_minimum_required(VERSION 3.5)/g" CMakeLists.txt \;
// now it works
cmake ../ -G Ninja -DCMAKE_CXX_COMPILER_CLANG_SCAN_DEPS:FILEPATH=/usr/bin/clang-scan-deps -D CMAKE_CXX_COMPILER=/usr/bin/clang++    
```

Try out (defines 2 queries against sample dataset and runs them):
`cd demo && ../build/jit_playground ./queries.txt`

Code statistics: `scc -v --exclude-dir generated src`

Tests: `cd tests ; python3 tests_compile.py`
