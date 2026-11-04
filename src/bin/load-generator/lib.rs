use std::{cmp, time::Duration};
use tokio::signal;
use tokio_util::sync::CancellationToken;
#[derive(Clone)]
pub enum LoadType {
    Mix,
    Get,
    Set,
}

#[derive(Debug)]
pub struct TimesReport {
    pub p50: Duration,
    pub p99: Duration,
    pub p99dot9: Duration,
    pub reqs_per_second: u64,
    pub slowest: Duration,
}

impl TimesReport {
    fn get_median_report(r1: &Self, r2: &Self, r3: &Self) -> Self {
        let mut p50s = [r1.p50, r2.p50, r3.p50];
        p50s.sort();
        let mut p99s = [r1.p99, r2.p99, r3.p99];
        p99s.sort();
        let mut p99dot9s = [r1.p99dot9, r2.p99dot9, r3.p99dot9];
        p99dot9s.sort();
        let mut reqs_per_seconds = [r1.reqs_per_second, r2.reqs_per_second, r3.reqs_per_second];
        reqs_per_seconds.sort();
        let mut slowests = [r1.slowest, r2.slowest, r3.slowest];
        slowests.sort();
        return Self {
            p50: p50s[1],
            p99: p99s[1],
            p99dot9: p99dot9s[1],
            reqs_per_second: reqs_per_seconds[1],
            slowest: slowests[2] //use the worst
        }

    }
}


impl LoadType {
    pub fn get_name_as_string(&self) -> String {
        match self {
            &LoadType::Get => "Get".into(),
            &LoadType::Set => "Set".into(),
            &LoadType::Mix => "Mix".into()
        }
    }
}

pub fn print_stats(stats: &TimesReport, load_type: LoadType, clients_to_spawn: u32, reqs_count: usize) {
    println!("TEST REPORT");
    println!("test type: {}", load_type.get_name_as_string());
    println!("clients used: {clients_to_spawn}");
    println!("total requests sent: {}", reqs_count);
    println!("p50: {:?}", stats.p50);
    println!("p99: {:?}", stats.p99);
    println!("p99.9: {:?}", stats.p99dot9);
    println!("requests per second: {}", stats.reqs_per_second);
    println!("slowest: {:?}", stats.slowest);
}


pub async fn graceful_shutdown_register(token: CancellationToken) {
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

pub fn get_median_time_figures(all_figures: Vec<Vec<TimesReport>>) -> [TimesReport; 12] {
    let mut iter_map =  (0..12).map(|i|{
        TimesReport::get_median_report(&all_figures[0][i], &all_figures[1][i], &all_figures[2][i])
    });
    std::array::from_fn(|_| iter_map.next().unwrap())
}