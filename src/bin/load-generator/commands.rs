use rand::RngExt;
use random_word::Lang;

use crate::lib::LoadType;

fn random_key() -> String {
    return format!("k{}", rand::rng().random_range(0..=9999));
}

pub fn commands_generator(requests_to_send: u32, load_type: LoadType) -> Vec<Vec<u8>>{
        let mut last_field_name: Option<String> = None;
        let commands: Vec<Vec<u8>> = (0..requests_to_send)
        .map(|i| {
            if i % 2 != 0 {
                //get
                let command = format!("GET {}\n", &last_field_name.clone().unwrap());
                let byte_command = command.into_bytes();
                return byte_command;

            } else {
                //limit uniqueness to 10k unique keys
                let random_field_name = random_key();
                let random_name = random_word::get(Lang::En);
                let command: String = format!("SET {} {}\n", random_field_name, random_name);
                last_field_name = Some(random_field_name);
                let byte_command = command.clone().into_bytes();
                return byte_command;
            }
        })
        .collect();
    return commands;
}