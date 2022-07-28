use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{Addr, Uint128};

use crate::state::{Thread, Comment};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    CreateThread {title: String, content: String, category: String},
    UpdateThreadContent {id: u64, content: String},
    UpdateThreadTitle {id: u64, title: String},
    AddComment {thread_id: u64, comment: String },
    UpdateComment {comment_id: u64, comment: String},
    Send {address: Addr, amount: Uint128}
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetThreadById {id: u64},
    GetThreadsByCategory {category: String, offset: Option<u64>, limit: Option<u32>},
    GetThreadsByAuthor {author: Addr, offset: Option<u64>, limit: Option<u32>},
    GetCommentById {id: u64},
    GetCommentsByThread {thread_id: u64, offset: Option<u64>, limit: Option<u32>}
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GetThreadByIdResponse {
    pub id: u64,
    pub title: String,
    pub content: String,
    pub category: String,
    pub author: Addr
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ThreadsResponse {
    pub entries: Vec<Thread>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CommentsResponse {
    pub entries: Vec<Comment>
}