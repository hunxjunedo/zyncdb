use std::{sync::Arc, time::Duration};

use random_word::Lang;
use tokio::{net::TcpStream, sync::Barrier, time::Instant, io::AsyncWriteExt};
use tokio_util::{codec::{Framed, LinesCodec}, sync::CancellationToken};
use tokio_stream::StreamExt;

pub async fn mix_client(requests_to_send: u32, token: CancellationToken, barrier: Arc<Barrier>) -> Vec<Duration> {
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
