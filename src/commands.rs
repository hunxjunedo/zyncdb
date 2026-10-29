use std::sync::Arc;

use anyhow::{Context, Result, anyhow};
use tokio::time::Instant;

use crate::{
    db::{Database, DbHandle},
    db_operations::{DbOperation, GetKey, RemoveKey, SetEvictionPolicy, SetKey},
    expiry,
};

//the user facing commands
//for now just make them return a db command, until we get to more complex commands.
//todo: make Command recieve the multi-sender, so that it has more control.
pub trait Command {
    fn execute(&self) -> DbOp;
    fn from_string(raw: String) -> Result<Box<dyn Command + Send>>
    where
        Self: Sized;
}

type DbOp = Box<dyn DbOperation + Send>;

pub struct Get {
    pub field: String,
}
pub struct Set {
    pub field: String,
    pub data: String,
    pub ttl: Option<u64>,
}
pub struct Del {
    pub field: String,
}

impl Command for Get {
    fn execute(&self) -> DbOp {
        Box::from(GetKey {
            key: self.field.clone(),
        })
    }

    fn from_string(raw: String) -> Result<Box<dyn Command + Send>>
    where
        Self: Sized,
    {
        let chunks: Vec<&str> = raw.split(' ').collect();
        if chunks.len() < 1 {
            return Err(anyhow!("field name not provided"));
        };
        return Ok(Box::new(Get {
            field: chunks[0].to_string(),
        }));
    }
}

impl Command for Set {
    fn execute(&self) -> DbOp {
        Box::from(SetKey {
            key: self.field.clone(),
            value: self.data.clone(),
            ttl: self.ttl,
        })
    }
    fn from_string(raw: String) -> Result<Box<dyn Command + Send>>
    where
        Self: Sized,
    {
        let chunks: Vec<&str> = raw.split(' ').collect();
        if chunks.len() < 2 {
            return Err(anyhow!("required arguments not provided"));
        };
        //parse the ttl
        let mut ttl: Option<u64> = None;
        if chunks.len() == 3
            && let parsed_result = chunks[2].parse::<u64>()
        {
            ttl = Some(parsed_result.context("TTL should be an unsigned 64-bit integer")?);
        }

        return Ok(Box::new(Set {
            field: chunks[0].to_string(),
            data: chunks[1].to_string(),
            ttl,
        }));
    }
}
impl Command for Del {
    fn execute(&self) -> DbOp {
        Box::from(RemoveKey {
            key: self.field.clone(),
        })
    }
    fn from_string(raw: String) -> Result<Box<dyn Command + Send>>
    where
        Self: Sized,
    {
        let chunks: Vec<&str> = raw.split(' ').collect();
        if chunks.len() < 1 {
            return Err(anyhow!("field name not provided"));
        };
        return Ok(Box::new(Del {
            field: chunks[0].to_string(),
        }));
    }
}

pub struct Expire {
    pub field: String,
    pub ttl: u64,
}

impl Command for Expire {
    fn execute(&self) -> DbOp {
        Box::from(SetEvictionPolicy {
            key: self.field.clone(),
            ttl: self.ttl.clone(),
        })
    }
    fn from_string(raw: String) -> Result<Box<dyn Command + Send>>
    where
        Self: Sized,
    {
        let chunks: Vec<&str> = raw.split(' ').collect();
        if chunks.len() < 2 {
            return Err(anyhow!("required arguments not provided"));
        };
        let ttl = chunks[1]
            .parse::<u64>()
            .context("TTL should be an unsigned 64-bit integer")?;
        return Ok(Box::new(Expire {
            field: chunks[0].to_string(),
            ttl,
        }));
    }
}
