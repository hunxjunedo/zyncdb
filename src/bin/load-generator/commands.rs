

use rand::RngExt;
use random_word::Lang;
use tokio::{io::AsyncWriteExt, net::TcpStream};
use tokio_stream::StreamExt;
use tokio_util::codec::{Framed, LinesCodec};

use crate::lib::LoadType;

fn random_key() -> String {
    //returns a random key in the range k0 - k9999, it is guaranteed that the key will pre-exist on the server
    return format!("k{}", rand::rng().random_range(0..=9999));
}

pub fn commands_generator(requests_to_send: u32, load_type: LoadType) -> Vec<Vec<u8>> {
    let mut last_field_name: Option<String> = None;
    let commands: Vec<Vec<u8>> = (0..requests_to_send)
        .map(|i| {
            match load_type {
                LoadType::Mix => {
                    if i % 2 != 0 {
                        //get
                        let command = format!("GET {}\n", &last_field_name.clone().unwrap());
                        let byte_command = command.into_bytes();
                        return byte_command;
                    } else {
                        //limit uniqueness to 10k unique keys
                        let random_field_name = random_key();
                        let random_name = random_word::get(Lang::En);
                        let command: String =
                            format!("SET {} {}\n", random_field_name, random_name);
                        last_field_name = Some(random_field_name);
                        let byte_command = command.clone().into_bytes();
                        return byte_command;
                    }
                }
                LoadType::Set => {
                    //limit uniqueness to 10k unique keys
                    let random_field_name = random_key();
                    let random_name = random_word::get(Lang::En);
                    let command: String = format!("SET {} {}\n", random_field_name, random_name);
                    let byte_command = command.clone().into_bytes();
                    return byte_command;
                }
                LoadType::Get => {
                    //get
                    let command = format!("GET {}\n", random_key());
                    let byte_command = command.into_bytes();
                    return byte_command;
                }
            }
        })
        .collect();
    return commands;
}


pub async fn pre_fill_keys(addr: &str, key_space: usize, value: &str) {
    let stream = TcpStream::connect(addr).await.unwrap();
    let mut framed = Framed::new(stream, LinesCodec::new());
    for i in 0..key_space {
        let command = format!("SET k{i} {value}\n");
        framed.get_mut().write_all(command.as_bytes()).await.unwrap();
        framed.next().await.expect("server closed").unwrap();   // wait for the reply
    }
}