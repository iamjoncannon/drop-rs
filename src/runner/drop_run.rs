use hcl::{eval::Context, Value};
use indexmap::IndexMap;

use crate::{
    call::DropCall,
    interpreter::evaluate::Evaluator,
    parser::{drop_block::DropBlock, drop_id::DropId},
};

/// DropRun manages evaluating the
/// final call block before execution
#[derive(Debug)]
pub struct DropRun {
    pub call_drop_container: &'static DropBlock,
    pub input_index_map: IndexMap<String, Value>,
    pub env_var_scope: Context<'static>,
}

impl DropRun {

    #[log_attributes::log(debug, "{fn}")]
    pub fn get_drop_call(&mut self, inputs_from_dependencies: IndexMap<String, Value>) -> DropCall {

        log::trace!("init DropRun get_drop_call: {self:?} inputs_from_dependencies: {inputs_from_dependencies:?}");

        let evaluated_block = self.evaluate_call_block_with_inputs(inputs_from_dependencies);

        let drop_id: DropId = self.call_drop_container.drop_id.as_ref().unwrap().clone();

        let drop_call = DropCall::from_hcl_block(&evaluated_block, drop_id);

        drop_call
    }

    pub fn evaluate_call_block_with_blank_inputs(&mut self) -> hcl::Block {
        let inputs_from_dependencies = IndexMap::<String, hcl::Value>::new();

        self.evaluate_call_block_with_inputs(inputs_from_dependencies)
    }

    #[log_attributes::log(debug, "{fn}")]
    pub fn evaluate_call_block_with_inputs(&mut self, inputs_from_dependencies: IndexMap<String, Value>) -> hcl::Block {

        log::trace!("DropRun evaluate_call_block_with_inputs");

        // final run may receive input variables from
        // previous runs

        // todo- merge into input_index_map during chain run

        Evaluator::insert_call_defaults_into_index_map(
            &self.call_drop_container,
            &mut self.input_index_map,
            &mut self.env_var_scope,
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
