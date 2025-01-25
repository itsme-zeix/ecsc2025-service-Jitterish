cmake_minimum_required(VERSION 3.30)
project(jit_playground)

set(CMAKE_CXX_STANDARD 23)

# TODO not so clean - saw some autodetect in another package
set(Java_JAVA_EXECUTABLE java)
set(ANTLR_JAR_LOCATION ${PROJECT_SOURCE_DIR}/antlr.jar)


# TODO also not clean - we have to download this from cmake (or so)
set(ANTLR_BUILD_SHARED OFF)
add_subdirectory(${PROJECT_SOURCE_DIR}/lib/antlr)


set(ANTLR_GENERATED_SOURCES
		${PROJECT_SOURCE_DIR}/src/generated/JitterishLexer.cpp
		${PROJECT_SOURCE_DIR}/src/generated/JitterishParser.cpp
		${PROJECT_SOURCE_DIR}/src/generated/JitterishBaseListener.cpp
		${PROJECT_SOURCE_DIR}/src/generated/JitterishBaseVisitor.cpp
		${PROJECT_SOURCE_DIR}/src/generated/JitterishListener.cpp
		${PROJECT_SOURCE_DIR}/src/generated/JitterishVisitor.cpp
)

foreach (src_file ${ANTLR_GENERATED_SOURCES})
	set_source_files_properties(${src_file} PROPERTIES GENERATED TRUE)
endforeach (src_file ${ANTLR_GENERATED_SOURCES})

add_custom_target(GenerateParser DEPENDS ${ANTLR_GENERATED_SOURCES})
add_custom_command(OUTPUT ${ANTLR_GENERATED_SOURCES}
   COMMAND
   ${CMAKE_COMMAND} -E make_directory ${PROJECT_SOURCE_DIR}/src/generated/
   COMMAND
   "${Java_JAVA_EXECUTABLE}" -jar ${ANTLR_JAR_LOCATION} -Werror -Dlanguage=Cpp -listener -visitor -o ${PROJECT_SOURCE_DIR}/src/generated/ -package generated_parser ${PROJECT_SOURCE_DIR}/Jitterish.g4
   WORKING_DIRECTORY "${CMAKE_BINARY_DIR}"
   DEPENDS ${PROJECT_SOURCE_DIR}/Jitterish.g4
   )


include_directories(
  ${PROJECT_SOURCE_DIR}/lib/antlr/runtime/src
  ${PROJECT_SOURCE_DIR}/lib/antlr/runtime/src/misc
  ${PROJECT_SOURCE_DIR}/lib/antlr/runtime/src/atn
  ${PROJECT_SOURCE_DIR}/lib/antlr/runtime/src/dfa
  ${PROJECT_SOURCE_DIR}/lib/antlr/runtime/src/tree
  ${PROJECT_SOURCE_DIR}/lib/antlr/runtime/src/support
  )

add_executable(jit_playground src/main.cpp ${ANTLR_GENERATED_SOURCES})

target_sources(jit_playground
		PUBLIC
		FILE_SET CXX_MODULES FILES
		src/parser.cppm
		src/ast.cppm
)


add_dependencies(jit_playground GenerateParser)

target_link_libraries(jit_playground antlr4_static)
