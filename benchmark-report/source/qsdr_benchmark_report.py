import pathlib

import numpy as np
import matplotlib.pyplot as plt
import matplotlib.ticker as tkr

RESULTS_PATH = (
    pathlib.Path(__file__).parent.parent.parent / 'benchmark-results')


def sizeof_fmt(x, pos=None):
    if x < 0:
        return ""
    for x_unit in ['bytes', 'kiB', 'MiB', 'GiB', 'TiB']:
        if x < 1024:
            return f'{x:.0f} {x_unit}'
        x /= 1024


def parse_bench(path):
    with open(path) as f:
        return np.array([[float(sec.split('=')[1].strip())
                          for sec in line.split(',')]
                         for line in f])


def plot_benchmark_channel():
    results = RESULTS_PATH / 'benchmark_channel'
    vals = {p.stem: parse_bench(p) for p in results.iterdir()}
    channels = {
        'async-channel-bounded': 'async-channel::bounded',
        'crossbeam-bounded': 'crossbeam-channel::bounded',
        'flume-bounded': 'flume::bounded',
        'flume-bounded-async': 'flume::bounded (async rx)',
        'kanal-bounded': 'kanal::bounded',
        'kanal-bounded-async': 'kanal::bounded_async',
        'qsdr-spsc-futex': 'qsdr::channel::spsc::futex',
        'qsdr-mpsc-futex': 'qsdr::channel::mpsc::futex',
        'qsdr-spsc-futures': 'qsdr::channel::spsc::futures',
        'qsdr-mpsc-futures': 'qsdr::channel::mpsc::futures',
        'std-mpsc': 'std::sync::mpsc',
        'tokio': 'tokio::sync::mpsc',
    }
    keys = sorted(vals.keys())
    labels = [channels[k] for k in keys]
    async_ = ['tokio' in label or 'futures' in label
              or 'async' in label for label in labels]
    labs = ['_async' if a else '_sync' for a in async_]
    for k in ['async', 'sync']:
        labs[labs.index(f'_{k}')] = k
    plt.bar(labels, [np.average(vals[k]) for k in keys],
            label=labs,
            color=['C1' if a else 'C0' for a in async_])
    plt.legend()
    plt.xticks(rotation=90)
    plt.ylabel('items/s')
    plt.title('Channel items/s between two threads (higher is beter)\n'
              'item size = 32 bytes, channel size = 64 items')
    plt.grid()


SAXPY_THEORETICAL_MAX = 1.333e9


def plot_benchmark_saxpy_scan_buffer_size():
    results = RESULTS_PATH / 'benchmark_saxpy' / 'scan-buffer-size.txt'
    vals = parse_bench(results)
    plt.figure()
    plt.plot(vals[:, 0], vals[:, 1], 'o-')
    plt.gca().set_xscale('log', base=2)
    plt.gca().xaxis.set_major_formatter(tkr.FuncFormatter(sizeof_fmt))

    plt.ylim(0, SAXPY_THEORETICAL_MAX)
    plt.grid()
    plt.xlabel('Buffer size')
    plt.ylabel('samples/s')


def plot_benchmark_saxpy_single_core():
    results = RESULTS_PATH / 'benchmark_saxpy' / 'single-core'

    def parse_key(k):
        return (int(k.split('_')[0]), int(k.split('_')[1]))

    vals = {parse_key(p.stem): parse_bench(p) for p in results.iterdir()}
    num_buffers = sorted({k[1] for k in vals.keys()})
    plt.figure()
    for n in num_buffers:
        buffer_sizes = sorted({k[0] for k in vals.keys()
                               if k[1] == n})
        sps = [np.average(vals[(b, n)][:, 0]) for b in buffer_sizes]
        plt.plot(np.array(buffer_sizes) * n, sps, 'o-',
                 label=f'{n} buffers')
        plt.gca().set_xscale('log', base=2)
        plt.gca().xaxis.set_major_formatter(tkr.FuncFormatter(sizeof_fmt))

    plt.ylim(0, SAXPY_THEORETICAL_MAX)
    plt.grid()
    plt.xlabel('Combined size of all buffers')
    plt.ylabel('samples/s')
    plt.legend()


