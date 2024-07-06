use std::pin::Pin;

use crate::{
    call::DropCall,
    cmd::{ctx::CmdContext, DropCommand},
    interpreter::evaluate::Evaluator,
    parser::{drop_block::DropBlock, drop_id::DropId},
    runner::{drop_run::DropRun, run_pool::RunPool},
};
use colored::Colorize;
use futures::Future;
use hcl::eval::Context;
use indexmap::IndexMap;

pub struct HitCommand {
    pub input_drop_id_string: String,
}

impl HitCommand {
    fn get_call_dependencies<'a>(&self) -> Result<(Context<'a>, &'a DropBlock), anyhow::Error> {
        let mut env_var_scope_res =
            Evaluator::get_module_dependencies_for_eval(&self.input_drop_id_string)?;

        let call_drop_container =
            Evaluator::get_selected_call_container(&self.input_drop_id_string)?;

        Ok((env_var_scope_res, call_drop_container))
    }
}

impl DropCommand for HitCommand {
    fn announce(&self) {
        log::info!(
            "hitting {} in environment {}\n",
            self.input_drop_id_string.yellow(),
            CmdContext::get_env().yellow()
        );
    }

    fn run(&self) -> Pin<Box<dyn Future<Output = ()>>> {
        // initialize call- generate single DropRun
        // and run pool

        let dependency_res = self.get_call_dependencies();

        if dependency_res.is_err() {
            panic!("error running call {}", self.input_drop_id_string);
        }

        let (env_var_scope, call_drop_container) = dependency_res.unwrap();

        let input_index_map = IndexMap::<String, hcl::Value>::new();

        let drop_run = DropRun {
            call_drop_container,
            input_index_map,
            env_var_scope,
        };

        Box::pin(RunPool::runner_pool(vec![drop_run]))
    }
}
