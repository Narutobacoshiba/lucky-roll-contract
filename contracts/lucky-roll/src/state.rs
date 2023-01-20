use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{Addr,Timestamp};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Configs {
    pub nois_proxy: Addr,
    pub time_start: Timestamp,
    pub time_end: Timestamp,
}
pub const CONFIGS: Item<Configs> = Item::new("configs");


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Attendee {
    pub address: Addr,
    pub lucky_number: [u8;32],
}
pub const ATTENDEE_LIST: Map<Addr, Attendee> = Map::new("attendee list");


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Status {
    pub attended: bool,
}
pub const WHITELIST: Map<Addr, Status> = Map::new("whitelist");


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DistributePrize {
    pub address: Addr,
    pub prize: String,
}
pub const DISTRIBUTE_PRIZES: Item<Vec<DistributePrize>> = Item::new("distribute prizes");


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Prizes {
    pub shuffle: bool,
    pub prizes: Vec<String>,
}
pub const PRIZES: Item<Prizes> = Item::new("prizes");

pub const OWNER: Item<Addr> = Item::new("owner");

pub const END_ROUND: Item<bool> = Item::new("end round");