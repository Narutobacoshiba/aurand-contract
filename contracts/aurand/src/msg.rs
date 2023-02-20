use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Uint128, Timestamp};
use nois::NoisCallback;
use crate::state::Commitment;

/// Message type for `instantiate` entry_point
#[cw_serde]
pub struct InstantiateMsg {
    pub nois_proxy: String, 
    pub time_expired: u64, //second
    pub time_per_block: u64, //second
    pub bounty_denom: String,
    pub fee: Uint128,
    pub nois_fee: Uint128,
    pub callback_limit_gas: u64,
    pub max_callback: u32,
}

/// Message type for `execute` entry_point
#[cw_serde]
pub enum ExecuteMsg {
    // set contract configs
    SetConfigs {
        bounty_denom: String, // denom string, ex: "ueaura"
        fee: Uint128, // fee of each random request
        callback_limit_gas: u64, // limmit gas of callback call for each request  
        max_callback: u32, // max number of callback submessage in each bot add randomness message
    },

    // set nois configs
    SetNoisConfigs {
        nois_proxy: String, // addr of nois proxy contract on aura chain
        nois_fee: Uint128, // fee that nois proxy contract requires for each call
    },

    // set time conditions for commitments
    SetTimeConfigs {
        time_expired: u64, // lifetime of commitments (seconds), ex: 5s
        time_per_block: u64,  // time for block creation on aura chain (seconds), currently 5s/block
    },

    // sign up bot for adding randomness and claiming reward
    RegisterBot {
        hashed_api_key: String, // hash of random org api key
        moniker: String, // bot name
    },
    
    // update bot information
    UpdateBot {
        hashed_api_key: String, // hash of random org api key
        moniker: String, // bot name
    },

    // owner remove bot from contract
    RemoveBot {
        address: String // addr of bot
    },

    // user request for hex randomness
    RequestHexRandomness{
        request_id: String, // id of request
        num: u32 // number of wanted randomness 
    },

    // user request for integer randomness
    RequestIntRandomness{
        request_id: String, // id of request
        min: i32, // min value of each randomness
        max: i32, // max valud of each randomness
        num: u32, // number of wanted randomness
    },
    
    // bot add randomness from random org
    AddRandomness{
        random_value: String, // random value return from random org
        signature: String // signature of random value, signed by random org. Public key https://api.random.org/server.crt
    },

    // catch nois proxy callback for receiving randomness from nois network
    NoisReceive {
        callback: NoisCallback // NoisCallback {job_id,randomness}
    },
}

/// Message type for `query` entry_point
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(PendingCommitmentsQuery)]
    GetPendingCommitments{limit: u32}, // get list of pending commitments

    #[returns(CommitmentsQuery)]
    GetCommitments{limit: u32}, // get list of commitments

    #[returns(NumberOfCommitmentQuery)]
    GetNumberOfCommitment{}, // get curent number of commitments

    #[returns(BotInfoQuery)]
    GetBotInfo{address: String}, // get bot information by address

    #[returns(ConfigsQuery)]
    GetConfigs{}, // get all contract configs
}

#[cw_serde]
pub struct PendingCommitmentsQuery {
    pub commitments: Vec<Commitment>
}

#[cw_serde]
pub struct CommitmentsQuery {
    pub commitments: Vec<Commitment>
}

#[cw_serde]
pub struct NumberOfCommitmentQuery {
    pub num: u32
}

#[cw_serde]
pub struct BotInfoQuery {
    pub address: String,
    pub hashed_api_key: String, // hash of random-org api key
    pub moniker: String,
    pub last_update: Timestamp,
}

#[cw_serde]
pub struct ConfigsQuery {
    pub nois_proxy: String,
    pub time_expired: u64, //second
    pub time_per_block: u64, //second
    pub bounty_denom: String,
    pub fee: Uint128,
    pub nois_fee: Uint128,
    pub callback_limit_gas: u64,
}

// callback function that user must define in contract for receiving aurand randomness
#[cw_serde]
pub enum CallbackExecuteMsg {
    ReceiveHexRandomness{
        request_id: String,
        randomness: Vec<String>
    },

    ReceiveIntRandomness{
        request_id: String,
        randomness: Vec<i32>
    },
}