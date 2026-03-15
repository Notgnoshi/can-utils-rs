# candumpr architecture

## Status

**TODO**

## Scope

This document specifies the internal architecture of candumpr, covering the threading model, I/O
strategy, and the mechanisms used to achieve lossless capture. It does not cover user-facing
features or CLI/config design (see [01-candumpr-ux](01-candumpr-ux.md)).

## Target environment

A modern-ish Linux with io_uring and socketcan available. A ~4 core ~1 GHz arm64 CPU with 1 GB
memory and 4+ J1939 CAN networks.

## Design goal: never drop a frame

TODO: Define what "never drop" means precisely. Kernel socket buffer overflow is the primary
mechanism for frame loss. Describe the end-to-end path from kernel socket buffer to flushed bytes on
disk, and identify every point where frames could be lost or delayed.

## Design goal: lowest system impact

TODO: I'm targeting using candumpr to log traffic from 4 500kbaud CAN networks on a 4 core system
responsible for other application concerns. The logging is not the purpose of the system, it's a
troubleshooting enabler. I'm after a solution with the minimal system performance impact.

## Design goal: long-running CAN logging daemon to facilitate troubleshooting

I want a long-running daemon to log all CAN traffic to facilitate future field issues. That means:

* Address claim PGN requests
* Log rotation policy
* Log retention policy
* Configuration
* Usability with other tools (e.g., .pcap files with Wireshark)

## Option 1: dedicated thread pairs

One recv thread and one write thread per interface. The recv thread reads frames from the socket and
passes them to its paired write thread over a channel. The write thread handles formatting,
compression, and file I/O.

TODO: Describe how io_uring fits in (recv side, write side, or both). Describe the channel type and
backpressure strategy. Describe how log rotation and SIGHUP are coordinated between the two threads.

## Option 2: shared threads

A small number of shared recv threads and shared write threads, rather than a dedicated pair per
interface. This may be a better fit for the target environment of 4 ARM cores with 4+ interfaces,
where dedicating 2 threads per interface would oversubscribe the CPU.

TODO: Describe the multiplexing strategy (io_uring multishot recv, epoll, etc.). Describe how write
work is distributed. Describe how this interacts with per-interface file handles, rotation, and
compression state.

## Back-of-the-napkin math

### Frame rate

A CAN 2.0B extended frame (29-bit ID) with an 8-byte payload uses the following bits on the wire,
assuming zero bitstuffing:

| Field          |    Bits |
| -------------- | ------: |
| SOF            |       1 |
| Base ID        |      11 |
| SRR            |       1 |
| IDE            |       1 |
| Extended ID    |      18 |
| RTR            |       1 |
| r1 (reserved)  |       1 |
| r0 (reserved)  |       1 |
| DLC            |       4 |
| Data (8 bytes) |      64 |
| CRC            |      15 |
| CRC delimiter  |       1 |
| ACK slot       |       1 |
| ACK delimiter  |       1 |
| EOF            |       7 |
| IFS            |       3 |
| **Total**      | **131** |

