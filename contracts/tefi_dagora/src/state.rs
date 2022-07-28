use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, StdResult, Storage, Uint128};
use cw_storage_plus::{Item, MultiIndex, IndexList, Index, IndexedMap};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
  pub thread_fee: Uint128,
  pub comment_fee: Uint128,
  pub admin_addr: Addr,
}

pub const CONFIG: Item<Config> = Item::new("CONFIG");

// Thread State and Indexed Map
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Thread {
    pub id: u64,
    pub title: String,
    pub content: String,
    pub author: Addr,
    pub category: String,
}

const THREAD_NAMESPACE: &str = "threads";
pub const THREAD_COUNTER: Item<u64> = Item::new("THREAD_COUNTER");

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

  //Comment State and Indexed Map
  #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
  pub struct Comment {
      pub comment_id: u64,
      pub comment: String,
      pub author: Addr,
      pub thread_id: u64,
  }
  
const COMMENT_NAMESPACE: &str = "comments";
pub const COMMENT_COUNTER: Item<u64> = Item::new("comment_counter");

pub fn next_comment_counter(store: &mut dyn Storage) -> StdResult<u64> {
    let id: u64 = COMMENT_COUNTER.may_load(store)?.unwrap_or_default() + 1;
    COMMENT_COUNTER.save(store, &id)?;
    Ok(id)
}

pub struct CommentIndexes<'a> {
    pub thread: MultiIndex<'a, Vec<u8>, Comment, Vec<u8>>,
}

impl<'a> IndexList<Comment> for CommentIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Comment>> + '_> {
      let v: Vec<&dyn Index<Comment>> = vec![&self.thread];
      Box::new(v.into_iter())
    }
}

pub fn comments<'a>() -> IndexedMap<'a, &'a [u8], Comment, CommentIndexes<'a>> {
    let indexes = CommentIndexes {
      thread: MultiIndex::new(
        |d: &Comment| d.thread_id.to_be_bytes().to_vec(),
        COMMENT_NAMESPACE,
        "comment__thread",
      ),
    };
    IndexedMap::new(COMMENT_NAMESPACE, indexes)
  }