#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    Addr, Api, Order, to_binary, ensure_eq, WasmMsg
};
use cw2::set_contract_version;
use nois::{
    NoisCallback, ProxyExecuteMsg,
    shuffle
};

use crate::error::ContractError;
use crate::msg::{
    ExecuteMsg, InstantiateMsg, QueryMsg,
    PrizesQuery, DistributePrizesQuery, AttendeeQuery
};
use crate::state::{
    Configs, CONFIGS,
    Attendee, ATTENDEE_LIST,
    Status, WHITELIST,
    DistributePrize, DISTRIBUTE_PRIZES,
    PRIZES, Prizes, OWNER, END_ROUND
};
use crate::utils::{
    convert_datetime_string, generate_lucky_number
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:lucky-roll";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");


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
        nois_proxy: nois_proxy_addr,
        time_start: convert_datetime_string(_msg.time_start),
        time_end: convert_datetime_string(_msg.time_end),
    })?;

    OWNER.save(deps.storage, &info.sender.clone())?;

    PRIZES.save(deps.storage, &Prizes{
        shuffle: false,
        prizes: vec![]
    })?;

    DISTRIBUTE_PRIZES.save(deps.storage, &vec![])?;

    END_ROUND.save(deps.storage, &false)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
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
        ExecuteMsg::Reset{
            nois_proxy,
            time_start,
            time_end,
        } => {
            let api = _deps.api;
            execute_reset(
                _deps,
                _info,
                optional_addr_validate(api,nois_proxy)?,
                time_start,
                time_end,
            )
        },

        ExecuteMsg::SetPrizes {
            prizes
        } => execute_set_prizes(_deps, _info, prizes),

        ExecuteMsg::SetWhiteList {
            attendees
        } => execute_set_whitelist(_deps,_info,attendees),

        ExecuteMsg::Roll {} => execute_roll(_deps, _env, _info),

        ExecuteMsg::NoisReceive { callback } => execute_receive(_deps, _env, _info, callback),
    
        ExecuteMsg::LuckyNumber {} => execute_lucky_number(_deps, _env, _info),
    }
}

fn optional_addr_validate(api: &dyn Api, addr: String) -> StdResult<Option<Addr>> {
    let addr = Some(api.addr_validate(&addr)?);
    Ok(addr)
}

fn execute_reset (
    _deps: DepsMut,
    _info: MessageInfo,
    nois_proxy: Option<Addr>,
    time_start: String,
    time_end: String,
) -> Result<Response, ContractError> {
    let owner = OWNER.load(_deps.storage)?;
    
    if !owner.eq(&_info.sender.clone()) {
        return Err(ContractError::Unauthorized{});
    }

    if !nois_proxy.is_some() {
        return Err(ContractError::CustomError{val:"Invalid nois proxy address!".to_string()});
    }

    CONFIGS.save(_deps.storage, &Configs{
        nois_proxy: nois_proxy.unwrap(),
        time_start: convert_datetime_string(time_start),
        time_end: convert_datetime_string(time_end),
    })?;
    
    WHITELIST.clear(_deps.storage);

    ATTENDEE_LIST.clear(_deps.storage);
    
    PRIZES.save(_deps.storage, &Prizes{
        shuffle: false,
        prizes: vec![]
    })?;

    DISTRIBUTE_PRIZES.save(_deps.storage, &vec![])?;

    END_ROUND.save(_deps.storage, &false)?;


    return Ok(Response::new().add_attribute("action","reset")
                            .add_attribute("owner", _info.sender));
}

fn execute_set_prizes(
    _deps: DepsMut,
    _info: MessageInfo,
    prizes: Vec<String>
) -> Result<Response, ContractError> {
    let configs = CONFIGS.load(_deps.storage)?;
    let owner = OWNER.load(_deps.storage)?;
    
    if !owner.eq(&_info.sender.clone()) {
        return Err(ContractError::Unauthorized{});
    }

    if END_ROUND.load(_deps.storage)? {
        return Err(ContractError::RoundEnd{});
    }

    PRIZES.save(_deps.storage, &Prizes {
        shuffle: false,
        prizes: prizes
    })?;

    let msg = WasmMsg::Execute {
        contract_addr: configs.nois_proxy.into(),
        msg: to_binary(&ProxyExecuteMsg::GetNextRandomness { 
                        job_id: "set prizes".to_string()})?,
        funds: _info.funds,
    };

    return Ok(Response::new().add_message(msg)
            .add_attribute("action","set prizes")
            .add_attribute("owner", _info.sender));
}


fn execute_set_whitelist(
    _deps: DepsMut,
    _info: MessageInfo,
    attendees: Vec<String>,
) -> Result<Response, ContractError> {
    let owner = OWNER.load(_deps.storage)?;
    
    if !owner.eq(&_info.sender.clone()) {
        return Err(ContractError::Unauthorized{});
    }

    if END_ROUND.load(_deps.storage)? {
        return Err(ContractError::RoundEnd{});
    }

    WHITELIST.clear(_deps.storage);

    for a in attendees.iter() {
        let attendee =  optional_addr_validate(_deps.api, (*a).clone())?;
        if !attendee.is_some() {
            return Err(ContractError::CustomError{val:"Invalid validator address! ".to_string() + &a});
        }

        WHITELIST.save(_deps.storage, attendee.unwrap(), &Status{
            attended: false
        })?;
    }

    return Ok(Response::new().add_attribute("action","set whitelist"));
}


