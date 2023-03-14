use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr,Timestamp,Storage,StdResult,Uint128};
use cw_storage_plus::{Item, Map, Deque};

#[cw_serde]
pub struct DataRequest {
    pub min: i32,
    pub max: i32,
    pub num: u32,
    pub data_type: String,
}

#[cw_serde]
pub struct Commitment {
    pub id: String,
    pub request_id: String,
    pub owner: Addr,
    pub commit_time: Timestamp,
    pub expired_time: Timestamp,
    pub data_request: DataRequest,
}

pub const COMMITMENTS: Deque<Commitment> = Deque::new("commitments"); // a list of commitments ordered by commit time that is used to determine which commits satisfy the time conditions
pub const PENDING_COMMITMENTS: Map<String, Commitment> = Map::new("pending commitments"); // map of commitments, use for getting commitment's information

/// get commitments that meet time conditions 
///      commit_time <= completion_time <= expired_time
pub fn get_commitments(
    storage: &mut dyn Storage,
    completion_time: Timestamp,
    max_callback: u32,
) -> StdResult<Vec<Commitment>> {
    let mut count: u32 = 0;
    let mut vecs: Vec<Commitment> = Vec::new();
    loop {
        // pop commitment from queue
        let commitment = COMMITMENTS.pop_back(storage)?;

        if commitment.is_none(){
            break;
        }

        let commitment = commitment.unwrap();

        // if meet a commitment has `commit_time` greater than `completion_time`, 
        // push it back to queue and return current list
        if commitment.commit_time.ge(&completion_time) {
            COMMITMENTS.push_back(storage, &commitment)?;
            break;
        }

        // if meet a commitment has expired time less than completion time, continue
        if commitment.expired_time.lt(&completion_time) {
            continue;
        }

        vecs.push(commitment.clone());

        PENDING_COMMITMENTS.remove(storage, commitment.id);

        count += 1;
        if count >= max_callback {
            break;
        }
    }

    return Ok(vecs);
}

/// get commitment from PENDING_COMMIMENTS by id
pub fn get_commitment(
    storage: &mut dyn Storage,
    commit_id: String,
) -> StdResult<Option<Commitment>> {

    if PENDING_COMMITMENTS.has(storage, commit_id.clone()) {
        let commitment = PENDING_COMMITMENTS.load(storage, commit_id.clone())?;
        PENDING_COMMITMENTS.remove(storage, commit_id);

        return Ok(Some(commitment));
    }

    return Ok(None);
}


#[cw_serde]
pub struct Bot {
    pub address: Addr,
    pub hashed_api_key: String, // hash of random-org api key
    pub moniker: String,
    pub last_update: Timestamp,
}

pub const BOTS: Map<Addr, Bot> = Map::new("bots");

#[cw_serde]
pub struct NoisConfigs {
    pub nois_proxy: Addr,
    pub nois_fee: Uint128,
}

pub const NOIS_CONFIGS: Item<NoisConfigs> = Item::new("nois configs");

#[cw_serde]
pub struct TimeConfigs {
    pub time_expired: u64, // second
    pub time_per_block: u64, // second
}

pub const TIME_CONFIGS: Item<TimeConfigs> = Item::new("time configs");

#[cw_serde]
pub struct Configs {
    pub bounty_denom: String,
    pub fee: Uint128,
    pub callback_limit_gas: u64,
    pub max_callback: u32, 
}

pub const CONFIGS: Item<Configs> = Item::new("configs");

pub const OWNER: Item<Addr> = Item::new("owner");
pub const NONCES: Map<Addr, u64> = Map::new("nonces");

#[cfg(test)]
mod unit_tests {
    use super::*;
    use cosmwasm_std::testing::{
        mock_dependencies, MockApi, MockQuerier, MockStorage,
    };
    use cosmwasm_std::OwnedDeps;

    const INT_DATA_TYPE: &str = "int";

    const OWNER: &str = "owner";
    
    fn add_commitments(deps: &mut OwnedDeps<MockStorage, MockApi, MockQuerier>, commit_id: String, commit_time: u64, expired_time: u64) {
        let commitment: Commitment = Commitment {
            id: commit_id.clone(),
            request_id: String::from("request id"),
            owner: Addr::unchecked(OWNER),
            commit_time: Timestamp::from_seconds(commit_time),
            expired_time: Timestamp::from_seconds(expired_time),
            data_request: DataRequest{
                min: 0,
                max: 255,
                num: 32,
                data_type: String::from(INT_DATA_TYPE),
            },
        };
        COMMITMENTS.push_front(&mut deps.storage, &commitment).unwrap();
        PENDING_COMMITMENTS.save(&mut deps.storage, commit_id, &commitment).unwrap();
    }

