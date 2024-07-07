use std::pin::Pin;

use crate::{
    call::DropCall,
    cmd::{ctx::CmdContext, DropCommand},
    interpreter::evaluate::Evaluator,
    parser::{
        drop_block::DropBlock,
        drop_id::{CallType, DropId},
    },
    runner::{drop_run::DropRun, run_pool::RunPool},
};
use colored::Colorize;
use futures::Future;
use hcl::eval::Context;
use indexmap::IndexMap;

#[derive(Debug)]
pub struct HitCommand {
    pub input_drop_id_string: String,
}

impl DropCommand for HitCommand {
    fn announce(&self) {
        println!(
            "hitting {} in environment {}\n",
            self.input_drop_id_string.yellow(),
            CmdContext::get_env().yellow()
        );
    }

    #[log_attributes::log(debug, "{fn} {self:?}")]
    fn run(&self) -> Pin<Box<dyn Future<Output = ()>>> {
        let call_type = DropId::get_call_type_from_raw_drop_id(&self.input_drop_id_string);

        // initialize call- generate single DropRun
        // and run pool

        let drop_run = match call_type {
            CallType::Hit => self.run_call(),
            CallType::Run => self.run_run(),
        };

        Box::pin(RunPool::runner_pool(vec![drop_run]))
    }
}

impl HitCommand {
    fn run_call(&self) -> DropRun {

        // get default variable context

        let mut env_var_scope_res =
            Evaluator::get_module_dependencies_for_eval(&self.input_drop_id_string);

        if env_var_scope_res.is_err() {
            panic!("error running call {}", self.input_drop_id_string);
        }

        let call_drop_container =
            Evaluator::get_selected_container(&self.input_drop_id_string, CallType::Hit);

        if call_drop_container.is_err() {
            panic!("error running call {}", self.input_drop_id_string);
        }

        // for a call, the input index is blank
        let input_index_map = IndexMap::<String, hcl::Value>::new();

        DropRun {
            call_drop_container: call_drop_container.unwrap(),
            input_index_map,
            env_var_scope: env_var_scope_res.unwrap(),
        }
    }

    fn run_run(&self) -> DropRun {

        // get default variable context

        let mut env_var_scope_res =
            Evaluator::get_module_dependencies_for_eval(&self.input_drop_id_string);

        if env_var_scope_res.is_err() {
            panic!("error running call {}", self.input_drop_id_string);
        }

        // get the run container for the drop id 
        
        let run_container = Evaluator::get_selected_container(&self.input_drop_id_string, CallType::Run);

        log::debug!("HitCommand run_run run_container {run_container:?}");

        todo!()
    }
}
