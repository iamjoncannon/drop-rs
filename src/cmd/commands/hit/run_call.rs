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

use super::HitCommand;

impl HitCommand {
    #[log_attributes::log(debug, "{fn} {self:?}")]
    pub fn run_call(&self) -> Vec<DropRun> {

        let mut env_var_scope = self.get_env_scope();

        let call_drop_container = self.get_drop_block_or_exit(&self.input_drop_id_string, DropResourceType::Call);

        let mut input_index_map = IndexMap::<String, hcl::Value>::new();

        if let DropBlockType::Call(call_block) = &call_drop_container.drop_block {
            if let Some(inputs) = &call_block.inputs {
                input_index_map = Evaluator::evaluate_input_block_and_create_index_map(
                    inputs.clone(),
                    &mut env_var_scope,
                );
            };
        }

        log::debug!("run_call input_index_map {input_index_map:?}");

        vec![DropRun {
            call_drop_container,
            input_index_map,
            env_var_scope,
            depends_on: None,
            call_block_overwrites: None,
        }]
    }
}