def plot_benchmark_saxpy_multi_core(bench):
    results = RESULTS_PATH / 'benchmark_saxpy' / bench

    def parse_key(k):
        return (int(k.split('_')[0]),
                int(k.split('_')[1]),
                int(k.split('_')[2]))

    vals = {parse_key(p.stem): parse_bench(p) for p in results.iterdir()}
    num_cpus = sorted({k[0] for k in vals.keys()})
    for ncpu in num_cpus:
        plt.figure()
        num_buffers = sorted({k[2] for k in vals.keys() if k[0] == ncpu})
        for n in num_buffers:
            buffer_sizes = sorted({k[1] for k in vals.keys()
                                   if k[0] == ncpu and k[2] == n})
            sps = [np.average(vals[(ncpu, b, n)][:, 0]) for b in buffer_sizes]
            plt.plot(buffer_sizes, sps, 'o-', label=f'{n} buffers')

        plt.gca().set_xscale('log', base=2)
        plt.gca().xaxis.set_major_formatter(tkr.FuncFormatter(sizeof_fmt))
        plt.ylim(0, SAXPY_THEORETICAL_MAX)
        plt.grid()
        plt.xlabel('Buffer size')
        plt.ylabel('samples/s')
        plt.legend()
        plt.title(f'{ncpu} CPU cores')


