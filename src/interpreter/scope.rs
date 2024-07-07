use std::{collections::HashSet, sync::OnceLock};
use anyhow::anyhow;
use hcl::{
    eval::{Context, Evaluate},
    Value,
};
use indexmap::IndexMap;
use log_derive::logfn;

use crate::{
    constants::*,
    parser::{
        constants::GLOBAL_MOD_BLOCK_KEY,
        drop_block::{DropBlock, DropBlockType},
        hcl_block::{HclBlock, HclObject},
        GlobalDropConfigProvider,
    },
};

use super::global_interpreter_context::GlobalInterpreterContext;

static GLOBAL_SCOPE_PROVIDER: OnceLock<Context> = OnceLock::new();

pub struct GlobalScopeProvider{}

impl GlobalScopeProvider{
    pub fn set(scope: Context<'static>){
        let cell_set_result = GLOBAL_SCOPE_PROVIDER.set(scope);
        if cell_set_result.is_err() {
            log::error!(
                "Error setting global config provider: {:?}",
                cell_set_result.unwrap_err()
            )
        }
    }

    pub fn get_mut() -> Result<Context<'static>, anyhow::Error> {
        let res = GLOBAL_SCOPE_PROVIDER.get();

        if let Some(ctx) = res {
            Ok(ctx.to_owned())
        } else {
            Err(anyhow!("error accessing global cmd ctx"))
        }
    }
}

#[derive(Debug)]
pub struct Scope {}

impl Scope {
    #[logfn(
        ok = "TRACE",
        err = "ERROR",
        fmt = "Resolved Variable Scope: {:?}",
        log_ts = true
    )]
    pub fn evaluate_variable_scope<'l>(
        secrets_hash_for_env: IndexMap<String, Value>,
        user_selected_env: &str,
    ) -> Result<Context<'l>, anyhow::Error> {
        let global_drop_config = GlobalDropConfigProvider::get();
        let module_blocks = &global_drop_config.modules;

        let mut global_variable_context = GlobalInterpreterContext::create();

        Scope::insert_object_into_hcl_context(
            &mut global_variable_context,
            SECRET_OBJECT_VAR_PREFIX,
            &secrets_hash_for_env,
        );

        let global_blocks: Vec<&DropBlock> = module_blocks
            .iter()
            .filter(|block_ref| {
                let rn = block_ref.drop_id.as_ref().unwrap().resource_name.to_string();
                rn == GLOBAL_MOD_BLOCK_KEY
            })
            .collect();

            Scope::insert_global_values_into_hcl_context(
            &mut global_variable_context,
            &global_blocks,
        );

        let env_blocks = &global_drop_config.environments;

        let env_blocks_for_this_env: Vec<&DropBlock> = env_blocks
            .iter()
            .filter(|b| {
                let rn = b.drop_id.as_ref().unwrap().resource_name.to_string();
                rn == user_selected_env
            })
            .collect();

        let env_block_for_this_env = match env_blocks_for_this_env.len() {
            0 => {
                if user_selected_env != BASE_ENVIRONMENT_TITLE {
                    panic!("no env block found for env {user_selected_env}");
                }

                None
            }
            1 => Some(env_blocks_for_this_env[0]),
            _ => panic!("Multiple blocks found for env {user_selected_env}. Only one is allowed."),
        };

        Scope::insert_current_env_into_hcl_context(
            &mut global_variable_context,
            env_block_for_this_env,
        );

        Ok(global_variable_context)
    }

    pub fn insert_object_into_hcl_context(
        variable_context: &mut Context<'_>,
        entry_key: &str,
        secret_map: &IndexMap<String, Value>,
    ) {
        let secret_value = Value::Object(secret_map.to_owned());
        variable_context.declare_var(entry_key, secret_value);
    }

    pub fn insert_global_values_into_hcl_context(
        variable_context: &mut Context<'_>,
        global_blocks: &[&DropBlock],
    ) {
        let mut secret_hash = HashSet::<String>::new();

        for global_block in global_blocks {
            let drop_block = &global_block.drop_block;

            let file_name = &global_block.file_name;

            match drop_block {
                DropBlockType::Module(module_drop_block) => {
                    if let Some(hcl_object) = module_drop_block {
                        hcl_object.iter().for_each(|(k, v)| {
                            let key_as_str = k.to_string();

                            assert!(!secret_hash.contains(&key_as_str), "global mod contains multiple entries for key: {key_as_str}. Second in: {file_name}.");

                            let mut expr = v.to_owned();

                            // evaluate expression from current context with secrets
                            let eval_result = expr.evaluate_in_place(variable_context);

                            if eval_result.is_err() {
                                let msg = eval_result.err().unwrap().to_string();
                                panic!("error evaluating {key_as_str}: {msg}")
                            }

                            let val_as_hcl_val = HclBlock::value_from_expr(expr);

                            secret_hash.insert(key_as_str.clone());

                            variable_context.declare_var(key_as_str, val_as_hcl_val);
                        });
                    }
                }
                _ => {
                    std::process::exit(1);
                }
            }
        }
    }

    pub fn insert_current_env_into_hcl_context(
        variable_context: &mut Context<'_>,
        current_env_module_block: Option<&DropBlock>,
    ) {
        let mut env_map: IndexMap<String, Value> = IndexMap::new();

        if current_env_module_block.is_none() {
            variable_context.declare_var(ENV_OBJECT_VAR_PREFIX, env_map);
            return;
        }

        let current_env_module_block = current_env_module_block.unwrap();

        let block_itself = &current_env_module_block.drop_block;

        let file_name = &current_env_module_block.file_name;

        match block_itself {
            DropBlockType::Environment(object_block) => {
                Scope::evaluate_and_insert_values_from_object_body_into_context(
                    object_block,
                    &mut env_map,
                    file_name,
                    variable_context,
                );
            }
            _ => panic!("error resolving module block"),
        }

        variable_context.declare_var(ENV_OBJECT_VAR_PREFIX, env_map);
    }

    pub fn evaluate_and_insert_values_from_object_body_into_context(
        object_block: &HclObject,
        env_map: &mut IndexMap<String, Value>,
        file_name: &str,
        variable_context: &mut Context<'_>,
    ) {
        object_block.iter().for_each(|(k, v)| {
            let key_as_str = k.to_string();

            assert!(!env_map.contains_key(&key_as_str), "environment block contains multiple entries for key: {key_as_str}. Second in: {file_name}.");

            let mut expr = v.to_owned();

            // evaluate expression from secrets
            let eval_result = expr.evaluate_in_place(variable_context);

            if eval_result.is_err() {
                let msg = eval_result.err().unwrap().to_string();
                panic!("{file_name} error evaluating {key_as_str}: {msg}")
            }

            let val_as_hcl_val = HclBlock::value_from_expr(expr);

            env_map.insert(key_as_str.clone(), val_as_hcl_val);

        });
    }
}
