# CMakeLists.txt for GeoDESK Rust bindings
# Automatically fetches and builds GeoDESK using CMake FetchContent

cmake_minimum_required(VERSION 3.14)
project(geodesk_bridge CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_POSITION_INDEPENDENT_CODE ON)

# Automatically fetch GeoDESK using FetchContent as per official documentation
include(FetchContent)

message(STATUS "Fetching GeoDESK library...")

# Disable building tests and examples for GeoDESK
set(BUILD_TESTING OFF CACHE BOOL "Disable tests" FORCE)
set(GEODESK_BUILD_TESTS OFF CACHE BOOL "Disable GeoDESK tests" FORCE)
set(GEODESK_BUILD_EXAMPLES OFF CACHE BOOL "Disable GeoDESK examples" FORCE)

FetchContent_Declare(geodesk
    GIT_REPOSITORY https://github.com/clarisma/libgeodesk.git
    GIT_TAG 0e4baedf470496f90440a99e2a1cdaf4a17f04eb  # v2.1.0 - pinned due to destructor bug
    PATCH_COMMAND ""
)

# Build GeoDESK as a static library
set(BUILD_SHARED_LIBS OFF CACHE BOOL "Build as static library" FORCE)

# Fetch the content first
FetchContent_GetProperties(geodesk)
if(NOT geodesk_POPULATED)
    FetchContent_Populate(geodesk)

    # Apply the patch to fix destructor call bug
    message(STATUS "Applying patch to fix FeatureIteratorBase destructor...")
    execute_process(
        COMMAND sed -i "s/\\.~WayNodeCursor()()/\\.~WayNodeCursor()/g"
                "${geodesk_SOURCE_DIR}/src/feature/FeatureIteratorBase.cpp"
        RESULT_VARIABLE PATCH_RESULT
    )
    if(PATCH_RESULT EQUAL 0)
        message(STATUS "Patch applied successfully")
    else()
        message(WARNING "Failed to apply patch, but continuing anyway")
    endif()

    # Now add the subdirectory
    add_subdirectory(${geodesk_SOURCE_DIR} ${geodesk_BINARY_DIR})
endif()

message(STATUS "GeoDESK fetched and configured successfully")

# Note: The bridge library (geodesk_bridge) is built by cxx-build in build.rs
# We only build the GeoDESK library here via CMake
# The bridge needs access to cxx.rs generated headers which are not available at CMake time

