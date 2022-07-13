use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr};
use cw_storage_plus::{Item, Map};
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Thread {
    pub title: String,
    pub msg: String,
    pub author: Addr
}

pub struct ReplyPost {
    pub msg: String,
    pub author: Addr,
}

pub const THREAD: Item<Thread> = Item::new("thread");
pub const REPLIES: Map<&str, ReplyPost> = Map::new("replies");