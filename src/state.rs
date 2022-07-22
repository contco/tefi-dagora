use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, StdResult, Storage};
use cw_storage_plus::{Item, Map, MultiIndex, IndexList, Index, IndexedMap, index_string};
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Thread {
    pub id: u64,
    pub title: String,
    pub content: String,
    pub author: Addr,
    pub category: String,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Reply {
    pub msg: String,
    pub author: Addr,
}

pub const THREAD: Item<Thread> = Item::new("thread");
pub const REPLY_COUNTER: Item<u64> = Item::new("reply_counter");
pub const REPLIES: Map<&[u8], Reply> = Map::new("replies");
pub const ADMIN: Item<Addr> = Item::new("ADMIN");
pub const THREAD_COUNTER: Item<u64> = Item::new("THREAD_COUNTER");

// Thread Indexed Map
const THREAD_NAMESPACE: &str = "threads";

pub fn next_thread_counter(store: &mut dyn Storage) -> StdResult<u64> {
    let id: u64 = THREAD_COUNTER.may_load(store)?.unwrap_or_default() + 1;
    THREAD_COUNTER.save(store, &id)?;
    Ok(id)
}

pub struct ThreadIndexes<'a> {
    pub author: MultiIndex<'a, Addr, Thread, Vec<u8>>,
    pub category: MultiIndex<'a, String, Thread, Vec<u8>>,
  }
  
  impl<'a> IndexList<Thread> for ThreadIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Thread>> + '_> {
      let v: Vec<&dyn Index<Thread>> = vec![&self.author, &self.category];
      Box::new(v.into_iter())
    }
  }
  
  
  pub fn threads<'a>() -> IndexedMap<'a, &'a [u8], Thread, ThreadIndexes<'a>> {
    let indexes = ThreadIndexes {
      author: MultiIndex::new(
        |d: &Thread| d.author.clone(),
        THREAD_NAMESPACE,
        "threads__author",
      ),
      category: MultiIndex::new(
        |d: &Thread| d.category.clone(),
        THREAD_NAMESPACE,
        "threads__category",
      ),
    };
    IndexedMap::new(THREAD_NAMESPACE, indexes)
  }
