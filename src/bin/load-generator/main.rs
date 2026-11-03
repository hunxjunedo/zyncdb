pub mod lib;
mod commands;
mod client;
use std::{ sync::Arc};
use tokio::{spawn, sync::Barrier, time::Instant};

use tokio_util::{
    sync::CancellationToken,
};

use crate::{client::client, commands::pre_fill_keys, lib::{LoadType, TimesReport, graceful_shutdown_register, print_stats}};

#[tokio::main]
async fn main() {
    let requests_to_send = 25000;
    let token = CancellationToken::new();
    spawn(graceful_shutdown_register(token.clone()));
    load_generator(requests_to_send, 50, LoadType::Mix, token).await;
}


async fn load_generator(
    requests_to_send: u32,
    clients_to_spawn: u32,
    load_type: LoadType,
    token: CancellationToken,
) -> TimesReport {
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
    return time_report;
}

