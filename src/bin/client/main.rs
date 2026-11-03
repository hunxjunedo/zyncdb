use core::time;
use std::time::Duration;

use rand::RngExt;
use random_word::Lang;
use tokio::{io::AsyncWriteExt, net::TcpStream, signal, spawn, time::Instant};
use tokio_stream::StreamExt;
use tokio_util::{codec::{Framed, LinesCodec}, sync::CancellationToken};

#[tokio::main]
async fn main() {
    let requests_to_send = 25000;
    let token = CancellationToken::new();
    spawn(graceful_shutdown_register(token.clone()));
    load_generator(requests_to_send, 50, LoadType::Mix, token).await;

    
}

async fn graceful_shutdown_register(token: CancellationToken){
        match signal::ctrl_c().await {
        Ok(()) => {token.cancel();},
        Err(err) => {
            eprintln!("Unable to listen for shutdown signal: {}", err);
            // we also shut down in case of error
        },
    }
}

enum LoadType {
    Mix,
    Get,
    Set
}

struct TimesReport {
    p50: Duration,
    p99: Duration,
    p99dot9: Duration,
    reqs_per_second: f64,
    slowest: Duration
}

async fn load_generator(requests_to_send: u32, clients_to_spawn: u32, load_type: LoadType, token: CancellationToken) -> TimesReport {
        let mut n  = 0;
        let mut handles = vec![];
        let mut times = vec![];
        
        loop{
        if n >= clients_to_spawn {
            break;
        }
        let token_to_pass = token.clone();
        let handle = tokio::spawn(match load_type {
            LoadType::Mix => {
                mix_client(requests_to_send, token_to_pass)
            },
            _ => {
                mix_client(requests_to_send, token_to_pass)
            }
        });
        handles.push(handle);
        n+=1;
    }
    for handle in handles {
        let mut client_times = handle.await.unwrap();
        times.append(&mut client_times);
    };
    println!("TEST REPORT");
    match load_type {
        LoadType::Mix => {
            println!("TEST TYPE: MIX")
        },
        _ => {
            println!("TEST TYPE: MIX")
        }
    };
    println!("CLIENTS USED: {clients_to_spawn}");
    let reqs_count = times.len();
    println!("TOTAL REQUESTS SENT: {}", reqs_count);
    times.sort();
    let total_time = times.iter().sum::<Duration>();
    let pick = |p: f64| times[((reqs_count as f64 * p) as usize).min(reqs_count - 1)];
    let time_report = TimesReport {
        p50: pick(0.50),
        p99: pick(0.99),
        p99dot9: pick(0.999),
        reqs_per_second: reqs_count as f64/total_time.as_secs_f64(),
        slowest: times[reqs_count - 1]
    };
    println!("p50: {:?}", time_report.p50);
    println!("p99: {:?}", time_report.p99);
    println!("p99.9: {:?}", time_report.p99dot9);
    println!("requests per second: {}", time_report.reqs_per_second);
    println!("slowest: {:?}", time_report.slowest);
    
    return time_report;

}

async fn mix_client(requests_to_send: u32, token: CancellationToken) -> Vec<Duration>{
    let mut stream = TcpStream::connect("127.0.0.1:8080").await.unwrap();
    let mut requests_sent = 0;
    let mut last_field_name:Option<String> = None;
    let mut framed = Framed::new(stream, LinesCodec::new());
    let mut times = vec![];
    loop {
        if requests_sent > requests_to_send || token.is_cancelled(){
            break;
        }
        if requests_sent % 2 != 0 {
            //get
            let command = format!("GET {}\n", &last_field_name.clone().unwrap());
            let byte_command = command.into_bytes();
            let start = Instant::now(); 
            framed.get_mut().write(&byte_command).await;
            let response = framed.next().await.unwrap().unwrap();
            times.push(start.elapsed());
            // println!("value: {}", response)
        }else{
            let random_field_name = format!("{}{}", random_word::get(Lang::En), requests_sent);
            let random_name = random_word::get(Lang::En);
            let command = format!("SET {} {}\n", random_field_name, random_name);
            last_field_name = Some(random_field_name);
            let byte_command = command.clone().into_bytes();
            let start = Instant::now(); 
            let _ = framed.get_mut().write(&byte_command).await.unwrap();
            let _response = framed.next().await.unwrap().unwrap();
            times.push(start.elapsed());
            // println!("SET response: {}", response)
            //set
        }
        requests_sent+=1;
    }
    println!("CLIENT COMPLETE with times: {}", times.len());

    return times;
}