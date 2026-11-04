use std::{fs, path::PathBuf, process::Command, time::Duration};

use serde_json::json;
use tera::{Context, Tera};

use crate::lib::TimesReport;


pub const WORKLOADS: [&str; 3] = ["GET", "SET", "Mixed"];
pub const CLIENT_RUNS: [(u32, u32); 4] = [(1, 100_000), (10, 50_000), (50, 25_000), (100, 15_000)];

// ----- constants -----
const TITLE: &str = "Baseline: single store task";
const PLACEMENT: &str = "server and load generator on the same laptop";
const CONDITIONS: &str = "plugged in, other apps closed";
const SWEEPS: u32 = 3;
const PAUSE_SECS: u32 = 10;
const KEY_SPACE: u32 = 10_000;
const WARMUP: &str = "One warm-up run (mixed, 10 clients, 10,000 requests each) was discarded.";
const VALIDITY_RULE: &str = "This version of the load generator does not check replies for errors, so error counts are not reported.";
// ----------------------------------------------

fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1000.0
}

/// 52310 -> "52,310"
fn thousands(n: u64) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// Run shell command.
fn sh_raw(cmd: &str) -> Option<String> {
    let out = Command::new("sh").arg("-c").arg(cmd).output().ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn sh(cmd: &str) -> String {
    sh_raw(cmd).filter(|s| !s.is_empty()).unwrap_or_else(|| "unknown".into())
}

fn env_or_unknown(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| "unknown".into())
}

/// `sweeps[s][i]` = run i of sweep s (the raw results); `median[i]` = median of run i.
pub fn write_readme(sweeps: &[Vec<TimesReport>], median: &[TimesReport; 12]) -> anyhow::Result<PathBuf> {
    dotenv::dotenv().ok(); // so the server settings in .env can be read

    // ---- results table ----
    let rows: Vec<_> = (0..12)
        .map(|i| {
            let (clients, reqs) = CLIENT_RUNS[i % 4];
            let rps: Vec<u64> = sweeps.iter().map(|s| s[i].reqs_per_second).collect();
            let m = &median[i];
            json!({
                "workload": WORKLOADS[i / 4],
                "clients": clients,
                "requests_per_client": thousands(reqs as u64),
                "rps": thousands(m.reqs_per_second),
                "rps_min": thousands(*rps.iter().min().unwrap()),
                "rps_max": thousands(*rps.iter().max().unwrap()),
                "p50_ms": ms(m.p50),
                "p99_ms": ms(m.p99),
                "p99_9_ms": ms(m.p99dot9),
                "slowest_ms": ms(m.slowest),
                "errors": "not checked",
            })
        })
        .collect();

    // ---- every individual run ----
    let mut raw_runs = vec![];
    let mut csv = String::from("sweep,workload,clients,requests_per_client,requests_per_second,p50_ms,p99_ms,p99_9_ms,slowest_ms\n");
    for (s, sweep) in sweeps.iter().enumerate() {
        for (i, r) in sweep.iter().enumerate() {
            let (clients, reqs) = CLIENT_RUNS[i % 4];
            raw_runs.push(json!({
                "sweep": s + 1, "workload": WORKLOADS[i / 4], "clients": clients,
                "rps": thousands(r.reqs_per_second),
                "p50_ms": ms(r.p50), "p99_ms": ms(r.p99), "p99_9_ms": ms(r.p99dot9),
                "slowest_ms": ms(r.slowest), "errors": "not checked",
            }));
            csv.push_str(&format!(
                "{},{},{},{},{},{:.4},{:.4},{:.4},{:.4}\n",
                s + 1, WORKLOADS[i / 4], clients, reqs, r.reqs_per_second,
                ms(r.p50), ms(r.p99), ms(r.p99dot9), ms(r.slowest)
            ));
        }
    }

    // ---- headline: the 50-client row of each workload ----
    let k = CLIENT_RUNS.iter().position(|(c, _)| *c == 50).expect("no 50-client run configured");
    let (get, set, mix) = (&median[k], &median[4 + k], &median[8 + k]);

    // ---- version and environment ----
    let branch = sh("git rev-parse --abbrev-ref HEAD");
    let commit = sh("git rev-parse --short HEAD");
    let dirty = sh_raw("git status --porcelain --untracked-files=no").map(|s| !s.is_empty()).unwrap_or(false);
    let cpu = format!("{}, {} logical cores", sh("sysctl -n machdep.cpu.brand_string"), sh("sysctl -n hw.ncpu"));

    let mut ctx = Context::new();
    ctx.insert("meta", &json!({
        "title": TITLE,
        "branch": branch,
        "commit": commit,
        "commit_message": sh("git log -1 --pretty=%s"),
        "dirty": dirty,
        "date": sh("date '+%Y-%m-%d %H:%M'"),
        "build_profile": "release",
        "rustc": sh("rustc --version"),
        "repro_command": "cargo run --release --bin load-generator",
        "raw_csv_path": "runs.csv",
    }));
    ctx.insert("env", &json!({
        "machine": sh("sysctl -n hw.model"),
        "cpu": cpu,
        "ram": sh("echo $(( $(sysctl -n hw.memsize) / 1073741824 )) GB"),
        "os": sh("uname -sr"),
        "ulimit": sh("ulimit -n"),
        "placement": PLACEMENT,
        "conditions": CONDITIONS,
    }));
    ctx.insert("server", &json!({
        "connection_limit": env_or_unknown("CONNECTION_LIMIT"),
        "max_command_length": env_or_unknown("MAX_COMMAND_LENGTH"),
        "connection_timeout": env_or_unknown("CONNECTION_TIMEOUT"),
        "sweeper_frequency": env_or_unknown("SWEEPER_FREQUENCY"),
        "extra": Vec::<serde_json::Value>::new(),
    }));
    ctx.insert("method", &json!({
        "key_space": KEY_SPACE,
        "value_description": "SET values are random English words; the fill step before each run uses the fixed value `sample_val`.",
        "clients_list": CLIENT_RUNS.iter().map(|(c, _)| c.to_string()).collect::<Vec<_>>().join(", "),
        "sweeps": SWEEPS,
        "pause_secs": PAUSE_SECS,
        "warmup": WARMUP,
        "validity_rule": VALIDITY_RULE,
    }));
    ctx.insert("headline", &json!({
        "clients": 50,
        "get_rps": thousands(get.reqs_per_second),
        "set_rps": thousands(set.reqs_per_second),
        "mix_rps": thousands(mix.reqs_per_second),
        "mix_p99_ms": ms(mix.p99),
    }));
    ctx.insert("rows", &rows);
    ctx.insert("raw_runs", &raw_runs);
    ctx.insert("summary", "");                              // write a sentence or two here by hand
    ctx.insert("notes", &Vec::<String>::new());             // observations, by hand
    ctx.insert("extra_caveats", &Vec::<String>::new());
    // "memory" and "compare" are not inserted, so those sections are skipped

    let text = Tera::one_off(include_str!("benchmark_report.md.tera"), &ctx, false)?;

    // ---- write benchmarks/<branch>/<commit>[-dirty]/ ----
    let mut dir = PathBuf::from("benchmarks");
    dir.push(branch.replace('/', "-"));
    dir.push(format!("{}{}", commit, if dirty { "-dirty" } else { "" }));
    fs::create_dir_all(&dir)?;
    fs::write(dir.join("README.md"), text)?;
    fs::write(dir.join("runs.csv"), csv)?;
    Ok(dir)
}
