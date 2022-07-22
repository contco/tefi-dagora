use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::Addr;
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    CreateThread {title: String, content: String, category: String},
    UpdateMessage { msg: String},
    AddReply { msg: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetThreadById {id: u64},
    GetReply {key: u64},
  //  GetReplies { offset: Option<u64>, limit: Option<u64>}
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GetThreadByIdResponse {
    pub id: u64,
    pub title: String,
    pub content: String,
    pub category: String,
    pub author: Addr
}
// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct QueryMsgResponse {
    pub msg: String,
    pub title: String,
    pub author: Addr,
    pub total_replies: u64,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct QueryReplyResponse {
    pub msg: String,
    pub author: Addr,
}
