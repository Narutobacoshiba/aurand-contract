use crate::error::ContractError;
use sha2::{Sha256,Sha512,Digest};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Local};
use nois::{int_in_range, sub_randomness_with_key};
use cosmwasm_std::Timestamp;

// calculate sha256 hash value
pub fn sha256_hash(string: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    // write input message
    hasher.update(string);
    // read hash digest and consume hasher
    let result = hasher.finalize();

    return result.to_vec();
}

// calculate sha512 hash value
pub fn sha512_hash(string: &[u8]) -> Vec<u8> {
    let mut hasher = Sha512::new();
    // write input message
    hasher.update(string);
    // read hash digest and consume hasher
    let result = hasher.finalize();

    return result.to_vec();
}

// generate commitment id from user 's address and user's nonce
pub fn make_commit_id(address: String, nonce: u64) -> String{
    let seed = address + &nonce.to_string();
    return hex::encode(sha256_hash(seed.as_bytes()));
}

// using nois's tool box to generate list of hex randomness
pub fn generate_hex_randomness(randomness: [u8;32], job_id: String, num: u32) -> Vec<String> {
    let mut return_data: Vec<String> = Vec::new();

    // generate random supplier from seed and key using pseudo-random algorithm
    let mut provider = sub_randomness_with_key(randomness, job_id.clone());

    for _ in 0..num {
        // provide different randomness each time
        let sub_randomness = provider.provide();
        return_data.push(hex::encode(sub_randomness));
    }

    return return_data;
}

// using nois's tool box to generate list of integer randomness
pub fn generate_int_randomness(randomness: [u8;32], job_id: String, min: i32, max: i32, num: u32) -> Vec<i32> {
    let mut return_data: Vec<i32> = Vec::new();

    // generate random supplier from seed and key using pseudo-random algorithm
    let mut provider = sub_randomness_with_key(randomness, job_id.clone());

    for _ in 0..num {
        // provide different randomness each time
        let sub_randomness = provider.provide();
        // randomly generate integers in range (min, max) using pseudo-random algorithm and seed
        let int_randomness = int_in_range(sub_randomness, min, max);

        return_data.push(int_randomness);
    }

    return return_data;
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug,)]
#[allow(non_snake_case)]
pub struct License {
    pub r#type: String,
    pub text: String,
    pub infoUrl: Option<String>,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug,)]
#[allow(non_snake_case)]
pub struct RandomOrgData {
    pub method: String,
    pub hashedApiKey: String,
    pub n: u32,
    pub min: u32,
    pub max: u32,
    pub replacement: bool,
    pub base: u32,
    pub pregeneratedRandomization: Option<String>,
    pub data: [u8; 32],
    pub license: License,
    pub licenseData: Option<String>,
    pub userData: Option<String>,
    pub ticketData: Option<String>,
    pub completionTime: String,
    pub serialNumber: u32,
}

// convert string to random org object
pub fn decode_randomorg_data(data: String) -> Result<RandomOrgData, ContractError> {
    // using serde_json_wasm, a serde-json alternative for CosmWasm smart contracts
    let random_org_data: RandomOrgData = serde_json_wasm::from_str(&data)
        .map_err(|_| ContractError::CustomError{val: String::from("Invalid random org data format!")})?;
    return Ok(random_org_data);
}

