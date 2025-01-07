Rust channels
=============

This shows the results of a benchmark of the performance of different
implementations of bounded channels in Rust, using a single producer and single
consumer. The benchmark is implemented by ``benchmark_channels`` in the
``qsdr-benchmarks`` crate.

The benchmark is done as follows. There are two threads pinned to different CPU
cores, and two channels of the same type connecting the threads. Each thread has
the transmitter for one of the channels and the receiver for the other
channel. The size of the channels is fixed to 64 items of size 32 bytes. Before
the benchmark begins, 64 items are transmitted into one of the channels. During
the benchmark, each thread receives an item from the other thread on one
channel, and immediately sends it back to the other thread using the other
channel. Additionally, one of the two threads measures the elapsed time every
1e7 processed items, in order to determine the rate at which items are
transferred.

For the async channels, each of the threads runs an async block in a separate
``block_on`` executor from the ``qsdr-benchmarks`` crate. This is a very simple
executor that tries to run a single future to completion on the current
thread. It is based on the ``block_on`` executor from ``futures-executor``. If
the future is not ready yet, it spins by polling the future multiple times (1024
times by default) until it decides to parks the thread. The waker passed to the
future's poll unparks the thread if it has not already been unparked (as
determined by the value of an atomic boolean that is modified before parking or
unparking). The spinning before parking is done to minimize the overhead caused
by calling the kernel to park and unpark the thread. For good performance, the
threads should get parked infrequently during the benchmark.

.. plot::

   import qsdr_benchmark_report
   qsdr_benchmark_report.plot_benchmark_channel()
