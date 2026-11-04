
# ZyncDB benchmark: `main` @ `1396ef5`

Baseline: single store task

**Headline (50 clients, median of 3 sweeps):** 77,495 GET/s, 77,247 SET/s, 77,765 mixed requests/s. Slowest 1% of mixed replies under 1.31 ms.

## Version

| | |
|---|---|
| Branch | `main` |
| Commit | `1396ef5` (**uncommitted changes present**) |
| Commit message | fix cancellation logic, final steps of load-generator, run and consolidate all tests properly, find median metrics also |
| Date | 2026-11-05 01:53 |
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
| GET | 1 | 100,000 | 15,261 (14,805 to 15,329) | 0.061 | 0.126 | 0.193 | 92.4 | not checked |
| GET | 10 | 50,000 | 49,278 (48,312 to 49,487) | 0.188 | 0.411 | 0.602 | 99.1 | not checked |
| GET | 50 | 25,000 | 77,495 (77,206 to 78,468) | 0.602 | 1.422 | 2.944 | 61.4 | not checked |
| GET | 100 | 15,000 | 86,208 (85,429 to 87,085) | 1.093 | 2.419 | 3.52 | 98.2 | not checked |
| SET | 1 | 100,000 | 14,861 (14,856 to 15,017) | 0.062 | 0.129 | 0.188 | 8.6 | not checked |
| SET | 10 | 50,000 | 48,451 (48,052 to 48,486) | 0.191 | 0.418 | 0.648 | 46.2 | not checked |
| SET | 50 | 25,000 | 77,247 (66,528 to 77,362) | 0.61 | 1.323 | 2.142 | 127.1 | not checked |
| SET | 100 | 15,000 | 83,439 (80,430 to 83,780) | 1.125 | 2.62 | 4.139 | 111.6 | not checked |
| Mixed | 1 | 100,000 | 13,636 (11,538 to 15,000) | 0.062 | 0.171 | 0.593 | 102.1 | not checked |
| Mixed | 10 | 50,000 | 48,886 (41,715 to 49,253) | 0.19 | 0.412 | 0.611 | 48.8 | not checked |
| Mixed | 50 | 25,000 | 77,765 (77,422 to 77,962) | 0.606 | 1.314 | 2.149 | 100.7 | not checked |
| Mixed | 100 | 15,000 | 84,800 (84,748 to 86,335) | 1.103 | 2.43 | 3.573 | 127.2 | not checked |





## Every individual run

<details>
<summary>All 36 runs (before taking the median)</summary>

| Sweep | Workload | Clients | Requests per second | p50 (ms) | p99 (ms) | p99.9 (ms) | Slowest (ms) | Errors |
|---|---|---|---|---|---|---|---|---|
| 1 | GET | 1 | 15,329 | 0.06 | 0.126 | 0.193 | 8.4 | not checked |
| 1 | GET | 10 | 49,487 | 0.186 | 0.388 | 0.519 | 99.1 | not checked |
| 1 | GET | 50 | 78,468 | 0.599 | 1.325 | 2.742 | 22.6 | not checked |
| 1 | GET | 100 | 87,085 | 1.079 | 2.335 | 3.27 | 98.2 | not checked |
| 1 | SET | 1 | 14,861 | 0.062 | 0.135 | 0.219 | 8.6 | not checked |
| 1 | SET | 10 | 48,052 | 0.191 | 0.422 | 0.739 | 46.2 | not checked |
| 1 | SET | 50 | 66,528 | 0.658 | 2.386 | 5.175 | 127.1 | not checked |
| 1 | SET | 100 | 80,430 | 1.138 | 3.197 | 6.553 | 71.6 | not checked |
| 1 | Mixed | 1 | 11,538 | 0.064 | 0.244 | 0.828 | 93 | not checked |
| 1 | Mixed | 10 | 41,715 | 0.206 | 0.721 | 1.751 | 48.8 | not checked |
| 1 | Mixed | 50 | 77,962 | 0.606 | 1.331 | 2.149 | 21.5 | not checked |
| 1 | Mixed | 100 | 84,748 | 1.103 | 2.436 | 3.592 | 127.2 | not checked |
| 2 | GET | 1 | 14,805 | 0.061 | 0.136 | 0.205 | 92.4 | not checked |
| 2 | GET | 10 | 49,278 | 0.188 | 0.411 | 0.602 | 10.4 | not checked |
| 2 | GET | 50 | 77,206 | 0.604 | 1.422 | 2.944 | 61.4 | not checked |
| 2 | GET | 100 | 86,208 | 1.093 | 2.419 | 3.52 | 30.1 | not checked |
| 2 | SET | 1 | 14,856 | 0.062 | 0.129 | 0.188 | 0.5 | not checked |
| 2 | SET | 10 | 48,451 | 0.191 | 0.418 | 0.648 | 3.1 | not checked |
| 2 | SET | 50 | 77,247 | 0.61 | 1.323 | 2.142 | 80 | not checked |
| 2 | SET | 100 | 83,780 | 1.118 | 2.488 | 3.781 | 111.6 | not checked |
| 2 | Mixed | 1 | 15,000 | 0.062 | 0.128 | 0.192 | 1.5 | not checked |
| 2 | Mixed | 10 | 48,886 | 0.19 | 0.412 | 0.611 | 4.1 | not checked |
| 2 | Mixed | 50 | 77,765 | 0.606 | 1.294 | 1.943 | 100.7 | not checked |
| 2 | Mixed | 100 | 84,800 | 1.105 | 2.43 | 3.573 | 97.7 | not checked |
| 3 | GET | 1 | 15,261 | 0.061 | 0.125 | 0.184 | 0.6 | not checked |
| 3 | GET | 10 | 48,312 | 0.188 | 0.432 | 0.875 | 32.2 | not checked |
| 3 | GET | 50 | 77,495 | 0.602 | 1.434 | 3.073 | 40.8 | not checked |
| 3 | GET | 100 | 85,429 | 1.093 | 2.543 | 4.772 | 50.1 | not checked |
| 3 | SET | 1 | 15,017 | 0.062 | 0.126 | 0.18 | 1 | not checked |
| 3 | SET | 10 | 48,486 | 0.191 | 0.413 | 0.601 | 6 | not checked |
| 3 | SET | 50 | 77,362 | 0.61 | 1.289 | 2.062 | 95.5 | not checked |
| 3 | SET | 100 | 83,439 | 1.125 | 2.62 | 4.139 | 14.4 | not checked |
| 3 | Mixed | 1 | 13,636 | 0.062 | 0.171 | 0.593 | 102.1 | not checked |
| 3 | Mixed | 10 | 49,253 | 0.189 | 0.402 | 0.579 | 5.4 | not checked |
| 3 | Mixed | 50 | 77,422 | 0.605 | 1.314 | 2.675 | 94.3 | not checked |
| 3 | Mixed | 100 | 86,335 | 1.094 | 2.385 | 3.486 | 41.9 | not checked |

</details>

## Caveats

- The server and the load generator share a machine unless the Environment table says otherwise, so they compete for CPU.
- Clients are closed loop, so slow moments can look smaller than a user with a steady arrival rate would experience.
- Key sequences come from an unseeded random generator, so runs vary slightly; the median and range account for that.
- In the mixed workload, each GET reads the key the same client just wrote.

---

Generated by the ZyncDB load generator on 2026-11-05 01:53. Raw data: `runs.csv`.
