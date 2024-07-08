use hcl::{eval::Context, Value};
use indexmap::IndexMap;

use crate::{
    call::DropCall,
    constants::INPUT_OBJECT_VAR_PREFIX,
    interpreter::{evaluate::Evaluator, scope::Scope},
    parser::{drop_block::DropBlock, drop_id::DropId},
};

/// DropRun manages evaluating the
/// final call block before execution
#[derive(Debug)]
pub struct DropRun {
    pub call_drop_container: &'static DropBlock,
    pub run_drop_container: Option<&'static DropBlock>,
    pub input_index_map: IndexMap<String, Value>,
    pub env_var_scope: Context<'static>,
}

impl DropRun {
    #[log_attributes::log(debug, "{fn}")]
    pub fn get_drop_call(&mut self, inputs_from_dependencies: IndexMap<String, Value>) -> DropCall {
        log::trace!("init DropRun get_drop_call: {self:?} inputs_from_dependencies: {inputs_from_dependencies:?}");

        let evaluated_block = self.evaluate_call_block_with_inputs(inputs_from_dependencies);

        let drop_id: DropId = self.call_drop_container.drop_id.as_ref().unwrap().clone();

        // overwrite assert and output for call block
        let drop_call = match self.run_drop_container {
            Some(run_drop_container) => DropCall::from_call_and_run_hcl_block(
                &evaluated_block,
                &run_drop_container.hcl_block.as_ref().unwrap(),
                drop_id,
            ),
            None => DropCall::from_call_hcl_block(&evaluated_block, drop_id),
        };

        drop_call
    }

    pub fn evaluate_call_block_with_blank_inputs(&mut self) -> hcl::Block {
        let inputs_from_dependencies = IndexMap::<String, hcl::Value>::new();

        self.evaluate_call_block_with_inputs(inputs_from_dependencies)
    }

    #[log_attributes::log(debug, "{fn}")]
    pub fn evaluate_call_block_with_inputs(
        &mut self,
        inputs_from_dependencies: IndexMap<String, Value>,
    ) -> hcl::Block {
        // todo- merge inputs_from_dependencies with self.input_index_map

        log::trace!("DropRun evaluate_call_block_with_inputs");

        Scope::insert_object_into_hcl_context(
            &mut self.env_var_scope,
            INPUT_OBJECT_VAR_PREFIX,
            &self.input_index_map,
        );

        log::debug!(
            "DropRun evaluate_call_block_with_inputs env_var_scope {:?}",
            self.env_var_scope
        );

        let (evaluated_block, eval_diagnostics) = Evaluator::evaluate_call_block_in_env(
            &self.call_drop_container,
            &mut self.env_var_scope,
        );

        log::trace!("DropRun eval_diagnostics {eval_diagnostics:?}");

        if eval_diagnostics.is_err() {
            eval_diagnostics.panic();
        }

        (evaluated_block)
    }
}
