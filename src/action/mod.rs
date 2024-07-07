use accessor::PostActionAccessor;
use colored::Colorize;
use hcl::{expr::Traversal, Expression};
use log::error;
use std::collections::HashMap;

use crate::{parser::hcl_block::HclBlock, persist::PersisterProvider, record::CallRecord};

pub mod accessor;

pub type AfterActionConfig = HashMap<String, Vec<HashMap<String, ActionValue>>>;

#[derive(Debug, Clone)]
pub enum ActionValue {
    String(String),
    Traversal(Traversal),
    Bool(bool),
}

pub struct PostAction {}

impl PostAction {
    pub fn prepare_action_from_hcl_expression(exp: &Expression) -> HashMap<String, ActionValue> {
        let mut map = HashMap::new();

        let throw = || {
            error!("invalid after action objects");
            std::process::exit(1);
        };

        match exp {
            Expression::Object(obj) => {
                for (key, value) in obj {
                    let value_to_insert = match value {
                        Expression::String(str) => ActionValue::String(str.to_string()),
                        Expression::Traversal(trav) => ActionValue::Traversal(*trav.clone()), // small clone
                        Expression::Bool(b) => ActionValue::Bool(*b),
                        &_ => throw(),
                    };
                    map.insert(key.to_string(), value_to_insert);
                }
            }
            &_ => {
                throw();
            }
        }

        map
    }

    pub fn run_post_action_callbacks(mut call_record: CallRecord) -> CallRecord {
        let after_action_config = call_record.after_action_config.take();

        // default post call action
        PostAction::post_action_persist()(&call_record);
        PostAction::post_action_print_outputs_to_console()(&call_record);

        match after_action_config {
            Some(user_post_action_configs) => {
                for (_, action_configs) in user_post_action_configs {
                    for action_config in action_configs {
                        let action_config_id = action_config.get("type").unwrap();

                        if let ActionValue::String(value) = action_config_id {
                            if value.as_str() == "set_secret" {
                                PostAction::post_action_set_env_vars(&action_config)(&call_record);
                            }
                        }
                    }
                }
            }
            None => {
                // TODO
                log::trace!("None run_post_action_callbacks after_action_config ");
            }
        }

        call_record
    }

    pub fn post_action_persist() -> impl FnMut(&CallRecord) {
        return |call_record| {
            
            let persister_lock = PersisterProvider::get_lock_to_persister();

            if persister_lock.is_none() {
                // warn 
            } else {
                let _ = persister_lock.unwrap().persist_call_record(call_record);
            }
            
        };
    }

    pub fn post_action_print_outputs_to_console() -> impl Fn(&CallRecord) {
        return |call_record| {
            let drop_id = call_record.drop_id();
            let status = call_record.status_code().unwrap();
            println!("\n{drop_id:#?} result: {status:#?}\n");
            let outputs = call_record.output_records();

            if let Some(outputs) = outputs {
                for output in outputs {
                    output.print();
                }
            }
        };
    }

    pub fn post_action_set_env_vars(
        action_config: &HashMap<String, ActionValue>,
    ) -> impl FnMut(&CallRecord) {
        let input = PostActionAccessor::get_trav_or_panic(action_config, "input").to_owned();
        let input = HclBlock::traversal_to_string(&input);
        let key_to_set = PostActionAccessor::get_string_or_panic(action_config, "key").clone();
        let overwrite =
            PostActionAccessor::get_bool_or_panic(action_config, "overwrite").to_owned();
        let env = PostActionAccessor::get_string_or_default(action_config, "env", "base").clone();

        return move |call_record| {

            let persister_lock = PersisterProvider::get_lock_to_persister();

            if persister_lock.is_none() {
                // warn 
            } else {
            
                let persister = &mut persister_lock.unwrap();

                let outputs = call_record.output_records();

                // todo- refactor into Result<>
                let mut matched = false;

                if let Some(outputs) = outputs {
                    for output_record in outputs {
                        let output_key = output_record.key();

                        if *output_key == input {
                            matched = true;
                            let output_value = output_record.value();

                            // todo
                            log::trace!(
                                "input {input} key_to_set {key_to_set} output_value {output_value}"
                            );

                            persister.insert_secret_into_env(
                                &key_to_set,
                                output_value,
                                &env,
                                overwrite,
                            );
                        }
                    }
                }

                if !matched {
                    println!("{} not found in outputs", input.yellow());
                }
            }
        };
    }
}
