#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Storage};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{QueryMsgResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Thread, THREAD, REPLY_COUNTER, Reply, REPLIES};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:tefi_dagora";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let thread = Thread {
        title: String::from(&msg.title),
        msg: String::from(&msg.msg),
        author: info.sender.clone(),
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    THREAD.save(deps.storage, &thread)?;
    REPLY_COUNTER.save(deps.storage, &0)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("author", info.sender)
        .add_attribute("title",  msg.title)
        .add_attribute("message", msg.msg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateMessage { msg } => update_message(deps, info, msg),
        ExecuteMsg::AddReply { msg } => add_reply(deps, info, msg),
    }
}

pub fn update_message(deps: DepsMut, info: MessageInfo, msg: String) -> Result<Response, ContractError> {
    THREAD.update(deps.storage, |mut state| -> Result<_, ContractError> {
        if info.sender != state.author {
           return Err(ContractError::Unauthorized {});
        }
        state.msg = String::from(&msg);
        Ok(state)
    })?;
    Ok(
        Response::new()
        .add_attribute("method", "update_message")
        .add_attribute("author", info.sender)
        .add_attribute("message",  msg)
    )
}


pub fn next_reply_counter(store: &mut dyn Storage) -> StdResult<u64> {
    let id: u64 = REPLY_COUNTER.may_load(store)?.unwrap_or_default() + 1;
    REPLY_COUNTER.save(store, &id)?;
    Ok(id)
}

pub fn add_reply(deps: DepsMut, info: MessageInfo, msg: String) -> Result<Response, ContractError> {
    let id = next_reply_counter(deps.storage)?;
    let reply = Reply {
        msg: String::from(&msg),
        author: info.sender.clone(),
    };
    REPLIES.save(deps.storage, id, &reply)?;
    Ok(
        Response::new()
        .add_attribute("method", "add_reply")
        .add_attribute("author", info.sender)
        .add_attribute("message", msg)
    )
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetMessage {} => to_binary(&query_message(deps)?),
    }
}

fn query_message(deps: Deps) -> StdResult<QueryMsgResponse> {
    let message_state = THREAD.load(deps.storage)?;
    let total_replies = REPLY_COUNTER.load(deps.storage)?;
    Ok(QueryMsgResponse { title: message_state.title, msg: message_state.msg, author: message_state.author, total_replies})
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies_with_balance, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let msg = InstantiateMsg { title: String::from("Hello World")  ,msg: String::from("Hello New Message") };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetMessage {}).unwrap();
        let value: QueryMsgResponse = from_binary(&res).unwrap();
        assert_eq!("Hello World", value.title);
        assert_eq!("Hello New Message", value.msg);
        assert_eq!("creator", value.author);
        assert_eq!(0, value.total_replies);
    }
/* 
    #[test]
    fn increment() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let msg = InstantiateMsg { count: 17 };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::Increment {};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // should increase counter by 1
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: CountResponse = from_binary(&res).unwrap();
        assert_eq!(18, value.count);
    }

    #[test]
    fn reset() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let msg = InstantiateMsg { count: 17 };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let unauth_info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::Reset { count: 5 };
        let res = execute(deps.as_mut(), mock_env(), unauth_info, msg);
        match res {
            Err(ContractError::Unauthorized {}) => {}
            _ => panic!("Must return unauthorized error"),
        }

        // only the original creator can reset the counter
        let auth_info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg::Reset { count: 5 };
        let _res = execute(deps.as_mut(), mock_env(), auth_info, msg).unwrap();

        // should now be 5
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: CountResponse = from_binary(&res).unwrap();
        assert_eq!(5, value.count);
    }*/
}
