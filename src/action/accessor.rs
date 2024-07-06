use std::collections::HashMap;
use colored::Colorize;
use hcl::expr::Traversal;

use super::ActionValue;

pub struct PostActionAccessor{}

impl PostActionAccessor {
    pub fn get_string_or_panic(action_config: &HashMap<String, ActionValue>, key: &str) -> String {
        match action_config.get(key) {
            Some(ActionValue::String(st)) => st.to_string(),
            _ => panic!("invalid value passed for {}", key.yellow()),
        }
    }
    
    pub fn get_string_or_default(
        action_config: &HashMap<String, ActionValue>,
        key: &str,
        default: &str,
    ) -> String {
        let passed = action_config.get(key);
    
        if let Some(av) = passed {
            match av {
                ActionValue::String(st) => st.to_string(),
                _ => panic!("invalid value passed for {}", key.yellow()),
            }
        } else {
            log::warn!(
                "no value set for {} in post action. Defaulting to {} ",
                key.yellow(),
                default.yellow()
            );
            default.to_string()
        }
    }
    
    pub fn get_trav_or_panic<'a>(
        action_config: &'a HashMap<String, ActionValue>,
        key: &'a str,
    ) -> &'a Traversal {
        if let Some(ActionValue::Traversal(t)) = action_config.get(key) {
            return t;
        }
        println!("{key:#?} not found in set action");
        panic!();
    }
    
    pub fn get_bool_or_panic<'a>(
        action_config: &'a HashMap<String, ActionValue>,
        key: &'a str,
    ) -> bool {
        if let ActionValue::Bool(t) = action_config.get(key).unwrap() {
            return *t;
        }
        println!("{key:#?} not found in set action");
        panic!()
    }
}