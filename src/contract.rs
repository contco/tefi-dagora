#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Storage};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ ExecuteMsg, InstantiateMsg, QueryMsg, GetThreadByIdResponse};
use crate::state::{ ADMIN, REPLY_COUNTER, THREAD_COUNTER, Thread, threads, next_thread_counter, Reply, REPLIES};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:tefi_dagora";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    ADMIN.save(deps.storage, &info.sender.clone())?;
    REPLY_COUNTER.save(deps.storage, &0)?;
    THREAD_COUNTER.save(deps.storage, &0)?;
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("admin", info.sender)
    )
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreateThread {title, content, category} => create_thread(deps, info, title, content, category),
        ExecuteMsg::UpdateThreadContent { id, content } => update_thread_content(deps, info, id, content),
    }
}

pub fn create_thread(deps: DepsMut, info: MessageInfo, title: String, content: String, category: String) ->Result<Response, ContractError> {
let thread_id = next_thread_counter(deps.storage)?; 
    let thread = Thread {
        id: thread_id,
        title,
        content: String::from(&content),
        category,
        author: info.sender.clone(),
    };
   threads().save(deps.storage, &thread_id.to_be_bytes().to_vec(), &thread)?;
    Ok(
        Response::new()
        .add_attribute("method", "create_thread")
        .add_attribute("author", info.sender)
        .add_attribute("message", content)
    )
    
}

pub fn update_thread_content(deps: DepsMut, info: MessageInfo, id: u64, content: String) -> Result<Response, ContractError> {
    threads().update(deps.storage, &id.to_be_bytes(), |old| match old {
        Some(Thread { id, title, author, category, ..}) => {
            if info.sender != author {
                return Err(ContractError::Unauthorized { });
            }
           let updated_thread = Thread {
            id,
            title,
            content: content.clone(),
            author,
            category
           };
           Ok(updated_thread)
        } ,
        None => Err(ContractError::ThreadNotExists {}),
    })?;
    Ok(
        Response::new()
        .add_attribute("method", "update_thread_content")
        .add_attribute("author", info.sender)
        .add_attribute("content", content),
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
    REPLIES.save(deps.storage, &id.to_be_bytes(), &reply)?;
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
        QueryMsg::GetThreadById { id } => to_binary(&query_thread_by_id(deps, id)?),
       // QueryMsg::GetReply { key } => to_binary(&query_reply_by_key(deps, key)?),
        //QueryMsg::GetReplies { offset, limit } => to_binary(&query_replies(deps, offset, limit)?),
    }
}

fn query_thread_by_id(deps: Deps, id: u64) -> StdResult<GetThreadByIdResponse> {
    let thread = threads().load(deps.storage, &id.to_be_bytes().to_vec())?;
    Ok(GetThreadByIdResponse { id:thread.id, title: thread.title, content: thread.content, author: thread.author, category: thread.category})
}

/*fn query_replies(deps: Deps, offset: Option<u64>, limit: Option<u64>) -> StdResult<u64> {
    let mut min_key: u64;
    let mut max_key: u64;

    match offset {
        Some(a) => min_key = a,
        None => min_key = 0,
    }

    match limit {
        Some(a) => max_key = a + min_key,
        None => max_key = 10 + min_key,
    }

    let replies: StdResult<Vec<_>> = REPLIES
        .range(
            deps.storage,
            Option::Some(Bound::Exclusive(&min_key.to_le_bytes())),
            Bound::Inclusive(&max_key.to_be_bytes()),
            Order::Descending,
        )
        .collect()?;
    Ok(4)
}*/

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies_with_balance, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};
    #[test]
    fn create_thread() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
        let msg = InstantiateMsg { };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let info = mock_info("anyone", &coins(2, "token"));
        let title = String::from("First Thread");
        let content = String::from("First Message");
        let category = String::from("General");
        let msg = ExecuteMsg::CreateThread { title: title.clone(), content: content.clone(), category: category.clone()};
        let _res = execute(deps.as_mut(), mock_env(), info, msg);

         // We should query thread response using id
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetThreadById {id: 1}).unwrap();
        let value: GetThreadByIdResponse = from_binary(&res).unwrap();
        assert_eq!(1, value.id);
        assert_eq!(title, value.title);
        assert_eq!(content, value.content);
        assert_eq!(category, value.category);

    }
    #[test]
    fn update_thread_content() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
        let msg = InstantiateMsg { };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let info = mock_info("creator", &coins(2, "token"));
        let title = String::from("First Thread");
        let content = String::from("First Message");
        let category = String::from("General");
        let msg = ExecuteMsg::CreateThread { title: title.clone(), content: content.clone(), category: category.clone()};
        let _res = execute(deps.as_mut(), mock_env(), info, msg);


        let updated_content = String::from("Updated Content!");

        // Should return error if not executed by thread author
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::UpdateThreadContent { id: 1, content: updated_content.clone()};
        let res = execute(deps.as_mut(), mock_env(), info, msg);
        
        match res {
            Err(ContractError::Unauthorized {}) => {}
            _ => panic!("Must return unauthorized error"),
        }

        // Should update content for author
        let info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg::UpdateThreadContent { id: 1, content: updated_content.clone()};
        let _res = execute(deps.as_mut(), mock_env(), info, msg);

         // Verify content is updated
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetThreadById {id: 1}).unwrap();
        let value: GetThreadByIdResponse = from_binary(&res).unwrap();
        assert_eq!(updated_content, value.content);

    }
}
/* 
    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let msg = InstantiateMsg {};
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

    #[test]
    fn update_message() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
        let msg = InstantiateMsg { title: String::from("Hello World")  ,msg: String::from("Hello New Message") };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Should return error for unauthorized users
        let unauth_info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::UpdateMessage { msg: String::from("Message is Updated!") };
        let res = execute(deps.as_mut(), mock_env(), unauth_info, msg);
        match res {
            Err(ContractError::Unauthorized {}) => {}
            _ => panic!("Must return unauthorized error"),
        }

        //Update only for authorized user
        let info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg::UpdateMessage {msg: String::from("Message is Updated!")};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // new message value should be updated
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetMessage {}).unwrap();
        let value: QueryMsgResponse = from_binary(&res).unwrap();
        assert_eq!("Message is Updated!", value.msg);
    }
    #[test]
    fn add_reply() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
        let msg = InstantiateMsg { title: String::from("Hello World")  ,msg: String::from("Hello New Message") };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // anyone can send a reply
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::AddReply {msg: String::from("This thread is awesome!")};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // reply count should increase by 1
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetMessage {}).unwrap();
        let value: QueryMsgResponse = from_binary(&res).unwrap();
        assert_eq!(1, value.total_replies);

        // reply should be fetched via query_reply_by_key
        let reply_res = query(deps.as_ref(), mock_env(), QueryMsg::GetReply {key: 1}).unwrap();
        let reply_value: QueryReplyResponse = from_binary(&reply_res).unwrap();
        assert_eq!("This thread is awesome!", reply_value.msg);
    }*/

