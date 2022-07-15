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
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Reply {
    pub msg: String,
    pub author: Addr,
}

pub const THREAD: Item<Thread> = Item::new("thread");
pub const REPLY_COUNTER: Item<u64> = Item::new("reply_counter");
pub const REPLIES: Map<u64, Reply> = Map::new("replies");