#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Order, Addr, Uint128, CosmosMsg, BankMsg, Coin};
use cw2::set_contract_version;
use cw_storage_plus::Bound;

use crate::error::ContractError;
use crate::msg::{ ExecuteMsg, InstantiateMsg, QueryMsg, GetThreadByIdResponse, ThreadsResponse, CommentsResponse, MigrateMsg};
use crate::state::{ CONFIG, Config, THREAD_COUNTER, Thread, threads, next_thread_counter, COMMENT_COUNTER, Comment, comments, next_comment_counter };

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:tefi_dagora";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {

    let config: Config = Config {
        thread_fee: msg.thread_fee.unwrap_or_else(|| Uint128::zero()),
        comment_fee: msg.comment_fee.unwrap_or_else(|| Uint128::zero()),
        admin_addr: info.sender.clone(),
    };

    CONFIG.save(deps.storage, &config)?;
    COMMENT_COUNTER.save(deps.storage, &0)?;
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
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreateThread {title, content, category} => create_thread(deps, info, title, content, category),
        ExecuteMsg::UpdateThreadContent { id, content } => update_thread_content(deps, info, id, content),
        ExecuteMsg::UpdateThreadTitle { id, title } => update_thread_title(deps, info, id, title),
        ExecuteMsg::AddComment { thread_id, comment } => add_comment(deps, info, thread_id, comment),
        ExecuteMsg::UpdateComment { comment_id, comment } => update_comment(deps, info, comment_id, comment),
        ExecuteMsg::Send { address, amount } => send(deps, env, info, address, amount),
        ExecuteMsg::UpdateFees {thread_fee, comment_fee} => update_fees(deps, info, thread_fee, comment_fee)
    }
}

