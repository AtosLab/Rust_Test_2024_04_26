// Address1 - has CW20-tokens-X
// Address2 - receiver
// When the user sends the CW20-token-Y from Address3 to Address2, SC sends the same amount of CW20-tokens-X from Address1 to Address3
// And at the same time, if the user sends the CW20-tokens-Z to Address2 - SC will not send nothing
// make and share the full code

use cosmwasm_std::{
    to_binary, Api, Binary, CosmosMsg, Env, Extern, HandleResponse, HumanAddr, Querier,
    StdError, StdResult, Storage, Uint128, WasmMsg, QueryRequest
};
use cw20::{BalanceResponse, Cw20ReceiveMsg};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct InitMsg {
    pub address1: HumanAddr,
    pub address2: HumanAddr,
    pub token_x_contract: HumanAddr,
    pub token_y_contract: HumanAddr,
    pub token_z_contract: HumanAddr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct HandleMsg {
    Receive(Cw20ReceiveMsg),
}

pub fn init(
    deps: &mut Extern
    ,
    env: Env,
    info: InitMsg,
) -> StdResult<()> {
    let state = State {
        address1: info.address1.clone(),
        address2: info.address2.clone(),
        token_x_contract: info.token_x_contract,
        token_y_contract: info.token_y_contract,
        token_z_contract: info.token_z_contract,
    };
    STATE.save(deps.storage, &state)?;
    Ok(())
}

pub fn handle(
    deps: Extern,
    env: Env,
    msg: HandleMsg,
) -> StdResult
{
    match msg {
        HandleMsg::Receive(msg) => receive_cw20(deps, env, msg),
    }
}

fn receive_cw20(
    deps: Extern,
    env: Env,
    msg: Cw20ReceiveMsg,
) -> StdResult
<()> {
    let state = STATE.load(deps.storage)?;
    let sender_address = deps.api.human_address(&env.message.sender)?;

    // Ensure the receiver is Address2
    if sender_address != state.address2 {
        return Err(StdError::generic_err("Invalid receiver address"));
    }

    match msg.sender.as_str() {
        // If the sender is Address3 and the token is CW20-token-Y
        address3_str if address3_str == state.address3.as_str() && msg.amount.denom == "CW20-token-Y" => {
            // Send the same amount of CW20-tokens-X from Address1 to Address3
            let transfer_msg = Cw20HandleMsg::Transfer {
                recipient: state.address3.clone(),
                amount: msg.amount,
            };
            let msg = WasmMsg::Execute {
                contract_addr: state.token_x_contract.clone(),
                msg: to_binary(&transfer_msg)?,
                funds: vec![],
            };
            Ok(HandleResponse {
                messages: vec![CosmosMsg::Wasm(msg)],
                log: vec![],
                data: None,
            })
        }
        // If the sender is Address3 but the token is not CW20-token-Y
        address3_str if address3_str == state.address3.as_str() => {
            // Do nothing if tokens other than CW20-token-Y are received from Address3
            Ok(HandleResponse::default())
        }
        // If the sender is not Address3
        _ => Err(StdError::generic_err("Invalid sender address")),
    }
}

fn query_token_balance(
    deps: &Extern,
    contract_addr: HumanAddr,
    address: HumanAddr,
) -> StdResult
{
    let query_msg = QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: contract_addr.into(),
        msg: to_binary(&BalanceQuery { address: address.into() })?,
    });

    let query_result = deps.querier.query(&query_msg)?;

    let balance_res: BalanceResponse = from_binary(&query_result)?;
    Ok(balance_res.balance)
}

// Define your State struct and storage key
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct State {
    pub address1: HumanAddr,
    pub address2: HumanAddr,
    pub address3: HumanAddr,
    pub token_x_contract: HumanAddr,
    pub token_y_contract: HumanAddr,
    pub token_z_contract: HumanAddr,
}

const STATE: Item<State> = Item::new("state");

// Implement your custom error types
pub enum CustomError {
    InvalidReceiverAddress,
    InvalidSenderAddress,
}

impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CustomError::InvalidReceiverAddress => write!(f, "Invalid receiver address"),
            CustomError::InvalidSenderAddress => write!(f, "Invalid sender address"),
        }
    }
}

impl StdError for CustomError {}

impl From
(CustomError) -> StdError {
    StdError::GenericErr {
        msg: "CustomError",
    }
}

// Define your custom messages
pub enum CustomMsg {
    InvalidReceiverAddress,
    InvalidSenderAddress,
}

// Implement your handle function
pub fn handle(
    deps: Extern,
    env: Env,
    msg: HandleMsg,
) -> StdResult {
    match msg {
        HandleMsg::Receive(msg) => receive_cw20(deps, env, msg),
    }
}
