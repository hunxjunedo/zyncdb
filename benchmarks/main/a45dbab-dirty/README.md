
# ZyncDB benchmark: `main` @ `a45dbab`

Baseline: single store task

**Headline (50 clients, median of 3 sweeps):** 104,025 GET/s, 105,528 SET/s, 105,730 mixed requests/s. Slowest 1% of mixed replies under 0.91 ms.

## Version

| | |
|---|---|
| Branch | `main` |
| Commit | `a45dbab` (**uncommitted changes present**) |
| Commit message | write benchmark results to markdown |
| Date | 2026-11-05 02:07 |
| Build | release |
| Rust | rustc 1.97.1 (8bab26f4f 2026-07-14) |
| Load generator command | `cargo run --release --bin load-generator` |

## Environment

| | |
|---|---|
| Machine | MacBookPro15,2 |
| CPU / cores | Intel(R) Core(TM) i7-8559U CPU @ 2.70GHz, 8 logical cores |
| RAM | 16 GB |
| OS | Darwin 24.6.0 |
| `ulimit -n` | 1048575 |
| Server and load generator | server and load generator on the same laptop |
| Conditions | plugged in, other apps closed |

## Server configuration

| Setting | Value |
|---|---|
| `CONNECTION_LIMIT` | 20000 |
| `MAX_COMMAND_LENGTH` | 1000 |
| `CONNECTION_TIMEOUT` (s) | 10 |
| `SWEEPER_FREQUENCY` (s) | 1 |

## Method

- **Workloads:** GET only, SET only, mixed (alternating SET and GET).
- **Keys:** 10000 keys (`k0` to `k9999`), all filled with a known value before every run, so GETs always find a value and the map does not grow during a run.
- **Values:** SET values are random English words; the fill step before each run uses the fixed value `sample_val`.
- **Clients:** closed loop (each client waits for a reply before sending the next request). Client counts: 1, 10, 50, 100.
- **Timing:** each request is timed from just before it is written until its reply line is read. Requests per second is total requests divided by the wall-clock time from when all clients are connected until the last reply. Filling, command generation and connecting are outside the clock.
- **Repeats:** One warm-up run (mixed, 10 clients, 10,000 requests each) was discarded. Then 3 sweeps of every run, 10 s pause between runs. Each cell is the **median** of the 3 runs; requests per second also shows the range. "Slowest reply" is the worst single reply seen in any run.
- **Validity:** This version of the load generator does not check replies for errors, so error counts are not reported.

## Results

Median of 3 runs. Times in milliseconds. p99 = the slowest 1%: 99 of every 100 replies were faster than this.

| Workload | Clients | Requests per client | Requests per second (range) | p50 | p99 | p99.9 | Slowest reply | Errors |
|---|---|---|---|---|---|---|---|---|
| GET | 1 | 100,000 | 21,946 (20,466 to 22,965) | 0.04 | 0.101 | 0.165 | 2.4 | not checked |
| GET | 10 | 50,000 | 70,071 (65,016 to 70,085) | 0.135 | 0.286 | 0.467 | 36.3 | not checked |
| GET | 50 | 25,000 | 104,025 (102,137 to 106,967) | 0.451 | 1.015 | 2.155 | 81.7 | not checked |
| GET | 100 | 15,000 | 112,653 (108,974 to 113,294) | 0.873 | 1.595 | 2.211 | 19.8 | not checked |
| SET | 1 | 100,000 | 21,378 (18,882 to 21,428) | 0.041 | 0.099 | 0.144 | 3.8 | not checked |
| SET | 10 | 50,000 | 67,556 (59,053 to 68,602) | 0.138 | 0.296 | 0.517 | 92.2 | not checked |
| SET | 50 | 25,000 | 105,528 (88,409 to 106,208) | 0.454 | 0.931 | 1.785 | 93 | not checked |
| SET | 100 | 15,000 | 106,527 (89,845 to 109,451) | 0.892 | 1.979 | 4.413 | 77.6 | not checked |
| Mixed | 1 | 100,000 | 21,311 (13,920 to 21,334) | 0.042 | 0.1 | 0.154 | 124.9 | not checked |
| Mixed | 10 | 50,000 | 69,533 (56,064 to 69,707) | 0.136 | 0.285 | 0.391 | 51.6 | not checked |
| Mixed | 50 | 25,000 | 105,730 (97,443 to 106,712) | 0.451 | 0.906 | 1.806 | 111.8 | not checked |
| Mixed | 100 | 15,000 | 108,947 (107,525 to 111,649) | 0.879 | 1.927 | 3.159 | 22.9 | not checked |





## Every individual run

<details>
<summary>All 36 runs (before taking the median)</summary>

