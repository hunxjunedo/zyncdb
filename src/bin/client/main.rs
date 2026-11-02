use core::time;
use std::time::Duration;

use rand::RngExt;
use random_word::Lang;
use tokio::{io::{AsyncWriteExt}, net::TcpStream, time::Instant};
use tokio_stream::StreamExt;
use tokio_util::codec::{Framed, LinesCodec};

#[tokio::main]
async fn main() {
    //spawn n clients
    let mut n = 0;
    let mut handles = vec![];
    let mut times = vec![];
    loop{
        if n > 1000 {
            break;
        }
        let handle = tokio::spawn(client());
        handles.push(handle);
        n+=1;
    }
    for handle in handles {
        let mut client_times = handle.await.unwrap();
        times.append(&mut client_times);
    }

    times.sort();
    let pick = |p: f64| times[((times.len() as f64 * p) as usize).min(times.len() - 1)];
    println!("typical (median): {:?}", pick(0.50));
    println!("slowest 1% start at: {:?}", pick(0.99));
    println!("slowest: {:?}", times[times.len() - 1]);
    
}

fn random_ttl() -> u64 {
    rand::rng().random_range::<u64, _>(1..10)
}

async fn client() -> Vec<Duration>{
    let mut stream = TcpStream::connect("127.0.0.1:8080").await.unwrap();
    //send 1000 requests
    let mut requests_sent = 0;
    let mut last_field_name:Option<String> = None;
    let mut framed = Framed::new(stream, LinesCodec::new());
    let mut times = vec![];
    loop {
        if requests_sent > 1001{
            break;
        }
        if requests_sent % 2 != 0 {
            //get
            let command = format!("GET {}\n", &last_field_name.clone().unwrap());
            let start = Instant::now(); 
            framed.get_mut().write(&command.into_bytes()).await;
            let response = framed.next().await.unwrap().unwrap();
            times.push(start.elapsed());
            // println!("value: {}", response)
        }else{
            let random_field_name = format!("{}{}", random_word::get(Lang::En), requests_sent);
            let random_name = random_word::get(Lang::En);
            let random_ttl = random_ttl();
            let command = format!("SET {} {} {}\n", random_field_name, random_name, random_ttl);
            last_field_name = Some(random_field_name);
            let start = Instant::now(); 
            let _ = framed.get_mut().write(&command.clone().into_bytes()).await.unwrap();

            // println!("write completed for command: {}", command);
            let response = framed.next().await.unwrap().unwrap();
            times.push(start.elapsed());
            // println!("SET response: {}", response)
            //set
        }
        requests_sent+=1;
    }
    println!("CLIENT COMPLETE");

    return times;
}