use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Activity {
    pub id: u8,
    pub duration: u8,
    pub r0: u8,
    pub r1: u8,
    pub successors: Vec<u8>,
}
