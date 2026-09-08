cmake_minimum_required(VERSION 3.10)
project(MyProject C)

set(CMAKE_C_STANDARD 99)
set(CMAKE_C_STANDARD_REQUIRED True)

add_executable(myprogram
    packages/sonar-legacy/main.c
    packages/sonar-legacy/library.c
)

target_include_directories(myprogram PRIVATE packages/sonar-legacy)