// convert time with format "D:M:Y s:m:hZ" to Timestamp
pub fn convert_datetime_string(data: String) -> Result<Timestamp, ContractError> {
    let date_time = data.parse::<DateTime<Local>>()
        .map_err(|_| ContractError::CustomError{val: String::from("Invalid date string format!")})?;
    return Ok(Timestamp::from_nanos(date_time.timestamp_nanos() as u64));
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn hash_256_success() {
        let data = [0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20];
        assert_eq!(sha256_hash(&data),
        vec![117, 174, 233, 220, 201, 251, 231, 221, 201, 57, 79, 91, 197, 211, 141, 159, 90, 211, 97, 240, 82, 15, 124, 234, 181, 150, 22, 227, 143, 89, 80, 181]);
    }

    #[test]
    fn hash_512_success() {
        let data = [0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20];
        assert_eq!(sha512_hash(&data),
        vec![115, 197, 213, 139, 5, 225, 230, 252, 228, 41, 159, 141, 146, 148, 104, 20, 22, 188, 55, 133, 245, 30, 64, 45, 206, 220, 14, 48, 192, 103, 29, 212, 131,
        33, 160, 36, 140, 204, 19, 56, 154, 1, 43, 82, 81, 63, 27, 91, 191, 130, 14, 145, 235, 79, 97, 105, 40, 24, 52, 133, 180, 241, 235, 34]);
    }

    #[test]
    fn make_commit_id_success() {
        let address = "aabbccddee".to_string();
        let nonce: u64 = 0;
        
        assert_eq!(make_commit_id(address, nonce),"3a904b5371a39495ed468856437d3ffc598edf9b36d1a4dcf710f9840bb8135b".to_string());
    }

    #[test]
    fn generate_hex_randomness_sucess() {
        let randomness: [u8;32] = [0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31];
        let job_id: String = "job test".to_string();
        let num: u32 = 1;

        assert_eq!(generate_hex_randomness(randomness,job_id,num),
        vec!["30a5509e065c583cdc30dabb0de84c8b5c726094ccd8cbf66b0f984167d0bddf".to_string()]);
    }

    #[test]
    fn generate_int_randomness_success() {
        let randomness: [u8;32] = [0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31];
        let job_id: String = "job test".to_string();
        let min: i32 = -10;
        let max: i32 = 10;
        let num: u32 = 32;

        assert_eq!(generate_int_randomness(randomness,job_id,min,max,num),
            vec![-1, 2, 5, -10, 7, 8, -10, 4, 2, -10, 4, -3, 1, -2, 8, 1, -10, -8, -7, 8, -3, -7, 3, -2, 2, 9, 2, -5, 9, 4, -6, -9]);
    }

    #[test]
    fn decode_randomorg_data_success() {
        let message: String = String::from(r#"{"method":"generateSignedIntegers","hashedApiKey":"uSE6BGQ+JMXW38yyAf+/Q+YVZif1ix0RBgq4T2pry5PQhtnNLPWHJYBHdeS+uLkl7YPT/CqMPPJRci1jnd7zJw==","n":32,"min":0,"max":255,"replacement":true,"base":10,"pregeneratedRandomization":null,"data":[108,225,160,35,143,134,3,38,110,245,237,117,0,21,131,185,248,16,8,196,36,56,148,106,32,114,53,114,37,127,216,255],"license":{"type":"developer","text":"Random values licensed strictly for development and testing only","infoUrl":null},"licenseData":null,"userData":null,"ticketData":null,"completionTime":"2023-01-09 02:01:26Z","serialNumber":2}"#);
        let data = RandomOrgData{
            method: "generateSignedIntegers".to_string(),
            hashedApiKey: "uSE6BGQ+JMXW38yyAf+/Q+YVZif1ix0RBgq4T2pry5PQhtnNLPWHJYBHdeS+uLkl7YPT/CqMPPJRci1jnd7zJw==".to_string(),
            n: 32,
            min: 0,
            max: 255,
            replacement: true,
            base: 10,
            pregeneratedRandomization: None,
            data: [108,225,160,35,143,134,3,38,110,245,237,117,0,21,131,185,248,16,8,196,36,56,148,106,32,114,53,114,37,127,216,255],
            license: License{
                r#type: "developer".to_string(),
                text: "Random values licensed strictly for development and testing only".to_string(),
                infoUrl: None
            },
            licenseData: None,
            userData: None,
            ticketData: None,
            completionTime: "2023-01-09 02:01:26Z".to_string(),
            serialNumber: 2
        };

        assert_eq!(data,decode_randomorg_data(message).unwrap());
    }

    #[test]
    fn decode_randomorg_data_fail_with_invalid_format() {
        let message: String = String::from(r#"{"lethod":"generateSignedIntegers","hashedApiKey":"uSE6BGQ+JMXW38yyAf+/Q+YVZif1ix0RBgq4T2pry5PQhtnNLPWHJYBHdeS+uLkl7YPT/CqMPPJRci1jnd7zJw==","n":32,"min":0,"max":255,"replacement":true,"base":10,"pregeneratedRandomization":null,"data":[108,225,160,35,143,134,3,38,110,245,237,117,0,21,131,185,248,16,8,196,36,56,148,106,32,114,53,114,37,127,216,255],"license":{"type":"developer","text":"Random values licensed strictly for development and testing only","infoUrl":null},"licenseData":null,"userData":null,"ticketData":null,"completionTime":"2023-01-09 02:01:26Z","serialNumber":2}"#);
        let data = decode_randomorg_data(message).unwrap_err();
        match data {
            ContractError::CustomError{val: v} => {assert_eq!(v, String::from("Invalid random org data format!"))},
            _ => panic!(),
        }
    }

    #[test]
    fn convert_datetime_success() {
        let time: String = String::from(r#"2023-01-09 02:01:26Z"#);
        assert_eq!(convert_datetime_string(time).unwrap().seconds(),1673229686);
    }

    #[test]
    fn convert_datetime_fail_with_invalid_format() {
        let time: String = String::from(r#"2023/01/09 02:01:26Z"#);
        let date = convert_datetime_string(time).unwrap_err();
        match date {
            ContractError::CustomError{val: v} => {assert_eq!(v, String::from("Invalid date string format!"))},
            _ => panic!(),
        }
    }
}