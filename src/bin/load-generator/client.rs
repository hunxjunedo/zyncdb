use std::{sync::Arc, time::Duration};

use tokio::{net::TcpStream, sync::Barrier, time::Instant, io::AsyncWriteExt};
use tokio_util::{codec::{Framed, LinesCodec}, sync::CancellationToken};
use tokio_stream::StreamExt;

use crate::{commands::commands_generator, lib::LoadType};

pub async fn client(requests_to_send: u32, token: CancellationToken, barrier: Arc<Barrier>, load_type: LoadType) -> Vec<Duration> {
    //if cancel is called whilst a client is running, it sends only the times it already has, works fine.
    //but what if cancel was already called, even before a client is run ?
    //still not a problem since we will judge based on only the clients that were able to produce some times
    //but what if cancel was called before any of the clients in the load could generate any times ? it would result in an empty vec, and have issues with calculation
    //solution: when a cancel is called, exit the current load with whatever times you have available, but dont just exit, move to the next load!
    let mut times = vec![];
    //also generate the commands beforehand
    let commands = commands_generator(requests_to_send, load_type);
    //wait for the barrier to open before sending commands
    let stream = TcpStream::connect("127.0.0.1:8080").await.unwrap();
    let mut framed = Framed::new(stream, LinesCodec::new());
    barrier.wait().await;
    for command in commands {
        if token.is_cancelled() {
            break;
        }
        let start = Instant::now();
        _ = framed.get_mut().write_all(&command).await;
        let _ = framed.next().await.unwrap().unwrap();
        times.push(start.elapsed());
    }
    return times;
}


