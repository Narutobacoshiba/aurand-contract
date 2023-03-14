#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, Addr, Api, Timestamp, SubMsg, coins, Uint128,
    MessageInfo, ReplyOn, Response, StdResult, WasmMsg, ensure_eq, Order, BankMsg, Reply
};
use cw2::set_contract_version;

use nois::{NoisCallback, ProxyExecuteMsg};

use crate::error::ContractError;
use crate::msg::{
    ExecuteMsg, InstantiateMsg, QueryMsg, CallbackExecuteMsg,
    PendingCommitmentsQuery, CommitmentsQuery, BotInfoQuery,
    NumberOfCommitmentQuery, ConfigsQuery
};
use crate::state::{
    CONFIGS, Configs, NOIS_CONFIGS, NoisConfigs, TIME_CONFIGS, TimeConfigs,
    COMMITMENTS, PENDING_COMMITMENTS, Commitment, DataRequest, get_commitments, get_commitment,
    BOTS, Bot,
    OWNER, NONCES,
};
use crate::rsa_verify::{verify_message};
use crate::utils::{
    generate_hex_randomness, generate_int_randomness,
    make_commit_id, 
    decode_randomorg_data,
    convert_datetime_string
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:aurand";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const NOIS_CALLBACK_REPLY_ID: u64 = 1;
const COMMITMENT_CALLBACK_REPLY_ID: u64 = 2;

const HEX_DATA_TYPE: &str = "hex";
const INT_DATA_TYPE: &str = "int";

const MIN_NUM: u32 = 1;
const MAX_NUM: u32 = 256;

/// Handling contract instantiation
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let nois_proxy_addr = deps
        .api
        .addr_validate(&_msg.nois_proxy)
        .map_err(|_| ContractError::InvalidProxyAddress{})?;

    CONFIGS.save(deps.storage, &Configs{
        bounty_denom: _msg.bounty_denom.clone(),
        fee: _msg.fee,
        callback_limit_gas: _msg.callback_limit_gas,
        max_callback: _msg.max_callback,
    })?;

    TIME_CONFIGS.save(deps.storage, &TimeConfigs { 
        time_expired: _msg.time_expired, 
        time_per_block: _msg.time_per_block, 
    })?;

    NOIS_CONFIGS.save(deps.storage, &NoisConfigs { 
        nois_proxy: nois_proxy_addr.clone(), 
        nois_fee: _msg.nois_fee 
    })?;

    OWNER.save(deps.storage, &info.sender)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("bounty_denom", _msg.bounty_denom)
        .add_attribute("fee", _msg.fee)
        .add_attribute("callback_limit_gas", _msg.callback_limit_gas.to_string())
        .add_attribute("time_expired", _msg.time_expired.to_string())
        .add_attribute("time_per_block", _msg.time_per_block.to_string())
        .add_attribute("nois_proxy", nois_proxy_addr.to_string())
        .add_attribute("nois_fee", _msg.nois_fee)
        .add_attribute("owner", info.sender))
}

/// Handling contract execution
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::SetConfigs{
            bounty_denom,
            fee,
            callback_limit_gas,
            max_callback,
        } => execute_set_configs(_deps,_info,bounty_denom,fee,callback_limit_gas,max_callback),

        ExecuteMsg::SetTimeConfigs{
            time_expired,
            time_per_block,
        } => execute_set_time_configs(_deps, _info, time_expired, time_per_block),

        ExecuteMsg::SetNoisConfigs{
            nois_proxy,
            nois_fee,
        } => {
            let api = _deps.api;
            execute_set_nois_configs(
                _deps, 
                _info, 
                optional_addr_validate(api,nois_proxy)?, 
                nois_fee
            )
        },

        ExecuteMsg::RegisterBot{
            hashed_api_key,
            moniker
        } => execute_register_bot(_deps,_env,_info,hashed_api_key,moniker),

        ExecuteMsg::UpdateBot{
            hashed_api_key,
            moniker
        } => execute_update_bot(_deps,_env,_info,hashed_api_key,moniker),

        ExecuteMsg::RemoveBot{
            address
        } => {
            let api = _deps.api;
            execute_remove_bot(
                _deps,
                _info,
                optional_addr_validate(api,address)?,
            )
        },

        ExecuteMsg::RequestHexRandomness{
            request_id, 
            num
        } => execute_request_hex_randomness(_deps,_env,_info,request_id,num),

        ExecuteMsg::RequestIntRandomness{
            request_id,
            min,
            max,
            num
        } => execute_request_int_randomness(_deps,_env,_info,request_id,min,max,num),
        
        ExecuteMsg::AddRandomness{
            random_value,
            signature
        } => execute_add_randomness(_deps,_info,random_value,signature),

        ExecuteMsg::NoisReceive{
            callback
        } => execute_nois_receive(_deps,_info,callback),
    }
}

