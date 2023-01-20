use chrono::{DateTime, Local};
use cosmwasm_std::{Timestamp};
use sha2::{Sha256,Digest};

fn sha256_hash(string: &[u8]) -> [u8;32] {
    let mut hasher = Sha256::new();
    // write input message
    hasher.update(string);
    // read hash digest and consume hasher
    let result = hasher.finalize();

    let x: [u8; 32] = result.as_slice().try_into().expect("Wrong hash length");

    return x;
}

pub fn convert_datetime_string(data: String) -> Timestamp {
    let date_time = data.parse::<DateTime<Local>>().unwrap();
    return Timestamp::from_nanos(date_time.timestamp_nanos() as u64);
}

pub fn generate_true_randomness(address: String, randomness: String) -> [u8;32] {
    let seed = address + &randomness;
    return sha256_hash(seed.as_bytes());
}

