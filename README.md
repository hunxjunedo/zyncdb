# ZyncDB

ZyncDB is a small in-memory key-value server with a line-based TCP protocol.
This is a learning project, not production software.

## Benchmark headline

At 50 clients, the report records 104,025 GET/s, 105,528 SET/s, and 105,730 mixed requests/s; mixed-workload p99 was 0.906 ms. The machine was a MacBookPro15,2, and the server and load generator ran on the same laptop. See the [full benchmark report](./benchmarks/main/a45dbab-dirty/README.md) and its [raw runs](./benchmarks/main/a45dbab-dirty/runs.csv).

## Features

- Accepts case-sensitive `GET`, `SET`, `DEL`, and `EXP` commands, one per line.
- Stores string keys and single-token string values in memory.
- `SET` creates or replaces a value. An optional positive TTL is in seconds. Setting a value without a TTL clears any previous TTL.
- `GET` returns the value. It reports an error for a missing or expired key.
- `DEL` removes a key. Its reply distinguishes an existing key from a missing key.
- `EXP` sets or replaces a TTL on an existing, unexpired key. It reports an error if the key does not exist or is expired.
- Expired keys are removed when accessed and by a background sweeper.
- TTL arguments must be positive unsigned 64-bit integers.

Commands and arguments are separated by spaces. Keys and values cannot contain spaces.

## Quick start

The server reads four required environment variables. Copy the example file, build the release binary, and run it:

```sh
cp .example.env .env
cargo build --release
./target/release/zyncdb
```

The server listens on `127.0.0.1:8080`. In another terminal, connect with `nc`:

```sh
nc 127.0.0.1 8080
SET greeting hello
GET greeting
```

The `GET` reply is `hello`. Press Ctrl-C in the server terminal to stop it.

## Configuration

All four variables in `.example.env` are read by the server. They must be present and parse as the indicated type.

| Variable | Example value | Meaning |
|---|---:|---|
| `CONNECTION_LIMIT` | `20000` | Semaphore limit for active connection-processing tasks. |
| `MAX_COMMAND_LENGTH` | `1000` | Maximum line length passed to the line codec. Longer lines receive `command very large`. |
| `SWEEPER_FREQUENCY` | `1` | Seconds between background expiry sweeps. |
| `CONNECTION_TIMEOUT` | `10` | Seconds to wait for the next complete command line before closing a connection. |

The bind address and port are fixed in the server code; there is no environment variable for them.

## Protocol

Each command is a text line. Each reply is a text line. The command parser accepts uppercase command names.

| Command | Syntax | Reply |
|---|---|---|
| `GET` | `GET key` | The stored value. If absent or expired: `ERROR: Value does not exist!` |
| `SET` | `SET key value [ttl_seconds]` | On creation: `Field did not exist, but created it`. On replacement: `Value updated`. |
| `DEL` | `DEL key` | If removed: `key removed succesfully`. If absent: `the key did not exist`. |
| `EXP` | `EXP key ttl_seconds` | On first TTL: `TTL added`. On TTL replacement: `TTL updated`. |

Command parsing and validation errors are sent as `ERROR: <message>`. The exact messages are:

| Condition | Message |
|---|---|
| Unknown or invalid command | `invalid command` |
| `GET` or `DEL` missing the key | `field name not provided` |
| `SET` missing its key or value | `required arguments not provided` |
| `EXP` missing its key or TTL | `required arguments not provided` |
| `SET` TTL is zero, negative, non-numeric, or too large | `TTL should be an unsigned 64-bit integer` |
| `EXP` TTL is zero, negative, non-numeric, or too large | `TTL should be an unsigned 64-bit integer` |
| `EXP` targets an absent or expired key | `could not add TTL: key does not exist!` |

The parser-error response has a trailing space after the message before its newline. The missing-key `GET` and `EXP` replies are database errors and do not have that trailing space. A line rejected for exceeding the configured maximum receives `command very large` without an `ERROR:` prefix.

## Design