fn optional_addr_validate(api: &dyn Api, addr: String) -> Result<Addr, ContractError> {
    let addr = api.addr_validate(&addr).map_err(|_| ContractError::InvalidAddress{})?;
    Ok(addr)
}

fn execute_set_configs(
    _deps: DepsMut, 
    _info: MessageInfo, 
    bounty_denom: String,
    fee: Uint128,
    callback_limit_gas: u64,
    max_callback: u32,
) -> Result<Response, ContractError> {
    let owner = OWNER.load(_deps.storage)?;

    if !owner.eq(&_info.sender) {
        return Err(ContractError::Unauthorized{});
    }

    CONFIGS.save(_deps.storage, &Configs{
        bounty_denom: bounty_denom.clone(),
        fee,
        callback_limit_gas,
        max_callback
    })?;

    Ok(Response::new()
        .add_attribute("action","set_config")
        .add_attribute("bounty_denom", bounty_denom)
        .add_attribute("fee", fee)
        .add_attribute("callback_limit_gas", callback_limit_gas.to_string())
        .add_attribute("max_callback", max_callback.to_string())
        .add_attribute("owner",_info.sender))
}

fn execute_set_time_configs(
    _deps: DepsMut, 
    _info: MessageInfo, 
    time_expired: u64,
    time_per_block: u64,
) -> Result<Response, ContractError> {
    let owner = OWNER.load(_deps.storage)?;

    if !owner.eq(&_info.sender) {
        return Err(ContractError::Unauthorized{});
    }

    TIME_CONFIGS.save(_deps.storage, &TimeConfigs{
        time_expired,
        time_per_block,
    })?;

    Ok(Response::new()
        .add_attribute("action","set_time_config")
        .add_attribute("time_expired", time_expired.to_string())
        .add_attribute("time_per_block", time_per_block.to_string())
        .add_attribute("owner",_info.sender))
}

fn execute_set_nois_configs(
    _deps: DepsMut, 
    _info: MessageInfo, 
    nois_proxy: Addr, 
    nois_fee: Uint128,
) -> Result<Response, ContractError> {
    let owner = OWNER.load(_deps.storage)?;

    if !owner.eq(&_info.sender.clone()) {
        return Err(ContractError::Unauthorized{});
    }

    NOIS_CONFIGS.save(_deps.storage, &NoisConfigs{
        nois_proxy: nois_proxy.clone(),
        nois_fee,
    })?;

    Ok(Response::new()
        .add_attribute("action","set_nois_config")
        .add_attribute("nois_proxy", nois_proxy.to_string())
        .add_attribute("nois_fee", nois_fee)
        .add_attribute("owner",_info.sender))
}

fn execute_register_bot(
    _deps: DepsMut, 
    _env: Env,
    _info: MessageInfo, 
    hashed_api_key: String, 
    moniker: String
) -> Result<Response, ContractError> {
    if BOTS.has(_deps.storage, _info.sender.clone()) {
        return Err(ContractError::AddressAlreadyRegistered{});
    }

    BOTS.save(_deps.storage, _info.sender.clone(), &Bot{
        address: _info.sender.clone(),
        hashed_api_key: hashed_api_key.clone(),
        moniker: moniker.clone(),
        last_update: _env.block.time,
    })?;

    Ok(Response::new().add_attribute("action","register_bot")
                    .add_attribute("hashed_api_key", hashed_api_key)
                    .add_attribute("moniker", moniker)
                    .add_attribute("bot_address", _info.sender))
}

