find_package(PkgConfig)

PKG_CHECK_MODULES(PC_GR_QSDR_BENCHMARK gnuradio-qsdr_benchmark)

FIND_PATH(
    GR_QSDR_BENCHMARK_INCLUDE_DIRS
    NAMES gnuradio/qsdr_benchmark/api.h
    HINTS $ENV{QSDR_BENCHMARK_DIR}/include
        ${PC_QSDR_BENCHMARK_INCLUDEDIR}
    PATHS ${CMAKE_INSTALL_PREFIX}/include
          /usr/local/include
          /usr/include
)

FIND_LIBRARY(
    GR_QSDR_BENCHMARK_LIBRARIES
    NAMES gnuradio-qsdr_benchmark
    HINTS $ENV{QSDR_BENCHMARK_DIR}/lib
        ${PC_QSDR_BENCHMARK_LIBDIR}
    PATHS ${CMAKE_INSTALL_PREFIX}/lib
          ${CMAKE_INSTALL_PREFIX}/lib64
          /usr/local/lib
          /usr/local/lib64
          /usr/lib
          /usr/lib64
          )

include("${CMAKE_CURRENT_LIST_DIR}/gnuradio-qsdr_benchmarkTarget.cmake")

INCLUDE(FindPackageHandleStandardArgs)
FIND_PACKAGE_HANDLE_STANDARD_ARGS(GR_QSDR_BENCHMARK DEFAULT_MSG GR_QSDR_BENCHMARK_LIBRARIES GR_QSDR_BENCHMARK_INCLUDE_DIRS)
MARK_AS_ADVANCED(GR_QSDR_BENCHMARK_LIBRARIES GR_QSDR_BENCHMARK_INCLUDE_DIRS)
