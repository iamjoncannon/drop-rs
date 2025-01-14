use super::{
    diagnostics::EvalDiagnostics,
    scope::{GlobalScopeProvider, Scope},
};
use crate::{
    call::DropCall,
    cmd::ctx::CmdContext,
    constants::MOD_OBJECT_VAR_PREFIX,
    parser::{
        self,
        block_type::run::RunBlock,
        drop_block::DropBlock,
        drop_id::{CallType, DropId},
        hcl_block::{HclBlock, HclObject},
        types::{DropBlockType, DropResourceType},
        GlobalDropConfigProvider,
    },
};
use colored::Colorize;
use hcl::{
    eval::{Context, Evaluate},
    Value,
};
use indexmap::IndexMap;
use log::trace;
use log_derive::logfn;

/// methods to evaluate hcl blocks
/// with specific environment variables
/// provided by user configuration
pub struct Evaluator {}

impl Evaluator {
    #[logfn(
        ok = "TRACE",
        err = "ERROR",
        fmt = "get_module_dependencies_for_eval: {:?}",
        log_ts = true
    )]
    pub fn get_module_dependencies_for_eval<'l>(
        target_drop_id: &str,
    ) -> Result<Context<'l>, anyhow::Error> {
        // get selected module structure
        let selected_module_block = Evaluator::get_selected_module_block(target_drop_id)?;

        // add module to context

        let mut env_var_scope = GlobalScopeProvider::get_mut()?;

        Evaluator::generate_module_context(selected_module_block, &mut env_var_scope);

        Ok(env_var_scope)
    }

    #[logfn(
        ok = "TRACE",
        err = "ERROR",
        fmt = "get_selected_module_block: {:?}",
        log_ts = true
    )]
    pub fn get_selected_module_block(drop_id: &str) -> Result<&'static DropBlock, anyhow::Error> {
        let drop_config = GlobalDropConfigProvider::get();

        let module_from_drop_id = DropId::get_module_from_drop_id(drop_id);

        let modules = &drop_config.modules;

        let matched_modules: Vec<&'static DropBlock> = modules
            .iter()
            .filter(|each| each.drop_id.as_ref().unwrap().resource_name == module_from_drop_id)
            .collect();

        // todo- handle errors

        Ok(matched_modules[0])
    }

    pub fn get_selected_container(
        target_drop_id: &str,
        drop_resource_type: DropResourceType,
    ) -> Result<&'static DropBlock, anyhow::Error> {
        let global_config = GlobalDropConfigProvider::get();

        let vector = match drop_resource_type {
            DropResourceType::Call => &global_config.hits,
            DropResourceType::Module => &global_config.modules,
            DropResourceType::Environment => &global_config.environments,
            DropResourceType::Run => &global_config.runs,
            DropResourceType::Chain => &global_config.chains,
            DropResourceType::ChainNode => &global_config.chain_nodes,
        };

        let matched_call: Vec<&'static DropBlock> = vector
            .iter()
            .filter(|each| each.drop_id.as_ref().unwrap().drop_id().unwrap() == target_drop_id)
            .collect();

        assert!(
            !matched_call.is_empty(),
            "No block found for {}",
            target_drop_id.yellow()
        );

        assert!(
            matched_call.len() == 1,
            "Multiple blocks found for {}",
            target_drop_id.yellow()
        );

        Ok(matched_call[0])
    }

    pub fn generate_module_context(
        module_container: &DropBlock,
        variable_context: &mut Context<'_>,
    ) {
        // evaluate module in current scope
        let block_for_module_opt = &module_container.drop_block;

        let mut env_map: IndexMap<String, Value> = IndexMap::new();

        // add module values to current scope
        if let DropBlockType::Module(object_block) = block_for_module_opt {
            Evaluator::evaluate_and_insert_values_from_object_body_into_context(
                object_block.as_ref().unwrap(),
                &mut env_map,
                &module_container.file_name,
                variable_context,
            );
        } else {
            trace!("entering _ block in block_for_module_opt match: {module_container:?}");
        }

        variable_context.declare_var(MOD_OBJECT_VAR_PREFIX, env_map);
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

    #[log_attributes::log(trace, "exit {fn}")]
    pub fn evaluate_input_block_and_create_index_map(
        inputs: hcl::Expression,
        env_var_scope: &mut Context<'_>,
    ) -> IndexMap<String, hcl::Value> {
        let mut input_index_map = IndexMap::<String, hcl::Value>::new();

        if let hcl::Expression::Object(exp_as_obj) = inputs {
            for (key, mut value_as_exp) in exp_as_obj {
                let _ = value_as_exp.evaluate_in_place(&env_var_scope);
                input_index_map.insert(key.to_string(), HclBlock::value_from_expr(value_as_exp));
            }
        }

        input_index_map
    }

    #[log_attributes::log(trace, "exit {fn}")]
    pub fn evaluate_call_block_in_env(
        call_block: &DropBlock,
        env_var_scope: &mut Context<'_>,
    ) -> (hcl::Block, EvalDiagnostics) {
        let drop_id = &call_block.drop_id.as_ref().unwrap();

        // we have to clone here because evaluate in place
        // requires mut borrow
        let hcl_block = call_block.hcl_block.clone();

        let file_name = &call_block.file_name;

        match hcl_block {
            Some(mut block) => {
                let eval_diagnostics = Evaluator::evaluate_user_defined_block_with_ctx(
                    &mut block,
                    env_var_scope,
                    file_name,
                );

                log::trace!("Evaluator::evaluate_call_block_in_env {block:?} {eval_diagnostics:?}");

                (block, eval_diagnostics)
            }
            _ => panic!("failure evaluating hcl block {drop_id:?} in file {file_name}"),
        }
    }

    pub fn evaluate_block_in_env<DropBlockType: for<'de> serde::Deserialize<'de>>(
        drop_block_container: &DropBlock,
        env_var_scope: &Context<'_>,
        run_drop_id: &str,
    ) -> (DropBlockType, hcl::Block, EvalDiagnostics) {
        // evaluate run hcl block to support parameterizing inputs
        let mut hcl_block = drop_block_container.hcl_block.as_ref().unwrap().to_owned();

        let errors_from_evaluate_call = hcl_block.evaluate_in_place(env_var_scope);

        let mut diag = EvalDiagnostics::new(&drop_block_container.file_name);

        if errors_from_evaluate_call.is_err() {
            let errors = errors_from_evaluate_call.unwrap_err();

            // todo- enable evaluate errors for run or chain node
            // diag.evaluate_errors(&errors);
        }

        // generate drop block again to extract inputs
        let block_res: Result<DropBlockType, hcl::Error> = hcl::from_body(hcl_block.body().to_owned());

        match block_res {
            Ok(run_block) => (run_block, hcl_block, diag),
            Err(err) => {
                panic!(
                    "error processing run block {}: {}",
                    run_drop_id.yellow(),
                    err
                )
            }
        }
    }

    #[log_attributes::log(trace, "exit {fn}")]
    fn evaluate_user_defined_block_with_ctx(
        block: &mut hcl::Block,
        ctx: &Context<'_>,
        file_name: &str,
    ) -> EvalDiagnostics {
        log::trace!("init Evaluator evaluate_user_defined_block_with_ctx ");

        let errors_from_evaluate_call = block.evaluate_in_place(ctx);

        let mut diag = EvalDiagnostics::new(file_name);

        if errors_from_evaluate_call.is_err() {
            let errors = errors_from_evaluate_call.unwrap_err();

            log::trace!("Evaluator.evaluate_user_defined_block_with_ctx errors {errors:?}");

            diag.evaluate_errors(&errors);
        }

        diag
    }
}