pub fn create_thread(deps: DepsMut, info: MessageInfo, title: String, content: String, category: String) ->Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    
    let coin_amount: Uint128 = info
    .funds
    .iter()
    .find(|c| c.denom == "uluna")
    .map(|c| Uint128::from(c.amount))
    .unwrap_or_else(Uint128::zero);

    if config.thread_fee > coin_amount {
        return Err(ContractError::LessFeeAmount {  });
    }

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
        Some(thread) => {
            if info.sender != thread.author {
                return Err(ContractError::Unauthorized { });
            }
           let updated_thread = Thread {
            content: content.clone(),
            ..thread
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

pub fn update_thread_title(deps: DepsMut, info: MessageInfo, id: u64, title: String) -> Result<Response, ContractError> {
    threads().update(deps.storage, &id.to_be_bytes(), |old| match old {
        Some(thread) => {
            if info.sender != thread.author {
                return Err(ContractError::Unauthorized { });
            }
           let updated_thread = Thread {
            title: title.clone(),
            ..thread
           };
           Ok(updated_thread)
        } ,
        None => Err(ContractError::ThreadNotExists {}),
    })?;
    Ok(
        Response::new()
        .add_attribute("method", "update_thread_title")
        .add_attribute("author", info.sender)
        .add_attribute("title", title),
    )
}

pub fn add_comment(deps: DepsMut, info: MessageInfo, thread_id: u64, comment: String) -> Result<Response, ContractError> {  
    let config = CONFIG.load(deps.storage)?;
    
    let coin_amount: Uint128 = info
    .funds
    .iter()
    .find(|c| c.denom == "uluna")
    .map(|c| Uint128::from(c.amount))
    .unwrap_or_else(Uint128::zero);

    if config.comment_fee > coin_amount {
        return Err(ContractError::LessFeeAmount {  });
    }
    let load_thread = threads().load(deps.storage, &thread_id.to_be_bytes().to_vec());
    match load_thread {
        Ok(_thread)=> {
            let comment_id = next_comment_counter(deps.storage)?;
            let new_comment = Comment {
                comment_id,
                comment: comment.clone(),
                thread_id,
                author: info.sender.clone(),
            };
            comments().save(deps.storage, &comment_id.to_be_bytes().to_vec(), &new_comment)?;
            Ok(
                Response::new()
                .add_attribute("method", "add_comment")
                .add_attribute("author", info.sender)
                .add_attribute("comment", comment)
            )
        },
        Err(_e) => Err(ContractError::ThreadNotExists {  }),
    }
}

pub fn update_comment(deps: DepsMut, info: MessageInfo, comment_id: u64, comment: String) -> Result<Response, ContractError> {  
    comments().update(deps.storage, &comment_id.to_be_bytes(), |old| match old {
     None => Err(ContractError::CommentNotExists { }),
     Some(old_comment) => {
        if info.sender != old_comment.author {
           return Err(ContractError::Unauthorized {  });
        }
        let updated_comment = Comment {
            comment: comment.clone(),
            ..old_comment
        };
        Ok(updated_comment)
     }
    })?;
    Ok(
        Response::new()
        .add_attribute("method", "update_comment")
        .add_attribute("author", info.sender)
        .add_attribute("new_comment", comment)
    )
}

pub fn update_fees(deps: DepsMut, info: MessageInfo, thread_fee: Option<Uint128>, comment_fee: Option<Uint128>) -> Result<Response, ContractError> {  

  let config = CONFIG.update(deps.storage, |mut config| -> Result<_, ContractError> {
        if info.sender != config.admin_addr {
            return Err(ContractError::Unauthorized {  });
         }
         config.thread_fee = thread_fee.unwrap_or_else(|| config.thread_fee);
         config.comment_fee = comment_fee.unwrap_or_else(|| config.comment_fee);
         
         Ok(config)
    })?;

    Ok(
        Response::new()
        .add_attribute("method", "update_fees")
        .add_attribute("author", info.sender)
        .add_attribute("thread_fee", config.thread_fee)
        .add_attribute("comment_fee", config.comment_fee),
    )
}


fn send(deps: DepsMut, env: Env, info: MessageInfo, address: Addr, amount: Uint128) -> Result<Response, ContractError> {  
    
    let config = CONFIG.load(deps.storage)?;
    
    if info.sender != config.admin_addr {
        return Err(ContractError::Unauthorized { });
    }
    let balance = deps.querier.query_balance(env.contract.address.clone(), "uluna".to_string())?;
    
    if amount > balance.amount {
        return Err(ContractError::NotEnoughBalance { });
    }

    let msg = CosmosMsg::Bank(BankMsg::Send {
        to_address: address.to_string(),
        amount: vec![
            Coin {
                denom: "uluna".to_string(),
                amount: amount,
            },
        ],
    });

    Ok(Response::new().add_message(msg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetThreadById { id } => to_binary(&query_thread_by_id(deps, id)?),
        QueryMsg::GetThreadsByCategory {category, offset, limit} => to_binary(&query_threads_by_category(deps, category, offset, limit)?),
        QueryMsg::GetThreadsByAuthor { author, offset, limit } =>  to_binary(&query_threads_by_author(deps, author, offset, limit)?),
        QueryMsg::GetCommentById {id} => to_binary(&query_comment_by_id(deps, id)?),
        QueryMsg::GetCommentsByThread { thread_id, offset, limit } => to_binary(&query_comments_by_thread(deps, thread_id, offset, limit)?),
        QueryMsg::GetConfig {  } => to_binary(&query_config(deps)?)
    }
}

fn query_thread_by_id(deps: Deps, id: u64) -> StdResult<Thread> {
    let thread = threads().load(deps.storage, &id.to_be_bytes().to_vec())?;
    Ok(Thread { id:thread.id, title: thread.title, content: thread.content, author: thread.author, category: thread.category})
}

// Limits for pagination
const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;

fn query_threads_by_category(deps: Deps, category: String, offset: Option<u64>, limit: Option<u32>) -> StdResult<ThreadsResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let latest_thread_id: u64 = THREAD_COUNTER.may_load(deps.storage)?.unwrap_or_default();
    let finish = offset.map(|offset| Bound::inclusive((latest_thread_id - offset).to_be_bytes().to_vec()));
   
    let list: StdResult<Vec<_>>  = threads()
    .idx.category
    .prefix(category)
    .range(deps.storage, None, finish, Order::Descending)
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
    let latest_thread_id: u64 = THREAD_COUNTER.may_load(deps.storage)?.unwrap_or_default();
    let finish = offset.map(|offset| Bound::inclusive((latest_thread_id - offset).to_be_bytes().to_vec()));

    let list: StdResult<Vec<_>>  = threads()
    .idx.author
    .prefix(author)
    .range(deps.storage, None, finish, Order::Descending)
    .take(limit)
    .map(|item| item.map(|(_, t)| t))
    .collect();

    let result = ThreadsResponse {
        entries: list?,
    };
    Ok(result)    
}

fn query_comment_by_id(deps: Deps, comment_id: u64) -> StdResult<Comment> {
    let comment = comments().load(deps.storage, &comment_id.to_be_bytes().to_vec())?;
    Ok(comment)
}

fn query_comments_by_thread(deps: Deps, thread_id: u64, offset: Option<u64>, limit: Option<u32>) -> StdResult<CommentsResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let latest_comment_id: u64 = COMMENT_COUNTER.may_load(deps.storage)?.unwrap_or_default();
    let finish = offset.map(|offset| Bound::inclusive((latest_comment_id - offset).to_be_bytes().to_vec()));

    let list: StdResult<Vec<_>>  = comments()
    .idx.thread
    .prefix(thread_id.to_be_bytes().to_vec())
    .range(deps.storage, None, finish, Order::Descending)
    .take(limit)
    .map(|item| item.map(|(_, comment)| comment))
    .collect();
    let result = CommentsResponse {
        entries: list?,
    };
    Ok(result)    
}

fn query_config(deps: Deps) -> StdResult<Config> {
    let config = CONFIG.load(deps.storage)?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies_with_balance, mock_env, mock_info, MockQuerier, MockApi};
    use cosmwasm_std::{coins, from_binary, OwnedDeps, MemoryStorage};

    fn instantiate_contract() -> OwnedDeps<MemoryStorage, MockApi, MockQuerier> {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
        let msg = InstantiateMsg { thread_fee: Option::Some(Uint128::from(10000u128)), comment_fee: Option::Some(Uint128::from(10000u128))};
        let info = mock_info("creator", &coins(1000000, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        return deps;
    }

    fn create_new_thread(deps: DepsMut) {
        let info = mock_info("creator", &coins(10000000, "uluna"));
        let title = String::from("First Thread");
        let content = String::from("First Message");
        let category = String::from("General");
        let msg = ExecuteMsg::CreateThread { title: title.clone(), content: content.clone(), category: category.clone()};
        let _res = execute(deps, mock_env(), info, msg);
    }

    fn create_new_comment(deps: DepsMut, info: MessageInfo) {
        let msg = ExecuteMsg::AddComment { thread_id: 1, comment: String::from("New Comment")};
        let _res = execute(deps, mock_env(), info.clone(), msg);
    }

    #[test]
    fn create_thread() {
        let mut deps = instantiate_contract();
        create_new_thread(deps.as_mut());

        //Expect Fee Error
        let info = mock_info("creator", &coins(0, "uluna"));
        let title = String::from("First Thread");
        let content = String::from("First Message");
        let category = String::from("General");
        let msg = ExecuteMsg::CreateThread { title: title.clone(), content: content.clone(), category: category.clone()};
        let res = execute(deps.as_mut(), mock_env(), info, msg);
        
        match res {
            Err(ContractError::LessFeeAmount {  } ) => {}
            _ => panic!("Must return less fee amount error"),
        }
         // We should query thread response using id
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetThreadById {id: 1}).unwrap();
        let value: GetThreadByIdResponse = from_binary(&res).unwrap();
        assert_eq!(1, value.id);
        assert_eq!(String::from("First Thread"), value.title);
        assert_eq!(String::from("First Message"), value.content);
        assert_eq!(String::from("General"), value.category);

    }
    #[test]
    fn update_thread_content() {
        let mut deps = instantiate_contract();
        
        create_new_thread(deps.as_mut());

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
    fn update_thread_title() {
        let mut deps = instantiate_contract();
        
        create_new_thread(deps.as_mut());

        let updated_title = String::from("Updated Title");

        // Should return error if not executed by thread author
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::UpdateThreadTitle { id: 1, title: updated_title.clone()};
        let res = execute(deps.as_mut(), mock_env(), info, msg);
        
        match res {
            Err(ContractError::Unauthorized {}) => {}
            _ => panic!("Must return unauthorized error"),
        }

        // Should update content for author
        let info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg::UpdateThreadTitle { id: 1, title: updated_title.clone()};
        let _res = execute(deps.as_mut(), mock_env(), info, msg);

         // Verify content is updated
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetThreadById {id: 1}).unwrap();
        let value: GetThreadByIdResponse = from_binary(&res).unwrap();
        assert_eq!(updated_title, value.title);

    }

    #[test]
    fn add_comment() {
        let mut deps = instantiate_contract();
        let info = mock_info("creator", &coins(10000, "uluna"));
        let comment = String::from("New Reply");
        let msg = ExecuteMsg::AddComment { thread_id: 1, comment: comment.clone()};
        // Add Reply Without Creating Thread
        let res = execute(deps.as_mut(), mock_env(), info.clone(), msg.clone());
        match res {
            Err(ContractError::ThreadNotExists{}) => {}
            _ => panic!("Must return thread not exists error"),
        }

        create_new_thread(deps.as_mut());

        // Check Fee Error
        let fee_info = mock_info("creator", &coins(10, "uluna"));
        let res = execute(deps.as_mut(), mock_env(), fee_info.clone(), msg.clone());
        match res {
            Err(ContractError::LessFeeAmount {  } ) => {}
            _ => panic!("Must return less fee amount error"),
        }
        // Add Reply After Creating Thread
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg);

        // Verify Comment is added
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCommentById {id: 1}).unwrap();
        let value: Comment = from_binary(&res).unwrap();
        assert_eq!(1, value.thread_id);

    }

    #[test]
    fn update_comment() {
        let mut deps = instantiate_contract();
        create_new_thread(deps.as_mut());

        let auth_info = mock_info("creator", &coins(10000, "uluna"));
        // Create A Comment
        create_new_comment(deps.as_mut(), auth_info.clone());

        let update_comment_msg = ExecuteMsg::UpdateComment { comment_id: 1, comment: String::from("Updated Comment")};
       
        // Update Without Authorized User
        let un_auth_info = mock_info("anon", &coins(10000, "uluna"));
        let res = execute(deps.as_mut(), mock_env(), un_auth_info.clone(), update_comment_msg.clone());

        match res {
            Err(ContractError::Unauthorized {}) => {}
            _ => panic!("Must return unauthorized error"),
        }

        // Update With Authorized User
        let _res = execute(deps.as_mut(), mock_env(), auth_info.clone(), update_comment_msg);

        // Verify Updated Comment
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCommentById {id: 1}).unwrap();
        let value: Comment = from_binary(&res).unwrap();
        assert_eq!(String::from("Updated Comment"), value.comment);

    }

    #[test]
    fn update_fees() {
        let mut deps = instantiate_contract();

        let auth_info = mock_info("creator", &coins(10000, "uluna"));
        let update_fee_msg = ExecuteMsg::UpdateFees { thread_fee: Option::Some(Uint128::from(2_u128)), comment_fee: Option::Some(Uint128::from(2_u128)) };
       
        // Update Without Authorized User
        let un_auth_info = mock_info("anon", &coins(10000, "uluna"));
        let res = execute(deps.as_mut(), mock_env(), un_auth_info.clone(), update_fee_msg.clone());

        match res {
            Err(ContractError::Unauthorized {}) => {}
            _ => panic!("Must return unauthorized error"),
        }

        // Update With Authorized User
        let _res = execute(deps.as_mut(), mock_env(), auth_info.clone(), update_fee_msg);

        // Verify Updated Fees
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetConfig {}).unwrap();
        let value: Config = from_binary(&res).unwrap();
        assert_eq!(Uint128::from(2_u128), value.thread_fee);
        assert_eq!(Uint128::from(2_u128), value.comment_fee);
    }

    #[test]
    fn query_threads_by_category() {
        let mut deps = instantiate_contract();

        let info = mock_info("creator", &coins(10000, "uluna"));
        let title = String::from("First Thread");
        let content = String::from("First Message");
        let category = String::from("General");
        // Create Two Threads
        let msg = ExecuteMsg::CreateThread { title: title.clone(), content: content.clone(), category: category.clone()};
        let mut iterator = 11;
        while iterator != 0 {
            let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg.clone());
            iterator -= 1;
        }
        
        // Query Threads With Pagination using Category Index
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetThreadsByCategory {category: String::from("General"), offset: Some(2_u64), limit: Some(10_u32)}).unwrap();
        let value: ThreadsResponse = from_binary(&res).unwrap();

        println!("{:?}", value.entries);

        // Verify Thread Vector
        assert_eq!(1, value.entries[8].id);
        assert_eq!(title, value.entries[0].title);
        assert_eq!(content, value.entries[0].content);
        assert_eq!(category, value.entries[0].category);
        assert_eq!(9, value.entries.len());

    }
    #[test]
    fn query_threads_by_author() {
        let mut deps = instantiate_contract();

        let info1 = mock_info("creator1", &coins(10000, "uluna"));
        let info2 = mock_info("creator2", &coins(10000, "uluna"));
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
    #[test]
    fn query_comments_by_thread() {
       let mut deps = instantiate_contract();
       
        create_new_thread(deps.as_mut());

        let info1 = mock_info("creator1", &coins(10000, "uluna"));
        let info2 = mock_info("creator2", &coins(10000, "uluna"));

        // Create Three Comments for Thread ID 1
        create_new_comment(deps.as_mut(), info1.clone());
        create_new_comment(deps.as_mut(), info1.clone());
        create_new_comment(deps.as_mut(), info2.clone());

        // Query Comments With Pagination using Thread Index
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCommentsByThread {thread_id: 1_u64, offset: Some(0_u64), limit: Some(10_u32)}).unwrap();
        let value: CommentsResponse = from_binary(&res).unwrap();

        // Verify Index Vector for Comments
        assert_eq!(3, value.entries[0].comment_id);
        assert_eq!(2, value.entries[1].comment_id);
        assert_eq!(1, value.entries[2].comment_id);
        assert_eq!(info2.sender, value.entries[0].author);
        assert_eq!(info1.sender, value.entries[2].author);
        assert_eq!(3, value.entries.len());

    }
}