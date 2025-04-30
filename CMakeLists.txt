cmake_minimum_required(VERSION 3.10)
project(CPP2048Player)

# C++ standard and warnings
set(CMAKE_CXX_STANDARD 23)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -Wall -Wextra -g")

find_package(Qt6 REQUIRED COMPONENTS Widgets Core Gui)

# Add include directory
include_directories(include)

# Find all source files in src/
file(GLOB SOURCES "src/*.cpp")

# Build executable
add_executable(cpp_2048 ${SOURCES})

# Link Qt libraries
target_link_libraries(cpp_2048 Qt6::Widgets Qt6::Core Qt6::Gui)
