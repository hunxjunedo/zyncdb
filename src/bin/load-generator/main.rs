pub mod lib;
mod commands;
mod client;
use core::time;
use std::{ sync::Arc, thread, time::Duration};
use tokio::{spawn, sync::Barrier, time::Instant};
use crate::lib::{LoadType::{Get, Mix, Set}, get_median_time_figures};

use tokio_util::{
    sync::CancellationToken,
};

use crate::{client::client, commands::pre_fill_keys, lib::{LoadType, TimesReport, graceful_shutdown_register, print_stats}};

//tasks for tomorrow:
// 1 client: 100000
// 10 client: 50000
// 50 client: 25000
// 100 client: 15000
// record in a readme
// Key space: 10,000 keys, in all runs.
// Warm-up: one throwaway run first (for example mixed, 10 clients) and discard it.
// Repeats: run the whole sweep 3 times and keep the median run for each row. 3 workloads × 4 sizes is 12 runs, about 5 to 10 minutes per sweep.
// A pause of about 10 seconds between runs. It lets the previous connections close and gives the laptop a moment to cool. laptop slows down when hot.


#[tokio::main]
async fn main() {
    //todo: double check and make sure no errors reported
    let token = CancellationToken::new();
    spawn(graceful_shutdown_register(token.clone()));
    //run a warmup first
    if load_generator(10000, 10, LoadType::Mix, token.clone()).await.is_none() {
        return;
    }

    let mut sweep_results = vec![];
    for _ in 0..3 {
        thread::sleep(Duration::from_secs(10));
        //three sweeps
        if token.is_cancelled() {
            return;
        }
        let result = sweep(token.clone()).await;
        if result.is_none() {
            return;
        }
        let result = result.unwrap();
        sweep_results.push(result);
    }

    let median_results : [TimesReport; 12] = get_median_time_figures(sweep_results);
    println!("{:?}", median_results);
    
}

async fn sweep(token: CancellationToken) -> Option<Vec<TimesReport>>{
    //what if cancellation is call
    let mut times_reports = vec![];
for workload in [Get, Set, Mix] {
    for (clients, reqs) in [(1, 100_000), (10, 50_000), (50, 25_000), (100, 15_000)] {
        if token.is_cancelled() { println!("aborted; finished runs are in the results file"); return None; }
        match load_generator(reqs, clients, workload.clone(), token.clone()).await {
            Some(report) => {times_reports.push(report)},
            None => { println!("run interrupted, discarded"); return None; }
        }
    }
}

return Some(times_reports);
}


async fn load_generator(
    requests_to_send: u32,
    clients_to_spawn: u32,
    load_type: LoadType,
    token: CancellationToken,
) -> Option<TimesReport> {
    let mut n = 0;
    let mut handles = vec![];
    let mut times = vec![];
    let barrier = Arc::new(Barrier::new(clients_to_spawn as usize + 1)); //parent will also wait
    //before running any test, prefill the key-set. But why not do it once in main ? Every benchmark should be self-contained, completely independent
    pre_fill_keys("127.0.0.1:8080", 10000, "sample_val").await;
    loop {
        if n >= clients_to_spawn {
            break;
        }
        let token_to_pass = token.clone();
        let barrier_to_pass = Arc::clone(&barrier);
        let handle = tokio::spawn(client(requests_to_send, token_to_pass, barrier_to_pass, load_type.clone()));
        handles.push(handle);
        n += 1;
    }

    //all clients are waiting now, parent should also wait to open the barrier:
    barrier.wait().await;
    let start = Instant::now();
        for handle in handles {
            let mut client_times = handle.await.unwrap();
            times.append(&mut client_times);
        }
    let elapsed = start.elapsed();

    //done, print the stats.
    
    let reqs_count = times.len();
    if reqs_count == 0 {
        return None;
    }
    times.sort();
    let pick = |p: f64| times[((reqs_count as f64 * p) as usize).min(reqs_count - 1)];
    let time_report = TimesReport {
        p50: pick(0.50),
        p99: pick(0.99),
        p99dot9: pick(0.999),
        reqs_per_second: (reqs_count as f64 / elapsed.as_secs_f64()).round() as u64,
        slowest: times[reqs_count - 1],
    };
    
    print_stats(&time_report, load_type, clients_to_spawn, reqs_count);
    return Some(time_report);
}