```text
TCP clients
    | one line at a time
    v
connection task per client
    | Command -> DbOperation
    | request through bounded mpsc channel (capacity 100)
    v
single database worker task -> owns the HashMap
    ^
    | RemoveExpiredKeys operation
expiry sweeper task
```

- Each accepted connection is handled by its own task. It reads and processes commands sequentially, waiting for each database reply before reading the next command.
- One database worker owns the `HashMap`. Connection tasks send database operations to it through a bounded channel with capacity 100 and receive replies through one-shot channels.
- `Command` represents user-facing parsed commands. Each command produces a `DbOperation`, the internal operation executed against the database.
- Expiry is checked lazily when a key is read or its expiry is changed. The background sweeper periodically scans for expired keys and removes up to 100 per sweep.
- A semaphore limits active connection-processing tasks using `CONNECTION_LIMIT`.
- `MAX_COMMAND_LENGTH` limits line length. `CONNECTION_TIMEOUT` limits how long the server waits for a complete next line.
- On Ctrl-C, the listener stops accepting connections and cancellation is sent to connection tasks. The main task waits for those connection-task handles. The database worker and sweeper are not explicitly joined.

## Decisions and trade-offs

The code gives the database to one worker task and sends operations to it over a channel. This keeps one owner for the data and avoids sharing the map behind `Arc<Mutex<HashMap>>`. The code comments also cite easier reasoning and data consistency as reasons for this design.

The worker executes database operations one at a time. That single task is therefore the ceiling for database-operation throughput. The benchmark report measures this single-store-task design.

## Benchmarks

The report records the following method:

- It tested GET-only, SET-only, and mixed workloads. Mixed requests alternated SET and GET.
- Each run used 10,000 prefilled keys, `k0` through `k9999`; GETs therefore found values and the map did not grow during a run.
- SET values were random English words. The prefill value was `sample_val`.
- Clients used a closed loop: each client waited for a reply before sending its next request. The client counts were 1, 10, 50, and 100.
- Request timing started just before writing a command and ended when its reply line was read. Filling, command generation, and connecting were outside the timed interval.
- One warm-up run was discarded. Three sweeps followed, with a 10-second pause between runs. Table entries are medians; the requests-per-second column also gives the range.

Results below are the medians in the report. Times are milliseconds. p99 is the report's 99th-percentile reply time. Errors were not checked by this version of the load generator.

| Workload | Clients | Requests/client | Requests/s (range) | p50 | p99 | p99.9 | Slowest reply | Errors |
|---|---:|---:|---:|---:|---:|---:|---:|---|
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

The report lists these caveats:

- The server and load generator ran on the same machine and competed for CPU.
- Closed-loop clients can make slow moments look smaller than they would with a steady arrival rate.
- The key sequence used an unseeded random generator, so runs varied; the report includes medians and ranges.
- In the mixed workload, each GET read the key that the same client had just written.

See the [benchmarks directory](./benchmarks/) for the [full report](./benchmarks/main/a45dbab-dirty/README.md) and [raw runs](./benchmarks/main/a45dbab-dirty/runs.csv).

## Tests

Run the test suite with:

```sh
cargo test
```

There are 20 tests in two test modules. [`parser_test.rs`](./tests/parser_test.rs) has 7 tests for command parsing, missing arguments, and TTL validation. [`unit_test.rs`](./tests/unit_test.rs) has 13 tests for database operations, missing keys, updates, TTL expiry and replacement, and removal by the sweeper.

## Not implemented

- Persistence: data is held in an in-memory `HashMap`.
- Replication.
- Values or keys containing spaces; the parser takes space-separated tokens.
- RESP2. The server reads and writes newline-delimited text instead.

## What I would do next

- Explore sharding the single database worker. Sharding is not implemented.
- Add RESP2 support if using `redis-benchmark` is a goal. RESP2 is not implemented.

## AI usage

AI was used only to consult on design decisions and generate documentation, including Markdown files. No code was generated by AI.

## Things I could not verify

- None. This README contains no `[TODO]` placeholders.
