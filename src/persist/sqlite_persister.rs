use std::sync::Mutex;

use colored::Colorize;
use hcl::Value;
use indexmap::IndexMap;
use log::{error, trace};
use log_derive::logfn;
use rusqlite::{Connection, Result};

use crate::record::CallRecord;

use super::{Persister, Secret};

pub struct SqlitePersister {
    conn: Mutex<Connection>,
}

impl SqlitePersister {
    pub fn init() -> SqlitePersister {
        let conn = Connection::open("drop.db");

        match conn {
            Ok(conn) => {
                let drop_record_call = conn.execute(
                    "create table if not exists drop_record (
                         id integer primary key,
                         drop_id text not null,
                         full_url text not null,
                         status_code integer,
                         full_response text,
                         timestamp date default (datetime('now','localtime'))
                        )",
                    (),
                );

                trace!("SqlitePersister create drop_record call res: {drop_record_call:#?}");

                let secret_call = conn.execute(
                    "create table if not exists secrets (
                         id integer primary key,
                         key text not null,
                         value text not null,
                         env text not null,
                         UNIQUE(key, env)
                     )",
                    (),
                );

                trace!("SqlitePersister create secrets call res: {secret_call:#?}");

                SqlitePersister { conn: Mutex::new(conn) }
            }
            Err(err) => {
                error!("SqlitePersister conn err {err:#?}");
                panic!();
            }
        }
    }
}

impl Persister for SqlitePersister {
    fn persist_call_record(&mut self, call_record: &CallRecord) -> Result<bool> {
        trace!("persist_call_record call_record: {call_record:#?}");

        let conn_attempt = self.conn.get_mut();

        if conn_attempt.is_err() {

            log::error!("SqlitePersister.persist_call_record error: {:?}", conn_attempt.unwrap_err());

            // todo- return error to caller
            Ok(false)
        } else {

        let res = conn_attempt.unwrap().execute(
            "INSERT INTO drop_record (drop_id, full_url, status_code, full_response) VALUES (?1, ?2, ?3, ?4)",
            (
                &call_record.drop_id,
                &call_record.full_url,
                &call_record.status_code.unwrap().as_u16(),
                &call_record.full_response,
            ),
        );

        match res {
            Ok(_code) => {
                trace!("persist_call_record result {res:#?}");
            }
            Err(err) => {
                // todo- move handling into caller 
                panic!("persist_call_record result err {err:#?}")
            }
        }

        Ok(true)
    }

    }

    fn insert_secret_into_env(&mut self, key: &str, value: &str, env: &str, is_overwrite: bool) {
        let sql = if is_overwrite {
            "INSERT or replace INTO secrets (key, value, env) VALUES (?1, ?2, ?3)"
        } else {
            "INSERT INTO secrets (key, value, env) VALUES (?1, ?2, ?3)"
        };

        // todo- handle unwrap correctly
        let insert_secret_call = self.conn.get_mut().unwrap().execute(sql, (key, value, env));

        match insert_secret_call {
            Ok(res) => {
                trace!("SqlitePersister create insert_secret_call call res: {res:#?}");
                println!(
                    "secret {} in environment {} set successfully.\n",
                    key.yellow(),
                    env.yellow()
                );
            }
            Err(err) => {
                if err.to_string().contains("UNIQUE constraint") {
                    panic!(
                        "Key {} already exists in env {}. Run `drop secret get {env}` to view.\n",
                        key.yellow(),
                        env.yellow()
                    )
                } else {
                    panic!("Failed to insert secret {key} into environment {value}: {err}");
                }
            }
        }
    }

    fn delete_secret_in_env(&mut self, key: &str, env: &str) {
        let sql = "delete from secrets where key = ?1 and env = ?2";

        // todo- handle unwrap correctly
        let delete_secret_call = self.conn.get_mut().unwrap().execute(sql, (key, env));

        match delete_secret_call {
            Ok(res) => {
                trace!("SqlitePersister create delete_secret_call call res: {res:#?}");
                println!("secrets in environment {} deleted: {res:?}\n", env.yellow());
            }
            Err(err) => {
                trace!("delete_secret_in_env err {err:?}");
                panic!("error deleting secret {key} in environment {env}")
            }
        }
    }

    fn get_all_secrets(&mut self) {

        // todo- handle unwrap correctly
        let select_secrets_for_all_env = self.conn.get_mut().unwrap().prepare("SELECT key, value, env FROM secrets");

        let mut select_secrets_for_env = match select_secrets_for_all_env {
            Ok(res) => res,
            Err(err) => {
                panic!("Select all secrets err: {err}");
            }
        };

        let secrets_for_env_res = select_secrets_for_env.query_map([], |row| {
            let key_res: Result<String> = row.get(0);
            let value_res: Result<String> = row.get(1);
            let env_val: Result<String> = row.get(2);

            let key = key_res.unwrap();
            let value = value_res.unwrap();
            let env = env_val.unwrap();

            Ok(Secret {
                key,
                value,
                _env: env,
            })
        });

        let secrets_for_env = match secrets_for_env_res {
            Ok(res) => res,
            Err(err) => {
                panic!("Failed to retreive all secrets: {err}");
            }
        };

        let as_iter = secrets_for_env.into_iter();

        let collected: Vec<Secret> = as_iter
            .map(|s| {
                if s.is_err() {
                    trace!("get_secrets_for_env invalid result returned: {s:?}");
                }
                s.unwrap()
            })
            .collect();

        if collected.is_empty() {
            println!("No secrets set");
        } else {
            println!("Secrets: {}", "all secrets".yellow());
            for secret in collected {
                println!("{secret:?}");
            }
        }
    }

    #[logfn(
        ok = "TRACE",
        err = "ERROR",
        fmt = "get_secrets_for_env: {:?}",
        log_ts = true
    )]
    fn get_secrets_for_env(&mut self, env: &str, is_cli: bool) -> Result<IndexMap<String, Value>, anyhow::Error> {

        // todo- handle unwrap correctly
        
        let select_secrets_for_env_call = self
            .conn
            .get_mut().unwrap()
            .prepare("SELECT key, value FROM secrets where env = :env");

        let mut select_secrets_for_env = match select_secrets_for_env_call {
            Ok(res) => res,
            Err(err) => {
                panic!("Failed to get secrets for environment {env}: {err}");
            }
        };

        let secrets_for_env_res = select_secrets_for_env.query_map(&[(":env", env)], |row| {
            let key_res: Result<String> = row.get(0);
            let value_res: Result<String> = row.get(1);

            let key = key_res.unwrap();
            let value = value_res.unwrap();

            Ok(Secret {
                key,
                value,
                _env: env.to_string(),
            })
        });

        let secrets_for_env = match secrets_for_env_res {
            Ok(res) => res,
            Err(err) => {
                panic!("Failed to retreive secrets from environment {env}: {err}");
            }
        };

        let as_iter = secrets_for_env.into_iter();

        let collected: Vec<Secret> = as_iter
            .map(|s| {
                if s.is_err() {
                    trace!("get_secrets_for_env invalid result returned: {s:?}");
                }
                s.unwrap()
            })
            .collect();

        if is_cli {
            if collected.is_empty() {
                println!("No secrets for env: {}", env.yellow());
            } else {
                println!("Secrets for env: {}", env.yellow());

                for secret in &collected {
                    println!("{secret:?}",);
                }
            }
        }

        let mut secret_map: IndexMap<String, Value> = IndexMap::new();

        for secret in collected {
            secret_map.insert(
                secret.key().to_string(),
                Value::String(secret.value().to_string()),
            );
        }
    
        Ok(secret_map)
    }
}
