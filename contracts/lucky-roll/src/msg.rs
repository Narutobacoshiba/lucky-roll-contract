use cosmwasm_schema::{cw_serde};
use nois::{NoisCallback};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::state::{
    Attendee, DistributePrize,
};

/// Message type for `instantiate` entry_point
#[cw_serde]
pub struct InstantiateMsg {
    pub nois_proxy: String,
    pub time_start: String,
    pub time_end: String,
}

/// Message type for `execute` entry_point
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Reset {
        nois_proxy: String,
        time_start: String,
        time_end: String,
    },

    SetWhiteList {
        attendees: Vec<String>
    },

    SetPrizes {
        prizes: Vec<String>
    },

    Roll {},

    NoisReceive { callback: NoisCallback },

    LuckyNumber {},
}


/// Message type for `query` entry_point
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetPrizes{},
    GetDistributePrizes{},
    GetAttendees{},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AttendeeQuery {
    pub attendees: Vec<Attendee>
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PrizesQuery {
    pub prizes: Vec<String>
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DistributePrizesQuery {
    pub prizes: Vec<DistributePrize>
}