Reference: linux-can/can-utils
[canframelen.c](https://github.com/linux-can/can-utils/blob/master/canframelen.c) computes
`(eff ? 67 : 47) + len * 8` for the no-bitstuffing case (CFL_NO_BITSTUFFING). The worst-case
bitstuffing formula from
[canframelen.h](https://github.com/linux-can/can-utils/blob/master/canframelen.h) is `80 + 10 * len`
bits for extended frames. IFS (3 bits) is included in both formulas.

Zero bitstuffing is the worst case for frame rate: fewer bits per frame means more frames per
second. J1939 always uses 29-bit extended IDs and 8-byte payloads.

At 500 kbaud with zero bitstuffing: 500,000 / 131 = **3816 frames/sec per bus**.

| Scenario                        | Frames/sec |
| ------------------------------- | ---------: |
| 1 bus, no bitstuffing           |      3,816 |
| 1 bus, worst-case bitstuffing   |      3,125 |
| 4 buses, no bitstuffing         |     15,264 |
| 4 buses, worst-case bitstuffing |     12,500 |

We'll proceed assuming 3,816 frames/sec, acknowledging that this would be 100% busload, which
doesn't happen in practice.

### Per-frame recv cost

The recv path for a single frame (ignoring formatting and write):

1. The kernel delivers the frame to the socket buffer via interrupt. This cost is the same across
   all backends.
2. The receiver wakes (or discovers a new CQE) and reads the frame.
3. The on_frame callback runs.

The distinguishing cost is step 2: how many syscalls the receiver makes per frame, and how much
userspace work each backend does.

| Approach                  | Syscalls/frame | Notes                                              |
| ------------------------- | -------------: | -------------------------------------------------- |
| Dedicated (blocking read) |              1 | One `read()` per frame per thread                  |
| epoll + read              |            1-2 | `epoll_wait` + `read`; drain loop adds EAGAIN read |
| epoll + recvmmsg          |             <1 | Batched reads reduce per-frame count               |
| io_uring single-shot      |             ~1 | `submit_and_wait` both submits and collects        |
| io_uring multishot        |             <1 | Multiple CQEs per `submit_and_wait`; no resubmit   |

For the multiplexed backends, frames arriving on multiple sockets within the same
`epoll_wait`/`submit_and_wait` window are serviced in a single wakeup. At 3816 fps per bus across 4
buses, the mean inter-arrival across all sockets is ~65us, so overlapping arrivals are common.

On a 1 GHz ARM64 processor, a syscall round trip (userspace to kernel and back) takes roughly 2-5us
depending on kernel mitigations and the specific operation. Using 3us as a rough estimate:

| Approach           | Est. recv CPU (4 buses, 3816 fps each) | Threads |
| ------------------ | -------------------------------------: | ------: |
| Dedicated          |                        ~6% of one core |       4 |
| epoll + read       |                      ~4-6% of one core |       1 |
| io_uring multishot |                        ~2% of one core |       1 |

These cover only recv syscall overhead, excluding on_frame processing, scheduling, and cache
effects.

### Context switches

Each sleep/wake cycle is a voluntary context switch. The scheduling cost itself is small (~1-5 us),
but each wakeup pollutes L1/L2 caches, affecting co-resident applications.

| Approach                             | Threads | Est. context switches/sec (4 buses) |
| ------------------------------------ | ------: | ----------------------------------: |
| Dedicated                            |       4 |                        Up to 15,264 |
| Multiplexed (one frame per wakeup)   |       1 |                        Up to 15,264 |
| Multiplexed (multiple frames/wakeup) |       1 |   Fewer; depends on arrival overlap |

The dedicated backend also incurs involuntary context switches when its recv threads compete with
application threads for cores.

### Plausible bottlenecks

1. **Context switches and cache pollution.** At worst-case rates, the receiver wakes up to ~15,000
   times/sec. Each wakeup pollutes L1/L2 caches, affecting co-resident applications. This is likely
   the dominant source of system impact, since the raw CPU cost for recv is small (2-6% of one
   core).

2. **Socket buffer overflow.** The CAN_RAW receive buffer capacity depends on `rmem_max` and sk_buff
   overhead; on a typical system it may hold only a few hundred frames. At 3816 fps, even a buffer
   of 200 frames fills in ~52 ms if the receiver stalls. Any write-path backpressure lasting longer
   causes frame loss.

3. **Thread oversubscription (dedicated only).** With 4 recv threads on a 4-core system, the recv
   threads alone use all available cores before accounting for write threads or other application
   work. Involuntary preemption increases and cache efficiency drops.

4. **Write path stalls (out of scope for recv benchmarks).** The recv path must drain the socket
   buffer faster than frames arrive, even when the write path stalls for disk I/O or log rotation.
   The socket buffer depth sets the maximum tolerable stall.

   candumpr will use a dedicated write thread and another larger and growable frame queue to
   mitigate this. This is an important choice, because we can't arbitrarily grow the recvbuf,
   there's a maximum limit.

## Benchmarking strategy

### Goals

The recv benchmarks answer three questions:

1. **Multiplexed vs. dedicated.** Does a single-threaded multiplexed receiver have lower system
   impact than one-thread-per-socket on a 4-core system with 4 CAN interfaces?

2. **Which multiplexed backend.** Among epoll + read, epoll + recvmmsg, io_uring single-shot, and
   io_uring multishot with provided buffers: which has the lowest per-frame overhead and the fewest
   context switches?

3. **Where to optimize.** What is the per-frame instruction cost, and where are the hot spots?

### Test environment

All benchmarks run on vcan interfaces inside an isolated user + network namespace (via
`unshare(2)`). This eliminates the need for root or hardware CAN interfaces.

vcan delivers frames as fast as possible synchronously through the kernel's loopback path, with no
bus timing or contention. This is appropriate for measuring recv path overhead in isolation. Results
should be validated on the target hardware before making final decisions.

### Benchmark A: per-frame instruction cost

**Purpose.** Measure userspace instruction overhead per frame for each backend. Identify which code
paths dominate the per-frame cost and where optimization effort should focus.

**Method.** Callgrind-based profiling via gungraun.

**Setup.**

1. Create 4 vcan interfaces.
2. Open one TX and one RX socket per interface.
3. Pre-send frames into the RX socket buffers. The number per interface is limited by the kernel
   socket receive buffer (constrained by `rmem_max`, which cannot be raised inside a user
   namespace). The benchmark should determine the usable capacity at runtime and fill to that limit.
4. Start the profiled region.
5. Run the backend to drain all frames.
6. End the profiled region.

Pre-filling rather than concurrent sending ensures that the profiled region contains only recv work
and that the send cost is identical across backends.

**Callback fairness.** The on_frame callback must have identical cost across all backends. The
dedicated backend runs multiple threads, so its counting mechanism needs to be thread-safe. Each
dedicated recv thread should count frames in a thread-local variable (not a shared atomic) to avoid
penalizing it with synchronization overhead that belongs to the test harness, not the backend. The
single-threaded backends should use the same local-variable approach.

**Metrics.**

* Instructions per frame
* L1 data cache miss rate
* Branch misprediction rate

**Limitations.** Callgrind counts userspace instructions only. For io_uring backends, kernel-side
CQE processing and buffer ring management are not captured. Treat io_uring instruction counts as a
lower bound that excludes kernel work.

### Benchmark B: steady-state system impact

**Purpose.** Measure the receiver's CPU time and scheduling overhead under sustained load at
realistic CAN frame rates.

**Method.** Concurrent senders and receiver. Collect per-thread resource usage for the receiver
only.

**Setup.**

1. Create vcan interfaces in an isolated namespace.
2. Spawn one sender thread per interface. Senders pace frames at the target rate using sleep-based
   timing (`clock_nanosleep` with `TIMER_ABSTIME`). Do not use busy-spin pacing; it burns CPU and
   contaminates resource measurements.
3. Run the backend under test on the receiver thread.
4. Collect resource usage via `getrusage(RUSAGE_THREAD)` on the receiver thread before and after the
   run. For the dedicated backend, collect `RUSAGE_THREAD` from each sub-thread and aggregate.
5. A timer thread stops all threads after the run duration.

**Metric isolation.** Using `RUSAGE_THREAD` rather than `RUSAGE_SELF` excludes sender threads, the
timer thread, and all other process-level overhead from the measurements. At realistic frame rates,
the receiver's CPU contribution is small and would be invisible in a process-wide measurement.

**Test matrix.**

| Parameter            | Values                                         |
| -------------------- | ---------------------------------------------- |
| Backends             | dedicated, epoll, recvmmsg, uring, uring_multi |
| Interfaces           | 1, 2, 4                                        |
| Rate (per interface) | 1000 fps, 2000 fps, 4000 fps                   |
| Duration             | 8 seconds                                      |
| Repetitions          | 4, report median by receiver kernel time       |
| Core constraint      | Use `taskset -c 0-3` to limit to 4 cores       |

5 backends x 3 interface counts x 3 rates = 45 configurations. At 4 repetitions and 8 seconds per
run, a full sweep takes roughly 24 minutes.

**Metrics (per run).**

| Metric                   | Source                    | Purpose                        |
| ------------------------ | ------------------------- | ------------------------------ |
| Receiver user CPU (ms)   | RUSAGE_THREAD `ru_utime`  | Userspace processing cost      |
| Receiver kernel CPU (ms) | RUSAGE_THREAD `ru_stime`  | Kernel time for recv syscalls  |
| Receiver voluntary csw   | RUSAGE_THREAD `ru_nvcsw`  | Sleep/wake frequency           |
| Receiver involuntary csw | RUSAGE_THREAD `ru_nivcsw` | Preemption frequency           |
| Frames sent              | Sender counter            | Confirms rate pacing accuracy  |
| Frames received          | Receiver counter          | Confirms lossless capture      |
| Frame loss %             | (sent - recv) / sent      | Must be 0% at all tested rates |

**Rate pacing accuracy.** At 4000 fps per interface, the inter-frame interval is 250us.
`clock_nanosleep` with absolute timestamps should achieve this within a few microseconds of jitter.
Verify that the actual sent count matches the expected count (rate x duration) within 1%.

### Benchmark C: recv under CPU contention

**Purpose.** Determine which backend is most resilient to frame loss when the system is under CPU
pressure from other workloads. candumpr is an ancillary concern on the target system; the primary
application may consume most of the available CPU, and the recv backend must survive this without
dropping frames.

**Method.** Run benchmark B's send/recv setup alongside a synthetic CPU load on the same cores.
Measure frame loss at different contention levels.

**Setup.**

1. Create 4 vcan interfaces in an isolated namespace.
2. Start a CPU load generator on the same cores as the benchmark. Use
   `stress-ng --cpu 4 --cpu-load P --taskset 0-3` where P is the target load percentage. Each worker
   duty-cycles between burning and sleeping to approximate P% utilization per core.
3. Run the send/recv harness from benchmark B (sleep-paced senders, receiver, timer) on the same
   cores.
4. Collect the same per-thread metrics as benchmark B, plus frame loss.

**Test matrix.**

| Parameter       | Values                                         |
| --------------- | ---------------------------------------------- |
| Backends        | dedicated, epoll, recvmmsg, uring, uring_multi |
| Interfaces      | 4                                              |
| Rate            | 4000 fps per interface                         |
| CPU contention  | 75%, 95%                                       |
| Duration        | 8 seconds                                      |
| Repetitions     | 4, report median by frame loss %               |
| Core constraint | `taskset -c 0-3`                               |

5 backends x 2 contention levels = 10 configurations. At 4 repetitions and 8 seconds per run, a full
sweep takes roughly 6 minutes.

**Metrics (per run).**

| Metric                   | Source                    | Purpose                              |
| ------------------------ | ------------------------- | ------------------------------------ |
| Frame loss %             | (sent - recv) / sent      | Primary: resilience under contention |
| Receiver user CPU (ms)   | RUSAGE_THREAD `ru_utime`  | How much CPU the receiver got        |
| Receiver kernel CPU (ms) | RUSAGE_THREAD `ru_stime`  | Kernel time under contention         |
| Receiver voluntary csw   | RUSAGE_THREAD `ru_nvcsw`  | Wakeup frequency under pressure      |
| Receiver involuntary csw | RUSAGE_THREAD `ru_nivcsw` | How often the receiver was preempted |

**What to look for.** At 75% contention, all backends should remain lossless (the receiver needs
only 2-6% of one core). At 95%, some backends may start dropping frames. The interesting result is
the relative degradation: a backend that degrades gradually (small loss %) is preferable to one that
collapses suddenly (large loss %).

### Caveats

* **vcan is not a real CAN bus.** There is no bus arbitration, no propagation delay, no error
  frames, no bitstuffing, and no hardware interrupt path. These benchmarks measure the software recv
  overhead only.
* **Callgrind and io_uring.** Instruction counts for io_uring backends undercount the true per-frame
  cost because kernel-side ring processing is not instrumented.
* **x86_64 vs. ARM64.** Benchmarks run on a development workstation. Syscall costs, cache sizes, and
  branch predictor behavior differ on the target ARM64 platform. Use these results for relative
  comparison between backends, not as absolute predictions.

## Open questions

TODO
