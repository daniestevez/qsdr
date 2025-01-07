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
/* BINDTOOL_HEADER_FILE(dummy_source.h)                                        */
/* BINDTOOL_HEADER_FILE_HASH(297fae5fe14dceeb58dac1f2053d2ac8)                     */
/***********************************************************************************/

#include <pybind11/complex.h>
#include <pybind11/pybind11.h>
#include <pybind11/stl.h>

namespace py = pybind11;

#include <gnuradio/qsdr_benchmark/dummy_source.h>
// pydoc.h is automatically generated in the build directory
#include <dummy_source_pydoc.h>

void bind_dummy_source(py::module& m)
{

    using dummy_source = ::gr::qsdr_benchmark::dummy_source;


    py::class_<dummy_source,
               gr::sync_block,
               gr::block,
               gr::basic_block,
               std::shared_ptr<dummy_source>>(m, "dummy_source", D(dummy_source))

        .def(py::init(&dummy_source::make), D(dummy_source, make))


        ;
}