def plot_benchmark_saxpy_multi_kernel(bench):
    results = RESULTS_PATH / 'benchmark_saxpy' / bench

    def parse_key(k):
        return (int(k.split('_')[0]),
                int(k.split('_')[1]),
                int(k.split('_')[2]),
                int(k.split('_')[3]))

    vals = {parse_key(p.stem): parse_bench(p) for p in results.iterdir()}
    num_cpus = sorted({k[0] for k in vals.keys()})
    for ncpu in num_cpus:
        num_kernels = sorted({k[1] for k in vals.keys() if k[0] == ncpu})
        for nker in num_kernels:
            plt.figure()
            num_buffers = sorted({k[3] for k in vals.keys()
                                  if k[0] == ncpu and k[1] == nker})
            for n in num_buffers:
                buffer_sizes = sorted(
                    {k[2] for k in vals.keys()
                     if k[0] == ncpu and k[1] == nker and k[3] == n})
                sps = [np.average(vals[(ncpu, nker, b, n)][:, 0])
                       for b in buffer_sizes]
                plt.plot(buffer_sizes, sps, 'o-', label=f'{n} buffers')

            plt.gca().set_xscale('log', base=2)
            plt.gca().xaxis.set_major_formatter(tkr.FuncFormatter(sizeof_fmt))
            plt.ylim(0, SAXPY_THEORETICAL_MAX / ((nker + ncpu - 1) // ncpu))
            plt.grid()
            plt.xlabel('Buffer size')
            plt.ylabel('samples/s')
            plt.legend()
            plt.title(f'{ncpu} CPU cores, {nker} kernels')


def plot_benchmark_saxpy_multi_kernel_fixed_buffers(bench, buffers_for_ncpu):
    results = RESULTS_PATH / 'benchmark_saxpy' / bench

    def parse_key(k):
        return (int(k.split('_')[0]),
                int(k.split('_')[1]),
                int(k.split('_')[2]),
                int(k.split('_')[3]))

    vals = {parse_key(p.stem): parse_bench(p) for p in results.iterdir()}
    num_cpus = sorted({k[0] for k in vals.keys()})
    for ncpu in num_cpus:
        num_buffers = buffers_for_ncpu(ncpu)
        num_kernels = np.array(sorted(
            {k[1] for k in vals.keys()
             if k[0] == ncpu if k[3] == num_buffers}))
        buffer_sizes = sorted({k[2] for k in vals.keys()
                               if k[0] == ncpu and k[3] == num_buffers})
        plt.figure()
        plt.plot(num_kernels,
                 SAXPY_THEORETICAL_MAX / ((num_kernels + ncpu - 1) // ncpu),
                 '.:', color='grey')
        for bs in buffer_sizes:
            sps = [np.average(vals[(ncpu, n, bs, num_buffers)][:, 0])
                   for n in num_kernels]
            plt.plot(num_kernels, sps, 'o-',
                     label=f'Buffer size {sizeof_fmt(bs)}')

        plt.gca().xaxis.set_major_locator(tkr.MaxNLocator(integer=True))
        plt.ylim(0, SAXPY_THEORETICAL_MAX)
        plt.grid()
        plt.xlabel('Number of kernels')
        plt.ylabel('samples/s')
        plt.legend()
        plt.title(f'{ncpu} CPU cores, {num_buffers} buffers')


def plot_runtimes_single_core():
    results = {
        'FutureSDR': RESULTS_PATH / 'futuresdr-benchmark' / 'single-core.txt',
        'GNU Radio 3.10': (RESULTS_PATH / 'gr_qsdr_benchmark' /
                           'affinities' / '1_1.txt'),
        'GNU Radio 4.0': (RESULTS_PATH / 'gr4-qsdr-benchmark' /
                          'single-core.txt'),
        'qsdr': RESULTS_PATH / 'benchmark_qsdr' / 'single-core.txt',
    }
    vals = {k: parse_bench(r) for k, r in results.items()}
    keys = sorted(vals.keys())
    plt.figure()
    plt.bar(keys, [np.average(vals[k]) for k in keys])
    plt.ylabel('samples/s')
    plt.ylim(0, SAXPY_THEORETICAL_MAX)
    plt.grid()


def plot_runtimes_multi_kernel():
    results = {
        'FutureSDR (custom sched)': (
            RESULTS_PATH / 'futuresdr-benchmark' / 'multi-kernel'),
        'FutureSDR (smol sched)': (
            RESULTS_PATH / 'futuresdr-benchmark' / 'multi-kernel-smol'),
        'GR 3.10 (thread affinities)': (
            RESULTS_PATH / 'gr_qsdr_benchmark' / 'affinities'),
        'GR 3.10 (no thread affinities)': (
            RESULTS_PATH / 'gr_qsdr_benchmark' / 'no-affinities'),
        'GR 4.0 (custom sched)': (
            RESULTS_PATH / 'gr4-qsdr-benchmark' / 'multi-kernel'),
        'GR 4.0 (simple sched)': (
            RESULTS_PATH / 'gr4-qsdr-benchmark' / 'multi-kernel-simple'),
        'qsdr (custom sched)': (
            RESULTS_PATH / 'benchmark_qsdr' / 'multi-kernel'),
        'qsdr (async-executor sched)': (
            RESULTS_PATH / 'benchmark_qsdr' / 'multi-kernel-async-executor'),
        'qsdr (tokio sched)': (
            RESULTS_PATH / 'benchmark_qsdr' / 'multi-kernel-tokio'),

    }
    for ncpu in range(1, 5):
        plt.figure()
        nk = np.arange(ncpu, 3*ncpu + 1)
        plt.plot(nk, SAXPY_THEORETICAL_MAX * ncpu / nk, '.--', color='grey')
        plt.plot(nk, SAXPY_THEORETICAL_MAX / ((nk + ncpu - 1) // ncpu),
                 '.:', color='grey')
        for runtime in results:

            def parse_key(k):
                return (int(k.split('_')[0]), int(k.split('_')[1]))

            vals = {parse_key(p.stem): parse_bench(p)
                    for p in results[runtime].iterdir()}
            nker = sorted({k[1] for k in vals.keys() if k[0] == ncpu})
            sps = [np.average(vals[(ncpu, n)]) for n in nker]
            plt.plot(nker, sps, 'o-', label=runtime)

        if ncpu >= 2:
            res = RESULTS_PATH / 'benchmark_saxpy' / 'multi-kernel-async'
            num_buffers = ncpu + 1
            buffer_size = 16384

            def parse_key(k):
                return (int(k.split('_')[0]),
                        int(k.split('_')[1]),
                        int(k.split('_')[2]),
                        int(k.split('_')[3]))

            vals = {parse_key(p.stem): parse_bench(p) for p in res.iterdir()}
            nker = sorted({k[1] for k in vals.keys()
                           if k[0] == ncpu and k[2] == buffer_size
                           and k[3] == num_buffers})
            sps = [np.average(vals[(ncpu, nk, buffer_size, num_buffers)][:, 0])
                   for nk in nker]
            plt.plot(nker, sps, 'o-', label=f'multi-kernel-async')

        plt.gca().xaxis.set_major_locator(tkr.MaxNLocator(integer=True))
        plt.ylim(0, SAXPY_THEORETICAL_MAX)
        plt.grid()
        plt.xlabel('Number of kernels')
        plt.ylabel('samples/s')
        plt.legend(loc=[1.02, 0.2])
        plt.title(f'{ncpu} CPU cores')