fn execute_update_bot(
    _deps: DepsMut, 
    _env: Env,
    _info: MessageInfo, 
    hashed_api_key: String, 
    moniker: String
) -> Result<Response, ContractError> {              
    if !BOTS.has(_deps.storage, _info.sender.clone()) {
        return Err(ContractError::UnregisteredAddress{});
    }

    let time_configs = TIME_CONFIGS.load(_deps.storage)?;

    let bot_infor: Bot = BOTS.load(_deps.storage, _info.sender.clone())?;

    // time conditions for bot update action, delay time between two actions must longer than commitment's expiration time  
    let bound_time: Timestamp = _env.block.time;
    let bound_time = bound_time.minus_seconds(time_configs.time_per_block)
                            .minus_seconds(time_configs.time_expired);

    if bound_time.lt(&bot_infor.last_update) {
        return Err(ContractError::ToManyAction{});
    }

    BOTS.save(_deps.storage, _info.sender.clone(), &Bot{
        address: _info.sender.clone(),
        hashed_api_key: hashed_api_key.clone(),
        moniker: moniker.clone(),
        last_update: _env.block.time,
    })?;
    
    Ok(Response::new().add_attribute("action","update_bot")
                    .add_attribute("hashed_api_key", hashed_api_key)
                    .add_attribute("moniker", moniker)
                    .add_attribute("bot_address", _info.sender))
}

fn execute_remove_bot(
    _deps: DepsMut, 
    _info: MessageInfo, 
    bot_addr: Addr
) -> Result<Response, ContractError> {
    let owner = OWNER.load(_deps.storage)?;

    if !owner.eq(&_info.sender) {
        return Err(ContractError::Unauthorized{});
    }


    BOTS.remove(_deps.storage, bot_addr.clone());

    Ok(Response::new().add_attribute("action","remove_bot")
                    .add_attribute("bot_addr", bot_addr)
                    .add_attribute("owner",_info.sender))
}

