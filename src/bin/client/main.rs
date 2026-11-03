use core::time;
use std::{ sync::Arc, time::Duration};

use random_word::Lang;
use tokio::{io::AsyncWriteExt, net::TcpStream, signal, spawn, sync::Barrier, time::Instant};
use tokio_stream::StreamExt;
use tokio_util::{
    codec::{Framed, LinesCodec},
    sync::CancellationToken,
};

#[tokio::main]
async fn main() {
    let requests_to_send = 25000;
    let token = CancellationToken::new();
    spawn(graceful_shutdown_register(token.clone()));
    load_generator(requests_to_send, 50, LoadType::Mix, token).await;
}

async fn graceful_shutdown_register(token: CancellationToken) {
    match signal::ctrl_c().await {
        Ok(()) => {
            token.cancel();
        }
        Err(err) => {
            eprintln!("Unable to listen for shutdown signal: {}", err);
            // we also shut down in case of error
        }
    }
}

enum LoadType {
    Mix,
    Get,
    Set,
}

struct TimesReport {
    p50: Duration,
    p99: Duration,
    p99dot9: Duration,
    reqs_per_second: u64,
    slowest: Duration,
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
    loop {
        if n >= clients_to_spawn {
            break;
        }
        let token_to_pass = token.clone();
        let barrier_to_pass = Arc::clone(&barrier);
        let handle = tokio::spawn(match load_type {
            LoadType::Mix => mix_client(requests_to_send, token_to_pass, barrier_to_pass),
            _ => mix_client(requests_to_send, token_to_pass, barrier_to_pass),
        });
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
    println!("TEST REPORT");
    match load_type {
        LoadType::Mix => {
            println!("TEST TYPE: MIX")
        }
        _ => {
            println!("TEST TYPE: MIX")
        }
    };
    println!("CLIENTS USED: {clients_to_spawn}");
    let reqs_count = times.len();
    println!("TOTAL REQUESTS SENT: {}", reqs_count);
    times.sort();
    let pick = |p: f64| times[((reqs_count as f64 * p) as usize).min(reqs_count - 1)];
    let time_report = TimesReport {
        p50: pick(0.50),
        p99: pick(0.99),
        p99dot9: pick(0.999),
        reqs_per_second: (reqs_count as f64 / elapsed.as_secs_f64()).round() as u64,
        slowest: times[reqs_count - 1],
    };
    println!("p50: {:?}", time_report.p50);
    println!("p99: {:?}", time_report.p99);
    println!("p99.9: {:?}", time_report.p99dot9);
    println!("requests per second: {}", time_report.reqs_per_second);
    println!("slowest: {:?}", time_report.slowest);

    return time_report;
}

async fn mix_client(requests_to_send: u32, token: CancellationToken, barrier: Arc<Barrier>) -> Vec<Duration> {
    let stream = TcpStream::connect("127.0.0.1:8080").await.unwrap();
    let mut last_field_name: Option<String> = None;
    let mut framed = Framed::new(stream, LinesCodec::new());
    let mut times = vec![];
    //also generate the commands beforehand
    let commands: Vec<Vec<u8>> = (0..requests_to_send)
        .map(|i| {
            if i % 2 != 0 {
                //get
                let command = format!("GET {}\n", &last_field_name.clone().unwrap());
                let byte_command = command.into_bytes();
                return byte_command;

            } else {
                let random_field_name = format!("{}{}", random_word::get(Lang::En), i);
                let random_name = random_word::get(Lang::En);
                let command: String = format!("SET {} {}\n", random_field_name, random_name);
                last_field_name = Some(random_field_name);
                let byte_command = command.clone().into_bytes();
                return byte_command;
            }
        })
        .collect();
    //wait for the barrier to open before sending commands
    barrier.wait().await;
    for command in commands {
        if token.is_cancelled() {
            break;
        }
        let start = Instant::now();
        _ = framed.get_mut().write(&command).await;
        let _ = framed.next().await.unwrap().unwrap();
        times.push(start.elapsed());
    }
    return times;
}
