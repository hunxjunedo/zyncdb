use std::time::Duration;
pub enum LoadType {
    Mix,
    Get,
    Set,
}

pub struct TimesReport {
    pub p50: Duration,
    pub p99: Duration,
    pub p99dot9: Duration,
    pub reqs_per_second: u64,
    pub slowest: Duration,
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