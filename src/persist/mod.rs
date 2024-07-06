use std::sync::{Mutex, MutexGuard};

use derive_getters::Getters;
use hcl::Value;
use indexmap::IndexMap;
use lazy_static::lazy_static;
use rusqlite::Result;
use sqlite_persister::SqlitePersister;

use crate::record::CallRecord;

pub mod sqlite_persister;

lazy_static! {
    static ref GLOBAL_PERSISTER_PROVIDER: Mutex<Box<dyn Persister>> = Mutex::new(Box::new(SqlitePersister::init()));
}

pub struct PersisterProvider {}

impl PersisterProvider {

    pub fn get_lock_to_persister() -> Option<MutexGuard<'static, Box<dyn Persister>>> {

        let lock = GLOBAL_PERSISTER_PROVIDER.lock();

        if lock.is_err() {
            log::trace!("error unwrapping PersisterProvider");
            None 
        } else {
            Some(lock.unwrap())
        }
    }
}

pub trait Persister: Send + Sync {
    fn persist_call_record(&mut self, call_record: &CallRecord) -> Result<bool>;
    fn insert_secret_into_env(&mut self, key: &str, value: &str, env: &str, is_overwrite: bool);
    fn get_all_secrets(&mut self);
    fn get_secrets_for_env(
        &mut self,
        env: &str,
        is_cli: bool,
    ) -> Result<IndexMap<String, Value>, anyhow::Error>;
    fn delete_secret_in_env(&mut self, key: &str, env: &str);
}

#[derive(Debug, Getters)]
pub struct Secret {
    key: String,
    value: String,
    _env: String,
}