fn execute_request_randomness(
    _deps: DepsMut,
    _env: Env, 
    _info: MessageInfo,
    request_id: String,
    data_request: DataRequest,
) -> Result<Response, ContractError> {

    // number of user required randomness must in range(MIN_NUM, MAX_NUM) 
    if data_request.num < MIN_NUM || data_request.num > MAX_NUM {
        return Err(ContractError::CustomError{val:String::from("number of randomness must be in range ")
                                                + &MIN_NUM.to_string() 
                                                + &"..".to_string() 
                                                + &MAX_NUM.to_string()});
    }

    let configs = CONFIGS.load(_deps.storage)?;
    let time_configs = TIME_CONFIGS.load(_deps.storage)?;
    let nois_configs = NOIS_CONFIGS.load(_deps.storage)?;

    // check denom and get amount
    let denom = configs.bounty_denom;
    let matching_coin = _info.funds.iter().find(|fund| fund.denom.eq(&denom));
    let sent_amount: Uint128 = match matching_coin {
        Some(coin) => coin.amount,
        None => {
            return Err(ContractError::CustomError {
                val: "Expected denom ".to_string() + &denom,
            });
        }
    };

    // total_fee is calculated by combining nois proxy contract fee and aurand contract fee for each request randomness
    let total_fee = configs.fee.checked_add(nois_configs.nois_fee)
        .map_err(|_| ContractError::Uint128Overflow{})?; 

    if sent_amount < total_fee {
        return Err(ContractError::CustomError{val: String::from("Insufficient fee! required ") 
                                                + &total_fee.to_string() + &denom});
    }

    let mut nonce: u64 = 0;

    if NONCES.has(_deps.storage, _info.sender.clone()) {
        nonce = NONCES.load(_deps.storage, _info.sender.clone())?;
    }else {
        NONCES.save(_deps.storage, _info.sender.clone(), &nonce)?;
    }

    // generate commitment id for request 
    let commit_id = make_commit_id(_info.sender.clone().into_string(), nonce);

    // calculate commitment generation time and expiration time
    let block_time = _env.block.time;
    let commit_time =  Timestamp::from_seconds(block_time.seconds())
                            .plus_seconds(time_configs.time_per_block);
    let expired_time = commit_time.clone()
                    .plus_seconds(time_configs.time_expired);

    let commitment: Commitment = Commitment {
        id: commit_id.clone(),
        request_id: request_id.clone(),
        owner: _info.sender.clone(),
        commit_time,
        expired_time,
        data_request
    };

    COMMITMENTS.push_front(_deps.storage, &(commitment.clone()))?;
    PENDING_COMMITMENTS.save(_deps.storage, commit_id.clone(), &commitment)?;

    // nonces[address] which was incremented by the above
    // successful RequestRandomnesss.
    // This provides protection against the user repeating request,
    // which would result in a predictable/duplicate output, if multiple such
    // requests appeared in the same block
    nonce += 1;
    NONCES.save(_deps.storage, _info.sender.clone(), &nonce)?;

    
    // make a request to Nois Proxy
    let sub_msg: SubMsg = SubMsg {
        msg: WasmMsg::Execute {
            contract_addr: nois_configs.nois_proxy.into(),
            msg: to_binary(&ProxyExecuteMsg::GetNextRandomness { 
                            job_id: commit_id.clone() })?,
            funds: coins(nois_configs.nois_fee.into(), denom),
        }
        .into(),
        id: NOIS_CALLBACK_REPLY_ID,
        gas_limit: None,
        reply_on: ReplyOn::Always,
    };


    Ok(Response::new().add_submessage(sub_msg)
            .add_attribute("action", "request_randomness")
            .add_attribute("commitment_id",commit_id)
            .add_attribute("request_id",request_id)
            .add_attribute("user", _info.sender))
}


fn execute_request_hex_randomness(
    _deps: DepsMut, 
    _env: Env, 
    _info: MessageInfo, 
    request_id: String,
    num: u32
) -> Result<Response, ContractError> {
    execute_request_randomness(_deps, _env, _info, request_id, 
        DataRequest {data_type: HEX_DATA_TYPE.to_string(), min: 0, max: 0, num})
}

fn execute_request_int_randomness(
    _deps: DepsMut, 
    _env: Env, 
    _info: MessageInfo,
    request_id: String, 
    min: i32, 
    max: i32, 
    num: u32
) -> Result<Response, ContractError> {
    execute_request_randomness(_deps, _env, _info, request_id,
        DataRequest {data_type: INT_DATA_TYPE.to_string(), min, max, num})
}

//generate submessage for user callback
fn generate_true_randomness_submsg(
    randomness: [u8; 32],
    commitment: Commitment,
    callback_limit_gas: u64,
) -> Option<SubMsg> {

    let data_request = commitment.data_request;

    match data_request.data_type.as_str() {
        HEX_DATA_TYPE => {
            
            // generate list of hex randomness using PRNG algorithm base on randomness as seed and commitment.id as key
            let hex_randomness = generate_hex_randomness(
                randomness, commitment.id, 
                data_request.num
            );

            let sub_msg = SubMsg {
                msg: WasmMsg::Execute {
                    contract_addr: commitment.owner.to_string(),
                    msg: to_binary(&CallbackExecuteMsg::ReceiveHexRandomness{ 
                        request_id: commitment.request_id, 
                        randomness: hex_randomness,
                    }).unwrap(),
                    funds: vec![],
                }
                .into(),
                id: COMMITMENT_CALLBACK_REPLY_ID,
                gas_limit: Some(callback_limit_gas),
                reply_on: ReplyOn::Always,
            };
        
            Some(sub_msg)
        },
        INT_DATA_TYPE => {

            // generate list of hex randomness using PRNG algorithm base on randomness as seed and commitment.id as key
            let int_randomness = generate_int_randomness(
                randomness, commitment.id, 
                data_request.min, 
                data_request.max, 
                data_request.num
            );
            
            let sub_msg = SubMsg {
                msg: WasmMsg::Execute {
                    contract_addr: commitment.owner.to_string(),
                    msg: to_binary(&CallbackExecuteMsg::ReceiveIntRandomness{ 
                        request_id: commitment.request_id, 
                        randomness: int_randomness
                    }).unwrap(),
                    funds: vec![],
                }
                .into(),
                id: COMMITMENT_CALLBACK_REPLY_ID,
                gas_limit: Some(callback_limit_gas),
                reply_on: ReplyOn::Always,
            };
        
            Some(sub_msg)
        },
        _ => None,
    }
}

