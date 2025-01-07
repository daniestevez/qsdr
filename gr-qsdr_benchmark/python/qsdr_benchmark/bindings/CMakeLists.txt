# Copyright 2020 Free Software Foundation, Inc.
#
# This file is part of GNU Radio
#
# SPDX-License-Identifier: GPL-3.0-or-later
#

########################################################################
# Check if there is C++ code at all
########################################################################
if(NOT qsdr_benchmark_sources)
    message(STATUS "No C++ sources... skipping python bindings")
    return()
endif(NOT qsdr_benchmark_sources)

########################################################################
# Check for pygccxml
########################################################################
gr_python_check_module_raw("pygccxml" "import pygccxml" PYGCCXML_FOUND)

include(GrPybind)

########################################################################
# Python Bindings
########################################################################

list(APPEND qsdr_benchmark_python_files
    dummy_source_python.cc
    saxpy_python.cc
    benchmark_sink_python.cc python_bindings.cc)

gr_pybind_make_oot(qsdr_benchmark ../../.. gr::qsdr_benchmark "${qsdr_benchmark_python_files}")

# copy bindings extension for use in QA test module
add_custom_command(
    TARGET qsdr_benchmark_python
    POST_BUILD
    COMMAND ${CMAKE_COMMAND} -E copy $<TARGET_FILE:qsdr_benchmark_python>
            ${PROJECT_BINARY_DIR}/test_modules/gnuradio/qsdr_benchmark/)

install(
    TARGETS qsdr_benchmark_python
    DESTINATION ${GR_PYTHON_DIR}/gnuradio/qsdr_benchmark
    COMPONENT pythonapi)
