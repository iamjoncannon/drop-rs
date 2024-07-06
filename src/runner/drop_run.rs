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

    fn evaluate(&mut self, inputs_from_dependencies: IndexMap<String, Value>) -> DropCall {

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

        if eval_diagnostics.is_err() {
            eval_diagnostics.panic();
        }

        let drop_id: DropId = self.call_drop_container.drop_id.as_ref().unwrap().clone();

        let drop_call = DropCall::from_hcl_block(&evaluated_block, drop_id);

        drop_call
    }
}
