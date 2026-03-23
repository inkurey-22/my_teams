cmake_minimum_required(VERSION 3.20)

project(my_teams VERSION 1.0.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 23)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

# Place executable targets at the repository root.
set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${CMAKE_SOURCE_DIR})
foreach(config Debug Release RelWithDebInfo MinSizeRel)
    string(TOUPPER ${config} config_upper)
    set(CMAKE_RUNTIME_OUTPUT_DIRECTORY_${config_upper} ${CMAKE_SOURCE_DIR})
endforeach()

# Shared library location for dynamic linking.
set(MYTEAMS_SHARED_LIB ${CMAKE_SOURCE_DIR}/libs/myteams/libmyteams.so)
set(MYTEAMS_SHARED_LIB_DIR ${CMAKE_SOURCE_DIR}/libs/myteams)

add_executable(myteams_server
    server/main.cpp
)

add_executable(myteams_cli
    client/main.cpp
)

# Declare the prebuilt shared object as an imported target.
add_library(myteams_shared SHARED IMPORTED GLOBAL)
set_target_properties(myteams_shared PROPERTIES
    IMPORTED_LOCATION ${MYTEAMS_SHARED_LIB}
)

target_link_libraries(myteams_server PRIVATE myteams_shared)
target_link_libraries(myteams_cli PRIVATE myteams_shared)

# Set build-time RPATH so executables can find libmyteams.so at runtime.
set_target_properties(myteams_server myteams_cli PROPERTIES
    BUILD_RPATH ${MYTEAMS_SHARED_LIB_DIR}
)