fn execute_roll(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo
) -> Result<Response, ContractError> {
    let owner = OWNER.load(_deps.storage)?;
    
    if !owner.eq(&_info.sender.clone()) {
        return Err(ContractError::Unauthorized{});
    }

    if END_ROUND.load(_deps.storage)? {
        return Err(ContractError::RoundEnd{});
    }

    let configs = CONFIGS.load(_deps.storage)?;
    let block_time = _env.block.time;

    if block_time.le(&configs.time_end) {
        return Err(ContractError::CustomError{val:"Game not end yet!".to_string()});
    }

    let prizes = PRIZES.load(_deps.storage)?;

    if !prizes.shuffle {
        return Err(ContractError::CustomError{val:"Prizes are not shuffled!".to_string()});
    }
    
    let mut prizes = prizes.prizes;
    let mut distribute_prizes: Vec<DistributePrize> = Vec::new(); 


    let vecs: StdResult<Vec<_>> = ATTENDEE_LIST
            .range_raw(_deps.storage, None, None, Order::Ascending)
            .collect();
    let vecs = vecs.unwrap();

    if vecs.len() > prizes.len() {
        return Err(ContractError::CustomError{val:"insufficient prize!".to_string()});
    }

    for v in vecs.iter() {
        prizes = shuffle(v.1.lucky_number, prizes);
        let prize = prizes.pop();

        distribute_prizes.push(DistributePrize{
            address: v.1.address.clone(),
            prize: prize.unwrap()
        });
    }

    DISTRIBUTE_PRIZES.save(_deps.storage, &distribute_prizes)?;

    END_ROUND.save(_deps.storage, &true)?;

    return Ok(Response::new().add_attribute("action","roll"));
}

fn execute_lucky_number(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
) -> Result<Response, ContractError> {
    if END_ROUND.load(_deps.storage)? {
        return Err(ContractError::RoundEnd{});
    }
    
    if !WHITELIST.has(_deps.storage, _info.sender.clone()) {
        return Err(ContractError::CustomError{val:"Denied action!".to_string()});
    }
    
    if WHITELIST.load(_deps.storage, _info.sender.clone())?.attended {
        return Err(ContractError::CustomError{val:"Only allowed to take the lucky number once!".to_string()});
    }

    let configs = CONFIGS.load(_deps.storage)?;
    let block_time = _env.block.time;

    if block_time.lt(&configs.time_start) {
        return Err(ContractError::CustomError{val:"Game not start yet!".to_string()});
    }

    if block_time.gt(&configs.time_end) {
        return Err(ContractError::CustomError{val:"Game has ended!".to_string()});
    }

    ATTENDEE_LIST.save(_deps.storage, _info.sender.clone(),&Attendee{
        address: _info.sender.clone(),
        lucky_number: [0u8;32]
    })?;

    WHITELIST.save(_deps.storage, _info.sender.clone(), &Status{
        attended: true
    })?;

    // generate callback to nois
    let msg = WasmMsg::Execute {
        contract_addr: configs.nois_proxy.into(),
        msg: to_binary(&ProxyExecuteMsg::GetNextRandomness { 
                        job_id: _info.sender.to_string()})?,
        funds: _info.funds,
    };

    return Ok(Response::new()
        .add_message(msg)
        .add_attribute("action","lucky number"));
}

pub fn execute_receive(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    callback: NoisCallback,
) -> Result<Response, ContractError> {
    //load proxy address from store
    let configs = CONFIGS.load(_deps.storage)?;
    ensure_eq!(_info.sender, configs.nois_proxy, ContractError::UnauthorizedReceive{});

    if END_ROUND.load(_deps.storage)? {
        return Err(ContractError::RoundEnd{});
    }
    
    let randomness: [u8; 32] = callback
        .randomness
        .to_array()
        .map_err(|_| ContractError::InvalidRandomness{})?;

    let job_id = callback.job_id;

    if job_id.eq(&String::from("set prizes")) {
        let mut prizes = PRIZES.load(_deps.storage)?.prizes;

        prizes = shuffle(randomness, prizes);

        PRIZES.save(_deps.storage, &Prizes{
            shuffle: true,
            prizes: prizes,
        })?;

        return Ok(Response::new().add_attribute("action","set prizes"));
    }
    
    let address = optional_addr_validate(_deps.api, job_id)?;

    if !address.is_some() {
        return Ok(Response::new());
    }

    let address = address.unwrap();

    if !ATTENDEE_LIST.has(_deps.storage, address.clone()) {
        return Ok(Response::new());
    }

    ATTENDEE_LIST.save(_deps.storage, address.clone(), &Attendee{
        address: address.clone(),
        lucky_number: generate_lucky_number(address.to_string(), randomness),
    })?;

    return Ok(Response::new().add_attribute("action","get lucky number"));
}



/// Handling contract query
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetPrizes{} => to_binary(&query_get_prizes(_deps)?),
        QueryMsg::GetDistributePrizes{} => to_binary(&query_get_distribute_prizes(_deps)?),
        QueryMsg::GetAttendees{} => to_binary(&query_get_attendees(_deps)?)
    }
}

pub fn query_get_prizes(deps: Deps) -> StdResult<PrizesQuery> {
    let prizes = PRIZES.load(deps.storage)?.prizes;
    return Ok(PrizesQuery{prizes});
}

pub fn query_get_distribute_prizes(deps: Deps) -> StdResult<DistributePrizesQuery> {
    let prizes = DISTRIBUTE_PRIZES.load(deps.storage)?;
    return Ok(DistributePrizesQuery{prizes});
}

pub fn query_get_attendees(deps: Deps) -> StdResult<AttendeeQuery> {
    let vecs: StdResult<Vec<_>> = ATTENDEE_LIST
        .range_raw(deps.storage, None, None, Order::Ascending)
        .collect();
    let vecs = vecs.unwrap();

    let mut attendees: Vec<Attendee> = Vec::new();
    for v in vecs.iter() {
        attendees.push(v.1.clone());
    }

    return Ok(AttendeeQuery{
        number: vecs.len(),
        attendees: attendees}
    );
}