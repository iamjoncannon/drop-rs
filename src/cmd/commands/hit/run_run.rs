use crate::{
    call::DropCall,
    cmd::{ctx::CmdContext, dropdown::DropDown, DropCommand},
    interpreter::evaluate::Evaluator,
    parser::{
        block_type::run::RunBlock, drop_block::DropBlock, drop_id::{CallType, DropId}, hcl_block::HclBlock, types::{DropBlockType, DropResourceType}
    },
    runner::{drop_run::{CallBlockOverWrites, DropRun}, run_pool::RunPool},
};
use colored::Colorize;
use futures::Future;
use hcl::eval::{Context, Evaluate};
use indexmap::IndexMap;

use super::HitCommand;

impl HitCommand {
    
    #[log_attributes::log(debug, "{fn} {self:?}")]
    pub fn run_run(&self) -> Vec<DropRun> {

        let mut env_var_scope = self.get_env_scope();

        // get the run container for the drop id

        let run_drop_container = self.get_drop_block_or_exit(&self.input_drop_id_string, DropResourceType::Run);

        log::debug!("HitCommand run_run run_container {run_drop_container:?}");

        // evaluate run hcl block with current env scope

        let (mut run_block, evaluated_run_hcl_block, eval_diagnostics) =
            Evaluator::evaluate_block_in_env::<RunBlock>(
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

        let call_drop_container = self.get_drop_block_or_exit(call_drop_id, DropResourceType::Call);

        let call_block_overwrites = CallBlockOverWrites::new().from_run_block(evaluated_run_hcl_block);

        vec![DropRun {
            call_drop_container,
            call_block_overwrites: Some(call_block_overwrites),
            input_index_map,
            env_var_scope,
            depends_on: None,
        }]
    }
}