    #[test]
    fn get_commitments_success() {
        let mut deps = mock_dependencies();
        
        add_commitments(&mut deps, String::from("1"), 0u64, 5u64);

        let completion_time: Timestamp = Timestamp::from_seconds(4);
        let commitments = get_commitments(&mut deps.storage, completion_time, 5u32).unwrap();

        assert_eq!(commitments.len(), 1);
        assert_eq!(COMMITMENTS.is_empty(&mut deps.storage).unwrap(), true);
        assert_eq!(PENDING_COMMITMENTS.is_empty(&mut deps.storage), true);
    }

    #[test]
    fn get_commitments_success_with_expired_commitment() {
        let mut deps = mock_dependencies();

        add_commitments(&mut deps, String::from("1"), 0u64, 5u64);
        add_commitments(&mut deps, String::from("2"), 5u64, 10u64);

        let completion_time: Timestamp = Timestamp::from_seconds(6);
        let commitments = get_commitments(&mut deps.storage, completion_time, 5u32).unwrap();

        assert_eq!(commitments.len(), 1);
        assert_eq!(COMMITMENTS.is_empty(&mut deps.storage).unwrap(), true);
        assert_eq!(PENDING_COMMITMENTS.is_empty(&mut deps.storage), false);
    }

    #[test]
    fn get_commitments_success_with_large_number_of_commitment() {
        let mut deps = mock_dependencies();

        add_commitments(&mut deps, String::from("1"), 0u64, 5u64);
        add_commitments(&mut deps, String::from("2"), 0u64, 5u64);
        add_commitments(&mut deps, String::from("3"), 0u64, 5u64);
        add_commitments(&mut deps, String::from("4"), 0u64, 5u64);
        add_commitments(&mut deps, String::from("5"), 0u64, 5u64);
        add_commitments(&mut deps, String::from("6"), 0u64, 5u64);

        let completion_time: Timestamp = Timestamp::from_seconds(4);
        let commitments = get_commitments(&mut deps.storage, completion_time, 5u32).unwrap();

        assert_eq!(commitments.len(), 5);
        assert_eq!(COMMITMENTS.len(&mut deps.storage).unwrap(), 1);
        assert_eq!(PENDING_COMMITMENTS.is_empty(&mut deps.storage), false);
    }

    #[test]
    fn get_commitments_success_with_future_commitment() {
        let mut deps = mock_dependencies();

        add_commitments(&mut deps, String::from("1"), 0u64, 5u64);
        add_commitments(&mut deps, String::from("2"), 0u64, 5u64);
        add_commitments(&mut deps, String::from("3"), 0u64, 5u64);
        add_commitments(&mut deps, String::from("4"), 0u64, 5u64);
        add_commitments(&mut deps, String::from("5"), 5u64, 10u64);

        let completion_time: Timestamp = Timestamp::from_seconds(4);
        let commitments = get_commitments(&mut deps.storage, completion_time, 5u32).unwrap();

        assert_eq!(commitments.len(), 4);
        assert_eq!(COMMITMENTS.len(&mut deps.storage).unwrap(), 1);
        assert_eq!(PENDING_COMMITMENTS.is_empty(&mut deps.storage), false);
    }


    #[test]
    fn get_commitment_success() {
        let mut deps = mock_dependencies();

        let commit_id: String = String::from("test id");
        let commitment: Commitment = Commitment {
            id: commit_id.clone(),
            request_id: String::from("request id"),
            owner: Addr::unchecked(OWNER),
            commit_time: Timestamp::from_seconds(0),
            expired_time: Timestamp::from_seconds(5),
            data_request: DataRequest{
                min: 0,
                max: 255,
                num: 32,
                data_type: String::from(INT_DATA_TYPE),
            },
        };
        COMMITMENTS.push_back(&mut deps.storage, &commitment).unwrap();
        PENDING_COMMITMENTS.save(&mut deps.storage, commit_id.clone(), &commitment).unwrap();

        let get_commitment = get_commitment(&mut deps.storage, commit_id).unwrap();

        assert_eq!(get_commitment.is_some(), true);
        assert_eq!(COMMITMENTS.is_empty(&mut deps.storage).unwrap(), false);
        assert_eq!(PENDING_COMMITMENTS.is_empty(&mut deps.storage), true);
    }

}