fn execute_add_randomness(
    _deps: DepsMut, 
    _info: MessageInfo, 
    random_value: String, 
    signature: String
) -> Result<Response, ContractError> {
    if !BOTS.has(_deps.storage, _info.sender.clone()) {
        return Err(ContractError::UnregisteredAddress{});
    }

    // verify random value
    if !verify_message(random_value.clone(), signature.clone())? {
        return Err(ContractError::RSAVerificationFail{});
    }

    let configs = CONFIGS.load(_deps.storage)?;
    let bounty_denom: String = configs.bounty_denom;
    let commit_bounty: Uint128 = configs.fee;

    let bot = BOTS.load(_deps.storage, _info.sender.clone())?;

    // convert string to random value obj
    let org_randomness = decode_randomorg_data(random_value.clone())?;

    // check if bot api key equivalent to api key use for generate random value
    if !org_randomness.hashedApiKey.eq(&bot.hashed_api_key) {
        return Err(ContractError::InvalidApiKey{});
    }

    // convert time with format "D:M:Y s:m:hZ" to Timestamp
    let completion_time: Timestamp = convert_datetime_string(org_randomness.completionTime)?;

    // get commitments that satisfy time conditions 
    //      commit_time <= completion_time <= expired_time
    let commitments = get_commitments(_deps.storage, completion_time, configs.max_callback)?;

    let mut total_bounty = Uint128::from(0u128);
    let mut messages: Vec<SubMsg> = Vec::new();
    
    // generate callback message for each selected commitment
    for commitment in commitments.iter() {
        if let Some(wasm_msg) = generate_true_randomness_submsg(
            org_randomness.data, 
            (*commitment).clone(), 
            configs.callback_limit_gas
        ) {
            total_bounty = total_bounty.checked_add(commit_bounty)
                .map_err(|_| ContractError::Uint128Overflow{})?;
            messages.push(wasm_msg);
        }
    }

    // create message to send bounty to bot for all success commitments
    if !total_bounty.is_zero() {
        messages.push(SubMsg::new(BankMsg::Send {
            to_address: _info.sender.to_string(),
            amount: coins(total_bounty.into(), bounty_denom),
        }));
    }
    
    Ok(Response::new().add_attribute("action","add_randomness")
                .add_attribute("random_value", random_value)
                .add_attribute("signature", signature)
                .add_attribute("bot", _info.sender)
                .add_submessages(messages))
}

fn execute_nois_receive(
    _deps: DepsMut, 
    _info: MessageInfo, 
    callback: NoisCallback
) -> Result<Response, ContractError> {
    let configs = CONFIGS.load(_deps.storage)?;
    let nois_configs = NOIS_CONFIGS.load(_deps.storage)?;

    ensure_eq!(_info.sender.clone(), nois_configs.nois_proxy, ContractError::UnauthorizedReceive{});

    let job_id = callback.job_id;
    let randomness: [u8; 32] = callback
        .randomness
        .to_array()
        .map_err(|_| ContractError::InvalidRandomness{})?;

    // get commitment with job_id
    let commitment = get_commitment(_deps.storage, job_id.clone())?;

    if commitment.is_none() {
        return Ok(Response::new().add_attribute("action","nois_receive")
                                .add_attribute("message","commitment has been made")
                                .add_attribute("nois_proxy_address", _info.sender));
    }   
    
    let mut sub_messages: Vec<SubMsg> = Vec::new(); 
    // generate callback submessage to user contract using receive randomnesss
    let wasm_msg = generate_true_randomness_submsg(
        randomness, 
        commitment.unwrap(), 
        configs.callback_limit_gas
    );
    
    sub_messages.push(wasm_msg.unwrap());

    // send bounty to contract owner 
    if !configs.fee.is_zero() {
        sub_messages.push(SubMsg::new(BankMsg::Send {
            to_address: OWNER.load(_deps.storage)?.to_string(),
            amount: coins(configs.fee.into(), configs.bounty_denom),
        }));
    }

    Ok(Response::new().add_submessages(sub_messages)
                .add_attribute("job_id", job_id)
                .add_attribute("randomness", hex::encode(randomness))
                .add_attribute("action","nois_receive")
                .add_attribute("nois_proxy_address", _info.sender))
}