| Sweep | Workload | Clients | Requests per second | p50 (ms) | p99 (ms) | p99.9 (ms) | Slowest (ms) | Errors |
|---|---|---|---|---|---|---|---|---|
| 1 | GET | 1 | 20,466 | 0.041 | 0.12 | 0.213 | 0.9 | not checked |
| 1 | GET | 10 | 70,085 | 0.135 | 0.284 | 0.409 | 10.1 | not checked |
| 1 | GET | 50 | 102,137 | 0.453 | 1.148 | 2.216 | 65.5 | not checked |
| 1 | GET | 100 | 108,974 | 0.877 | 1.99 | 2.89 | 12 | not checked |
| 1 | SET | 1 | 18,882 | 0.044 | 0.137 | 0.359 | 3.2 | not checked |
| 1 | SET | 10 | 59,053 | 0.147 | 0.501 | 1.299 | 92.2 | not checked |
| 1 | SET | 50 | 88,409 | 0.479 | 1.888 | 6.975 | 93 | not checked |
| 1 | SET | 100 | 89,845 | 0.945 | 3.59 | 10.392 | 77.6 | not checked |
| 1 | Mixed | 1 | 13,920 | 0.053 | 0.244 | 1.288 | 124.9 | not checked |
| 1 | Mixed | 10 | 56,064 | 0.151 | 0.602 | 1.753 | 51.6 | not checked |
| 1 | Mixed | 50 | 97,443 | 0.46 | 1.443 | 3.373 | 111.8 | not checked |
| 1 | Mixed | 100 | 108,947 | 0.879 | 1.927 | 3.455 | 22.9 | not checked |
| 2 | GET | 1 | 21,946 | 0.04 | 0.101 | 0.165 | 2.4 | not checked |
| 2 | GET | 10 | 70,071 | 0.134 | 0.286 | 0.467 | 20.4 | not checked |
| 2 | GET | 50 | 106,967 | 0.449 | 0.9 | 1.563 | 12.9 | not checked |
| 2 | GET | 100 | 112,653 | 0.873 | 1.595 | 2.211 | 7.9 | not checked |
| 2 | SET | 1 | 21,428 | 0.041 | 0.099 | 0.143 | 3.8 | not checked |
| 2 | SET | 10 | 68,602 | 0.138 | 0.288 | 0.4 | 4.4 | not checked |
| 2 | SET | 50 | 105,528 | 0.453 | 0.931 | 1.785 | 20.7 | not checked |
| 2 | SET | 100 | 109,451 | 0.883 | 1.717 | 2.902 | 52.7 | not checked |
| 2 | Mixed | 1 | 21,334 | 0.041 | 0.1 | 0.154 | 5.3 | not checked |
| 2 | Mixed | 10 | 69,707 | 0.136 | 0.285 | 0.391 | 2.9 | not checked |
| 2 | Mixed | 50 | 106,712 | 0.451 | 0.889 | 1.337 | 11.3 | not checked |
| 2 | Mixed | 100 | 107,525 | 0.885 | 2.028 | 3.159 | 19.5 | not checked |
| 3 | GET | 1 | 22,965 | 0.04 | 0.089 | 0.125 | 0.8 | not checked |
| 3 | GET | 10 | 65,016 | 0.139 | 0.364 | 1.025 | 36.3 | not checked |
| 3 | GET | 50 | 104,025 | 0.451 | 1.015 | 2.155 | 81.7 | not checked |
| 3 | GET | 100 | 113,294 | 0.868 | 1.588 | 2.175 | 19.8 | not checked |
| 3 | SET | 1 | 21,378 | 0.041 | 0.099 | 0.144 | 0.6 | not checked |
| 3 | SET | 10 | 67,556 | 0.137 | 0.296 | 0.517 | 28.5 | not checked |
| 3 | SET | 50 | 106,208 | 0.454 | 0.876 | 1.231 | 11.4 | not checked |
| 3 | SET | 100 | 106,527 | 0.892 | 1.979 | 4.413 | 63.9 | not checked |
| 3 | Mixed | 1 | 21,311 | 0.042 | 0.096 | 0.142 | 0.6 | not checked |
| 3 | Mixed | 10 | 69,533 | 0.136 | 0.284 | 0.384 | 1.6 | not checked |
| 3 | Mixed | 50 | 105,730 | 0.451 | 0.906 | 1.806 | 51.6 | not checked |
| 3 | Mixed | 100 | 111,649 | 0.875 | 1.639 | 2.196 | 11.7 | not checked |

</details>

## Caveats

- The server and the load generator share a machine unless the Environment table says otherwise, so they compete for CPU.
- Clients are closed loop, so slow moments can look smaller than a user with a steady arrival rate would experience.
- Key sequences come from an unseeded random generator, so runs vary slightly; the median and range account for that.
- In the mixed workload, each GET reads the key the same client just wrote.

---

Generated by the ZyncDB load generator on 2026-11-05 02:07. Raw data: `runs.csv`.
