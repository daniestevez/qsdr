cmake_minimum_required(VERSION 3.25)

project(gr4-qsdr-benchmark CXX)
set(CMAKE_CXX_STANDARD 23)
set(CMAKE_CXX_VISIBILITY_PRESET hidden)
set(CMAKE_VISIBILITY_INLINES_HIDDEN 1)

option(ADDRESS_SANITIZER "Enable address sanitizer" OFF)
option(UB_SANITIZER "Enable undefined behavior sanitizer" OFF)
option(THREAD_SANITIZER "Enable thread sanitizer" OFF)

# Make sure our local CMake Modules path comes first
list(INSERT CMAKE_MODULE_PATH 0 ${PROJECT_SOURCE_DIR}/cmake/Modules)

set(CMAKE_EXT_DEP_WARNING_GUARD "")
if(DISABLE_EXTERNAL_DEPS_WARNINGS) # enable warnings for external dependencies
    set(CMAKE_EXT_DEP_WARNING_GUARD SYSTEM)
endif()

if (CMAKE_CXX_COMPILER_ID MATCHES "(Clang|GNU|Intel)")
    # -Og is a much more reasonable default for debugging. Also enable gdb extensions.
    set(CMAKE_CXX_FLAGS_DEBUG "-Og -ggdb" CACHE INTERNAL
        "Flags used by the compiler during debug builds.")

    # Add a build type that keeps runtime checks enabled
    set(CMAKE_CXX_FLAGS_RELWITHASSERT "-O3" CACHE INTERNAL
        "Flags used by the compiler during release builds containing runtime checks.")

    # The default value is often an empty string, but this is usually not desirable and one of the
    # other standard build types is usually more appropriate.
    if(NOT CMAKE_BUILD_TYPE)
        set(CMAKE_BUILD_TYPE "RelWithAssert" CACHE STRING
            "Choose the type of build. Options are: None Debug Release RelWithAssert RelWithDebInfo MinSizeRel.\n\
 - None: no compiler flags, defaults and target-specific flags apply\n\
 - Debug: best/complete debugging experience; as optimized as reasonable\n\
 - Release: full optimization; some runtime checks disabled\n\
 - RelWithAssert: full optimization; runtime checks enabled\n\
 - RelWithDebInfo: optimized; debug info; some runtime checks disabled"
            FORCE)
    endif(NOT CMAKE_BUILD_TYPE)

    if (CMAKE_BUILD_TYPE STREQUAL "" AND NOT CMAKE_CXX_FLAGS MATCHES "-O[123gs]")
        message(WARNING "It seems you are compiling without optimization. Please set CMAKE_BUILD_TYPE or CMAKE_CXX_FLAGS.")
    endif ()
endif ()

# Mainly for FMT
set(CMAKE_POSITION_INDEPENDENT_CODE TRUE)

include(FetchContent)

FetchContent_Declare(
    gnuradio4
    GIT_REPOSITORY https://github.com/daniestevez/gnuradio4.git
    GIT_TAG b6729cdfa029465bddd5496f1f93451d98fbd91f # cortex-a53 branch
)

FetchContent_MakeAvailable(gnuradio4)

add_executable(gr4-qsdr-benchmark src/gr4_qsdr_benchmark.cpp)
target_link_libraries(gr4-qsdr-benchmark PRIVATE gnuradio-core fmt::fmt)
