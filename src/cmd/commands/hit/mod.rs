use std::pin::Pin;

use crate::{
    call::DropCall,
    cmd::{ctx::CmdContext, dropdown::DropDown, DropCommand},
    interpreter::evaluate::Evaluator,
    parser::{
        drop_block::DropBlock,
        drop_id::{CallType, DropId},
        hcl_block::HclBlock,
        types::{DropBlockType, DropResourceType},
    },
    runner::{drop_run::DropRun, run_pool::RunPool},
};
use colored::Colorize;
use futures::Future;
use hcl::eval::{Context, Evaluate};
use indexmap::IndexMap;

pub mod run_call;
pub mod run_chain;
pub mod run_run;

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

        let drop_runs = match call_type {
            CallType::Hit => self.run_call(),
            CallType::Run => self.run_run(),
            CallType::Chain => self.run_chain(),
        };

        Box::pin(RunPool::runner_pool(drop_runs))
    }
}

impl HitCommand {

    #[log_attributes::log(debug, "{fn} {self:?}")]
    pub fn get_env_scope(&self) -> Context<'static> {
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
        let env_var_scope = env_var_scope_res.unwrap();

        env_var_scope
    }

    #[log_attributes::log(debug, "{fn} {self:?}")]
    pub fn get_drop_block_or_exit(&self, block_drop_id: &str, drop_resource_type: DropResourceType) -> &'static DropBlock {
        let drop_container_res =
            Evaluator::get_selected_container(block_drop_id, drop_resource_type);

        if drop_container_res.is_err() {
            log::trace!("{drop_container_res:?}");
            log::error!(
                "error running block {}: {}",
                self.input_drop_id_string,
                drop_container_res.unwrap_err()
            );
            std::process::exit(1);
        }

        let call_drop_container = drop_container_res.unwrap();

        call_drop_container
    }
}
