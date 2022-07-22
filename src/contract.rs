#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Storage, Order, Addr};
use cw2::set_contract_version;
use cw_storage_plus::{Bound};

use crate::error::ContractError;
use crate::msg::{ ExecuteMsg, InstantiateMsg, QueryMsg, GetThreadByIdResponse, ThreadsResponse};
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
        QueryMsg::GetThreadsByCategory {category, offset, limit} => to_binary(&query_threads_by_category(deps, category, offset, limit)?),
        QueryMsg::GetThreadsByAuthor { author, offset, limit } =>  to_binary(&query_threads_by_author(deps, author, offset, limit)?),
    }
}

fn query_thread_by_id(deps: Deps, id: u64) -> StdResult<GetThreadByIdResponse> {
    let thread = threads().load(deps.storage, &id.to_be_bytes().to_vec())?;
    Ok(GetThreadByIdResponse { id:thread.id, title: thread.title, content: thread.content, author: thread.author, category: thread.category})
}

// Limits for pagination
const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;

fn query_threads_by_category(deps: Deps, category: String, offset: Option<u64>, limit: Option<u32>) -> StdResult<ThreadsResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = offset.map(|offset| Bound::exclusive(offset.to_be_bytes().to_vec()));
   
    let list: StdResult<Vec<_>>  = threads()
    .idx.category
    .prefix(category)
    .range(deps.storage, start, None, Order::Ascending)
    .take(limit)
    .map(|item| item.map(|(_, t)| t))
    .collect();

    let result = ThreadsResponse {
        entries: list?,
    };
    Ok(result)    
}

fn query_threads_by_author(deps: Deps, author: Addr, offset: Option<u64>, limit: Option<u32>) -> StdResult<ThreadsResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = offset.map(|offset| Bound::exclusive(offset.to_be_bytes().to_vec()));

    let list: StdResult<Vec<_>>  = threads()
    .idx.author
    .prefix(author)
    .range(deps.storage, start, None, Order::Ascending)
    .take(limit)
    .map(|item| item.map(|(_, t)| t))
    .collect();

    let result = ThreadsResponse {
        entries: list?,
    };
    Ok(result)    
}

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

    #[test]
    fn query_threads_by_category() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
        let msg = InstantiateMsg { };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let info = mock_info("creator", &coins(2, "token"));
        let title = String::from("First Thread");
        let content = String::from("First Message");
        let category = String::from("General");
        // Create Two Threads
        let msg = ExecuteMsg::CreateThread { title: title.clone(), content: content.clone(), category: category.clone()};
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg.clone());
        let _res = execute(deps.as_mut(), mock_env(), info, msg);
        
        // Query Threads With Pagination using Category Index
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetThreadsByCategory {category: String::from("General"), offset: Some(0_u64), limit: Some(10_u32)}).unwrap();
        let value: ThreadsResponse = from_binary(&res).unwrap();

        // Verify Thread Vector
        assert_eq!(1, value.entries[0].id);
        assert_eq!(title, value.entries[0].title);
        assert_eq!(content, value.entries[0].content);
        assert_eq!(category, value.entries[0].category);
        assert_eq!(2, value.entries.len());

    }
    #[test]
    fn query_threads_by_author() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
        let msg = InstantiateMsg { };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let info1 = mock_info("creator1", &coins(2, "token"));
        let info2 = mock_info("creator2", &coins(2, "token"));
        let title = String::from("First Thread");
        let content = String::from("First Message");
        let category = String::from("General");
        // Create Two Threads
        let msg = ExecuteMsg::CreateThread { title: title.clone(), content: content.clone(), category: category.clone()};
        let _res = execute(deps.as_mut(), mock_env(), info1.clone(), msg.clone());
        let _res = execute(deps.as_mut(), mock_env(), info2.clone(), msg);
        
        // Query Threads With Pagination using Author Index
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetThreadsByAuthor {author: info1.sender.clone(), offset: Some(0_u64), limit: Some(10_u32)}).unwrap();
        let value: ThreadsResponse = from_binary(&res).unwrap();

        // Verify Thread Vector For Author1 Address
        assert_eq!(1, value.entries[0].id);
        assert_eq!(info1.sender, value.entries[0].author);
        assert_eq!(1, value.entries.len());

    }
}