use std::pin::Pin;

use crate::{
    call::DropCall,
    cmd::{ctx::CmdContext, dropdown::DropDown, DropCommand},
    interpreter::evaluate::Evaluator,
    parser::{
        drop_block::DropBlock,
        drop_id::{CallType, DropId},
        hcl_block::HclBlock,
        types::DropBlockType,
    },
    runner::{drop_run::DropRun, run_pool::RunPool},
};
use colored::Colorize;
use futures::Future;
use hcl::eval::{Context, Evaluate};
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

    #[log_attributes::log(debug, "{fn}")]
    fn run(&mut self) -> Pin<Box<dyn Future<Output = ()>>> {
        let drop_id = DropDown::drop_down(&self.input_drop_id_string);

        self.input_drop_id_string = drop_id;

        let call_type = DropId::get_call_type_from_raw_drop_id(&self.input_drop_id_string);

        // initialize call- generate single DropRun
        // and run pool

        let drop_runs = match call_type {
            CallType::Hit => self.run_call(),
            CallType::Run => self.run_run(),
            CallType::Chain => todo!(),
        };

        Box::pin(RunPool::runner_pool(drop_runs))
    }
}

impl HitCommand {
    #[log_attributes::log(debug, "{fn} {self:?}")]
    pub fn run_call(&self) -> Vec<DropRun> {
        // get default variable context

        let mut env_var_scope_res =
            Evaluator::get_module_dependencies_for_eval(&self.input_drop_id_string);

        if env_var_scope_res.is_err() {
            panic!("error running call {}", self.input_drop_id_string);
        }

        let mut env_var_scope = env_var_scope_res.unwrap();

        let call_drop_container_res =
            Evaluator::get_selected_container(&self.input_drop_id_string, CallType::Hit);

        if call_drop_container_res.is_err() {
            panic!("error running call {}", self.input_drop_id_string);
        }

        let call_drop_container = call_drop_container_res.unwrap();

        let mut input_index_map = IndexMap::<String, hcl::Value>::new();

        if let DropBlockType::Call(call_block) = &call_drop_container.drop_block {
            if let Some(inputs) = &call_block.inputs {
                input_index_map = Evaluator::evaluate_input_block_and_create_index_map(
                    inputs.clone(),
                    &mut env_var_scope,
                );
            };
        }

        log::debug!("input_index_map {input_index_map:?}");

        vec![DropRun {
            call_drop_container,
            input_index_map,
            env_var_scope,
            run_drop_container: None
        }]
    }

    pub fn run_run(&self) -> Vec<DropRun> {
        // get default variable context

        let mut env_var_scope_res =
            Evaluator::get_module_dependencies_for_eval(&self.input_drop_id_string);

        if env_var_scope_res.is_err() {
            log::trace!("{env_var_scope_res:?}");
            log::error!(
                "error running call {} {}",
                self.input_drop_id_string,
                env_var_scope_res.unwrap_err()
            );
            std::process::exit(1);
        }

        // get the run container for the drop id

        let run_container_res =
            Evaluator::get_selected_container(&self.input_drop_id_string, CallType::Run);

        if run_container_res.is_err() {
            log::trace!("{run_container_res:?}");
            log::error!(
                "error running call {} {}",
                self.input_drop_id_string,
                run_container_res.unwrap_err()
            );
            std::process::exit(1);
        }

        log::debug!("HitCommand run_run run_container {run_container_res:?}");

        // evaluate run hcl block with current env scope

        let mut env_var_scope = env_var_scope_res.unwrap();

        let run_drop_container = run_container_res.unwrap();

        let (mut run_block, evaluated_run_hcl_block, eval_diagnostics) =
            Evaluator::evaluate_run_block_in_env(
                &run_drop_container,
                &env_var_scope,
                &self.input_drop_id_string,
            );

        let call_drop_id = &run_block.get_drop_id_of_hit();

        let inputs = run_block.inputs;

        // pull input values from evaluated run block

        let input_index_map = Evaluator::evaluate_input_block_and_create_index_map(
            inputs,
            &mut env_var_scope,
        );

        // get call container referenced on run block

        let call_drop_container_res =
            Evaluator::get_selected_container(call_drop_id, CallType::Hit);

        if call_drop_container_res.is_err() {
            log::trace!("{call_drop_container_res:?}");
            log::error!(
                "error running call {}: {}",
                self.input_drop_id_string,
                call_drop_container_res.unwrap_err()
            );
            std::process::exit(1);
        }

        let call_drop_container = call_drop_container_res.unwrap();

        // replace call outputs with run outputs

        // replace call asserts with run asserts 

        // the runner will evaluate the final call block

        vec![DropRun {
            call_drop_container,
            run_drop_container: Some(run_drop_container),
            input_index_map,
            env_var_scope,
        }]
    }

    pub fn run_chain(&self) -> Vec<DropRun> {
        todo!()
    }
}