/// Handling contract query
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetPendingCommitments{limit} => to_binary(&query_get_pending_commitments(_deps, limit)?),
        QueryMsg::GetCommitments{limit} => to_binary(&query_get_commitments(_deps, limit)?),
        QueryMsg::GetNumberOfCommitment{} => to_binary(&query_get_number_of_commitments(_deps)?),
        QueryMsg::GetBotInfo{address} => to_binary(&query_bot_info(_deps,address)?),
        QueryMsg::GetConfigs{} => to_binary(&query_configs(_deps)?),
    }
}

pub fn query_configs(_deps: Deps) -> StdResult<ConfigsQuery> {
    let configs = CONFIGS.load(_deps.storage)?;
    let nois_configs = NOIS_CONFIGS.load(_deps.storage)?;
    let time_configs = TIME_CONFIGS.load(_deps.storage)?;

    Ok(ConfigsQuery{
        nois_proxy: nois_configs.nois_proxy.into(),
        nois_fee: nois_configs.nois_fee,
        bounty_denom: configs.bounty_denom,
        fee: configs.fee,
        callback_limit_gas: configs.callback_limit_gas,
        time_expired: time_configs.time_expired,
        time_per_block: time_configs.time_per_block,
    })
}

pub fn query_bot_info(_deps: Deps, address: String) -> StdResult<Option<BotInfoQuery>> {
    let addr = optional_addr_validate(_deps.api, address).unwrap();
    
    if !BOTS.has(_deps.storage, addr.clone()) {
        return Ok(None);
    }

    let bot = BOTS.load(_deps.storage, addr).unwrap();

    Ok(Some(BotInfoQuery{
        address: bot.address.to_string(),
        hashed_api_key: bot.hashed_api_key,
        moniker: bot.moniker,
        last_update: bot.last_update,
    }))
}

pub fn query_get_pending_commitments(_deps: Deps, limit: u32) -> StdResult<PendingCommitmentsQuery> {
    let vecs: StdResult<Vec<_>> = PENDING_COMMITMENTS
            .range_raw(_deps.storage, None, None, Order::Ascending)
            .take(limit as usize) // we limit number of commitments can take per query to prevent out of gas
            .collect();
    let vecs = vecs.unwrap();

    let mut commitments: Vec<Commitment> = Vec::new();
    for v in vecs.iter() {
        commitments.push(v.1.clone());
    }

    Ok(PendingCommitmentsQuery{commitments})
}

pub fn query_get_commitments(_deps: Deps, limit: u32) -> StdResult<CommitmentsQuery> {
    let mut vecs: Vec<Commitment> = Vec::new();

    // we limit number of commitments can take per query to prevent out of gas
    for i in COMMITMENTS.iter(_deps.storage)?.take(limit as usize) {
        let commitment = i?;
        vecs.push(commitment);
    }

    Ok(CommitmentsQuery{commitments:vecs})
}

pub fn query_get_number_of_commitments(deps: Deps) -> StdResult<NumberOfCommitmentQuery> {
    let count: u32 = COMMITMENTS.len(deps.storage)?;
    Ok(NumberOfCommitmentQuery{num: count})
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, _msg: Reply) -> Result<Response, ContractError> {
    Ok(Response::new())
}
