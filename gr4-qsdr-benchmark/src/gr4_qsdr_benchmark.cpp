/* -*- c++ -*- */
/*
 * Copyright 2024 Daniel Estevez <daniel@destevez.net>
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

#include <fmt/core.h>
#include <gnuradio-4.0/Block.hpp>
#include <gnuradio-4.0/Graph.hpp>
#include <gnuradio-4.0/Scheduler.hpp>
#include <algorithm>
#include <cstdint>
#include <exception>
#include <mutex>
#include <random>
#include <ranges>
#include <string>

// This #ifdef works with gcc and clang, but not with other compilers
#ifdef __aarch64__
#include <arm_neon.h>
#endif

template <typename T>
class DummySource : public gr::Block<DummySource<T>>
{
public:
    gr::PortOut<T> out;

    GR_MAKE_REFLECTABLE(DummySource, out);

    [[nodiscard]] constexpr gr::work::Status processBulk(gr::OutputSpanLike auto&)
    {
        // publish is called automatically by the runtime
        return gr::work::Status::OK;
    }
};

template <typename T>
class BenchmarkSink : public gr::Block<BenchmarkSink<T>>
{
public:
    gr::PortIn<T> in;
    uint64_t _count{ 0 };
    using ClockType = std::chrono::steady_clock;
    std::chrono::time_point<ClockType> _time;
    static constexpr uint64_t _measure_every = 1UL << 30;

    GR_MAKE_REFLECTABLE(BenchmarkSink, in);

    void start() { _time = ClockType::now(); }

    [[nodiscard]] gr::work::Status processBulk(gr::InputSpanLike auto& inSpan)
    {
        _count += inSpan.size();
        if (_count >= _measure_every) {
            const auto now = ClockType::now();
            const double elapsed = std::chrono::duration<double>(now - _time).count();
            const double samples_per_sec = static_cast<double>(_count) / elapsed;
            fmt::println("samples/s = {:.3e}", samples_per_sec);
            _count = 0;
            _time = now;
        }
        // consume is called automatically by the runtime
        return gr::work::Status::OK;
    }
};

class Saxpy : public gr::Block<Saxpy>
{
public:
#ifdef __aarch64__
    // the NEON kernel requires at least 64 floats (and a multiple of 32 floats)
    gr::PortIn<float, gr::RequiredSamples<64UZ>> in;
    gr::PortOut<float, gr::RequiredSamples<64UZ>> out;
#else
    gr::PortIn<float> in;
    gr::PortOut<float> out;
#endif
    float a;
    float b;

    GR_MAKE_REFLECTABLE(Saxpy, in, out, a, b);

#ifdef __aarch64__
    [[nodiscard]] gr::work::Status processBulk(gr::InputSpanLike auto& inSpan,
                                               gr::OutputSpanLike auto& outSpan)
    {
        static constexpr size_t floats_per_iter = 32;
        if (inSpan.size() % floats_per_iter != 0) {
            std::terminate();
        }
        const size_t iterations = inSpan.size() / floats_per_iter - 1;
        const float* buff_in0 = &inSpan[0];
        const float* buff_in1 = &inSpan[floats_per_iter / 2];
        const float* buff_in0_end = &inSpan[floats_per_iter * iterations];
        float* buff_out = &outSpan[0];
        uint64_t scratch0;
        uint64_t scratch1;
        uint64_t scratch2;
        uint64_t scratch3;
        uint64_t scratch4;
        // manually allocate these to v31 and v30 because gcc doesn't understand
        // that v0-v7 are overwritten and cannot be used to hold these values
        register float s31 __asm__("s31") = a;
        register float32x4_t v30 __asm__("v30") = vdupq_n_f32(b);
        __asm__ volatile(
            "ld1 {v4.4s-v7.4s}, [%[buff_in0]]\n\t"
            "fmul v4.4s, v4.4s, %[vA].s[0]\n\t"
            "prfm PLDL1KEEP, [%[buff_in0], #128]\n\t"
            "fmul v5.4s, v5.4s, %[vA].s[0]\n\t"
            "ldr %[scratch0], [%[buff_in0], #72]\n\t"
            "fmul v6.4s, v6.4s, %[vA].s[0]\n\t"
            "ldr %[scratch1], [%[buff_in0], #88]\n\t"
            "fmul v7.4s, v7.4s, %[vA].s[0]\n\t"
            "ldr %[scratch2], [%[buff_in0], #104]\n\t"
            "fadd v4.4s, v4.4s, %[vB].4s\n\t"
            "ldr %[scratch3], [%[buff_in0], #120]\n\t"
            "fadd v5.4s, v5.4s, %[vB].4s\n\t"
            "ldr %[scratch4], [%[buff_in0], #112]\n\t"
            "fadd v6.4s, v6.4s, %[vB].4s\n\t"
            "prfm PLDL1KEEP, [%[buff_in1], #128]\n\t"
            "ldr d0, [%[buff_in0], #64]\n\t"
            "ins v3.d[1], %[scratch3]\n\t"
            "ldr d1, [%[buff_in0], #80]\n\t"
            "ins v0.d[1], %[scratch0]\n\t"
            "ldr d2, [%[buff_in0], #96]\n\t"
            "ins v1.d[1], %[scratch1]\n\t"
            "ins v3.d[0], %[scratch4]\n\t"
            "ins v2.d[1], %[scratch2]\n\t"
            "fadd v7.4s, v7.4s, %[vB].4s\n\t"
            "0:\n\t"
            "fmul v0.4s, v0.4s, %[vA].s[0]\n\t"
            "ldr %[scratch0], [%[buff_in0], #136]\n\t"
            "fmul v1.4s, v1.4s, %[vA].s[0]\n\t"
            "fmul v2.4s, v2.4s, %[vA].s[0]\n\t"
            "ldr %[scratch1], [%[buff_in0], #152]\n\t"
            "fmul v3.4s, v3.4s, %[vA].s[0]\n\t"
            "ldr %[scratch2], [%[buff_in0], #168]\n\t"
            "fadd v0.4s, v0.4s, %[vB].4s\n\t"
            "ldr %[scratch3], [%[buff_in0], #184]\n\t"
            "fadd v1.4s, v1.4s, %[vB].4s\n\t"
            "ldr %[scratch4], [%[buff_in0], #176]\n\t"
            "st1 {v4.4s-v7.4s}, [%[buff_out]], #64\n\t"
            "ldr d5, [%[buff_in0], #144]\n\t"
            "ins v7.d[1], %[scratch3]\n\t"
            "ldr d6, [%[buff_in0], #160]\n\t"
            "ins v5.d[1], %[scratch1]\n\t"
            "ldr d4, [%[buff_in0], #128]!\n\t"
            "ins v6.d[1], %[scratch2]\n\t"
            "ins v7.d[0], %[scratch4]\n\t"
            "ins v4.d[1], %[scratch0]\n\t"
            "fadd v2.4s, v2.4s, %[vB].4s\n\t"
            "prfm PLDL1KEEP, [%[buff_in1], #192]\n\t"
            "fadd v3.4s, v3.4s, %[vB].4s\n\t"
            "prfm PLDL1KEEP, [%[buff_in1], #256]\n\t"
            "fmul v4.4s, v4.4s, %[vA].s[0]\n\t"
            "ldr %[scratch0], [%[buff_in1], #136]\n\t"
            "fmul v5.4s, v5.4s, %[vA].s[0]\n\t"
            "fmul v6.4s, v6.4s, %[vA].s[0]\n\t"
            "ldr %[scratch1], [%[buff_in1], #152]\n\t"
            "fmul v7.4s, v7.4s, %[vA].s[0]\n\t"
            "ldr %[scratch2], [%[buff_in1], #168]\n\t"
            "fadd v4.4s, v4.4s, %[vB].4s\n\t"
            "ldr %[scratch3], [%[buff_in1], #184]\n\t"
            "fadd v5.4s, v5.4s, %[vB].4s\n\t"
            "ldr %[scratch4], [%[buff_in1], #176]\n\t"
            "st1 {v0.4s-v3.4s}, [%[buff_out]], #64\n\t"
            "ldr d1, [%[buff_in1], #144]\n\t"
            "ins v3.d[1], %[scratch3]\n\t"
            "ldr d2, [%[buff_in1], #160]\n\t"
            "ins v1.d[1], %[scratch1]\n\t"
            "ldr d0, [%[buff_in1], #128]!\n\t"
            "ins v2.d[1], %[scratch2]\n\t"
            "ins v3.d[0], %[scratch4]\n\t"
            "ins v0.d[1], %[scratch0]\n\t"
            "fadd v6.4s, v6.4s, %[vB].4s\n\t"
            "cmp %[buff_in0], %[buff_in0_end]\n\t"
            "fadd v7.4s, v7.4s, %[vB].4s\n\t"
            "b.ne 0b\n\t"
            "fmul v0.4s, v0.4s, %[vA].s[0]\n\t"
            "fmul v1.4s, v1.4s, %[vA].s[0]\n\t"
            "fmul v2.4s, v2.4s, %[vA].s[0]\n\t"
            "fmul v3.4s, v3.4s, %[vA].s[0]\n\t"
            "st1 {v4.4s-v7.4s}, [%[buff_out]], #64\n\t"
            "fadd v0.4s, v0.4s, %[vB].4s\n\t"
            "fadd v1.4s, v1.4s, %[vB].4s\n\t"
            "fadd v2.4s, v2.4s, %[vB].4s\n\t"
            "fadd v3.4s, v3.4s, %[vB].4s\n\t"
            "st1 {v0.4s-v3.4s}, [%[buff_out]]"
            : [buff_in0] "+r"(buff_in0),
              [buff_in1] "+r"(buff_in1),
              [buff_in0_end] "+r"(buff_in0_end),
              [buff_out] "+r"(buff_out),
              [scratch0] "=r"(scratch0),
              [scratch1] "=r"(scratch1),
              [scratch2] "=r"(scratch2),
              [scratch3] "=r"(scratch3),
              [scratch4] "=r"(scratch4)
            : [vA] "w"(s31), [vB] "w"(v30)
            : "cc", "memory", "v0", "v1", "v2", "v3", "v4", "v5", "v6", "v7");
        return gr::work::Status::OK;
    }
#else
    [[nodiscard]] constexpr gr::work::Status processBulk(gr::InputSpanLike auto& inSpan,
                                                         gr::OutputSpanLike auto& outSpan)
    {
        std::transform(inSpan.begin(), inSpan.end(), outSpan.begin(), [this](float x) {
            return a * x + b;
        });
        return gr::work::Status::OK;
    }
#endif
};

// scheduler that uses an externally-provided jobLists
class JobListScheduler
    : public gr::scheduler::SchedulerBase<JobListScheduler,
                                          gr::scheduler::ExecutionPolicy::multiThreaded>
{
    friend class gr::lifecycle::StateMachine<JobListScheduler>;
    friend class gr::scheduler::
        SchedulerBase<JobListScheduler, gr::scheduler::ExecutionPolicy::multiThreaded>;

public:
    using base_t =
        gr::scheduler::SchedulerBase<JobListScheduler,
                                     gr::scheduler::ExecutionPolicy::multiThreaded>;
    explicit JobListScheduler(gr::Graph&& graph,
                              std::vector<std::vector<gr::BlockModel*>>&& jobs)
        : base_t(
              std::move(graph),
              std::make_shared<gr::scheduler::BasicThreadPool>("job-list-scheduler-pool",
                                                               gr::thread_pool::CPU_BOUND,
                                                               jobs.size(),
                                                               jobs.size())),
          _jobs(std::move(jobs))
    {
    }

private:
    std::vector<std::vector<gr::BlockModel*>> _jobs;

    void init()
    {
        base_t::init();
        std::lock_guard lock(_jobListsMutex);
        *_jobLists = std::move(_jobs);
    }
};

void single_core()
{
    std::random_device rd;
    std::default_random_engine gen(rd());
    std::uniform_real_distribution<float> dist;

    gr::Graph fg;
    auto& source = fg.emplaceBlock<DummySource<float>>();
    auto& saxpy = fg.emplaceBlock<Saxpy>({ { "a", dist(gen) }, { "b", dist(gen) } });
    auto& sink = fg.emplaceBlock<BenchmarkSink<float>>();
    if (fg.connect<"out">(source).to<"in">(saxpy) != gr::ConnectionResult::SUCCESS) {
        std::terminate();
    }
    if (fg.connect<"out">(saxpy).to<"in">(sink) != gr::ConnectionResult::SUCCESS) {
        std::terminate();
    }

    gr::scheduler::Simple sched{ std::move(fg) };
    if (!sched.runAndWait().has_value()) {
        std::terminate();
    }
}

void multi_kernel(size_t num_kernels, size_t num_cpus, bool job_list_scheduler)
{
    std::random_device rd;
    std::default_random_engine gen(rd());
    std::uniform_real_distribution<float> dist;

    std::vector<std::vector<gr::BlockModel*>> jobs(num_cpus);
    const std::vector<size_t> kernels_per_core =
        std::views::iota(0UZ, num_cpus) |
        std::views::transform([num_kernels, num_cpus](size_t n) {
            return num_kernels / num_cpus + (n < num_kernels % num_cpus);
        }) |
        std::ranges::to<std::vector>();

    gr::Graph fg;

    // source (core 0)
    auto& source = fg.emplaceBlock<DummySource<float>>();
    jobs.front().push_back(&*fg.blocks().back());

    // first Saxpy kernel (always core 0)
    auto& first_saxpy =
        fg.emplaceBlock<Saxpy>({ { "a", dist(gen) }, { "b", dist(gen) } });
    jobs.front().push_back(&*fg.blocks().back());
    if (fg.connect<"out">(source).to<"in">(first_saxpy) !=
        gr::ConnectionResult::SUCCESS) {
        std::terminate();
    }

    size_t core = 0;
    size_t kernels_in_core = 1; // the first kernel we've already allocated
    Saxpy* previous_saxpy = &first_saxpy;
    for (auto _ : std::views::iota(1UZ, num_kernels)) {
        if (kernels_in_core == kernels_per_core[core]) {
            ++core;
            kernels_in_core = 0;
        }

        auto& saxpy = fg.emplaceBlock<Saxpy>({ { "a", dist(gen) }, { "b", dist(gen) } });
        jobs.at(core).push_back(&*fg.blocks().back());
        if (fg.connect<"out">(*previous_saxpy).to<"in">(saxpy) !=
            gr::ConnectionResult::SUCCESS) {
            std::terminate();
        }
        previous_saxpy = &saxpy;
        ++kernels_in_core;
    }

    // sink (last core)
    auto& sink = fg.emplaceBlock<BenchmarkSink<float>>();
    jobs.back().push_back(&*fg.blocks().back());
    if (fg.connect<"out">(*previous_saxpy).to<"in">(sink) !=
        gr::ConnectionResult::SUCCESS) {
        std::terminate();
    }

    if (job_list_scheduler) {
        JobListScheduler sched{ std::move(fg), std::move(jobs) };
        if (!sched.runAndWait().has_value()) {
            std::terminate();
        }
    } else {
        gr::scheduler::Simple<gr::scheduler::ExecutionPolicy::multiThreaded> sched{
            std::move(fg),
            // limit to only num_cpus worker threads
            std::make_shared<gr::scheduler::BasicThreadPool>(
                "simple-scheduler-pool", gr::thread_pool::CPU_BOUND, num_cpus, num_cpus)
        };
        if (!sched.runAndWait().has_value()) {
            std::terminate();
        }
    }
}

void print_usage(char** argv)
{
    fmt::println(
        "usage: {} <single-core|multi-kernel|multi-kernel-simple> {{options...}}\n",
        argv[0]);
    fmt::println("options for single-core: none");
    fmt::println(
        "options for multi-kernel/multi-kernel-simple: <num_kernels> <num_cpus>");
}

int main(int argc, char** argv)
{
    using namespace std::string_literals;

    if (argc <= 1) {
        print_usage(argv);
        return 1;
    }
    if (argv[1] == "single-core"s) {
        if (argc != 2) {
            print_usage(argv);
            return 1;
        }
        single_core();
    } else if (argv[1] == "multi-kernel"s) {
        if (argc != 4) {
            print_usage(argv);
            return 1;
        }
        const size_t num_kernels = std::stoul(argv[2]);
        const size_t num_cpus = std::stoul(argv[3]);
        multi_kernel(num_kernels, num_cpus, true);
    } else if (argv[1] == "multi-kernel-simple"s) {
        if (argc != 4) {
            print_usage(argv);
            return 1;
        }
        const size_t num_kernels = std::stoul(argv[2]);
        const size_t num_cpus = std::stoul(argv[3]);
        multi_kernel(num_kernels, num_cpus, false);
    } else {
        print_usage(argv);
        return 1;
    }

    return 0;
}
