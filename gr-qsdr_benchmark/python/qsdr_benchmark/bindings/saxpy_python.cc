/*
 * Copyright 2024 Free Software Foundation, Inc.
 *
 * This file is part of GNU Radio
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 */

/***********************************************************************************/
/* This file is automatically generated using bindtool and can be manually edited  */
/* The following lines can be configured to regenerate this file during cmake      */
/* If manual edits are made, the following tags should be modified accordingly.    */
/* BINDTOOL_GEN_AUTOMATIC(0)                                                       */
/* BINDTOOL_USE_PYGCCXML(0)                                                        */
/* BINDTOOL_HEADER_FILE(saxpy.h)                                        */
/* BINDTOOL_HEADER_FILE_HASH(9dcfbe635f9cb036f98f7d367b6c6319)                     */
/***********************************************************************************/

#include <pybind11/complex.h>
#include <pybind11/pybind11.h>
#include <pybind11/stl.h>

namespace py = pybind11;

#include <gnuradio/qsdr_benchmark/saxpy.h>
// pydoc.h is automatically generated in the build directory
#include <saxpy_pydoc.h>

void bind_saxpy(py::module& m)
{

    using saxpy = ::gr::qsdr_benchmark::saxpy;


    py::class_<saxpy, gr::sync_block, gr::block, gr::basic_block, std::shared_ptr<saxpy>>(
        m, "saxpy", D(saxpy))

        .def(py::init(&saxpy::make), py::arg("a"), py::arg("b"), D(saxpy, make))


        ;
}