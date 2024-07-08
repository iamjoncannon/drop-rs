use indexmap::IndexMap;

use crate::{
    interpreter::evaluate::Evaluator,
    parser::{
        block_type::chain::ChainNode,
        drop_id::CallType,
        hcl_block::HclBlock,
        types::{DropBlockType, DropResourceType},
    },
    runner::drop_run::{CallBlockOverWrites, DropRun},
};

use super::HitCommand;

impl HitCommand {
    #[log_attributes::log(debug, "{fn} {self:?}")]
    pub fn run_chain(&self) -> Vec<DropRun> {
        let mut env_var_scope = self.get_env_scope();

        let chain_container =
            self.get_drop_block_or_exit(&self.input_drop_id_string, DropResourceType::Chain);

        let mut drop_runs: Vec<DropRun> = Vec::new();

        if let DropBlockType::Chain(chain) = &chain_container.drop_block {
            chain.nodes.iter().for_each(|node| {
                let drop_id_from_string = HclBlock::traversal_to_string(node);

                let node_container =
                    self.get_drop_block_or_exit(&drop_id_from_string, DropResourceType::ChainNode);

                // evaluate variables on node container

                let (mut chain_node, evaluated_chain_node_hcl_block, eval_diagnostics) =
                    Evaluator::evaluate_block_in_env::<ChainNode>(
                        &node_container,
                        &env_var_scope,
                        &drop_id_from_string,
                    );

                let drop_id_of_call_from_string = HclBlock::traversal_to_string(&chain_node.hit);

                let call_drop_container = self
                    .get_drop_block_or_exit(&drop_id_of_call_from_string, DropResourceType::Call);

                let inputs = chain_node.inputs;

                // pull input values from evaluated run block

                let input_index_map = if inputs.is_some() {
                    Evaluator::evaluate_input_block_and_create_index_map(
                        inputs.unwrap(),
                        &mut env_var_scope,
                    )
                } else {
                    IndexMap::<String, hcl::Value>::new()
                };

                let call_block_overwrites = CallBlockOverWrites::new().from_chain_block(evaluated_chain_node_hcl_block);

                let drop_run = DropRun {
                    call_drop_container,
                    call_block_overwrites: Some(call_block_overwrites),
                    input_index_map,
                    depends_on: None,
                    env_var_scope: env_var_scope.clone(),
                };

                drop_runs.push(drop_run);
            });
        }

        drop_runs
    }

    pub fn map_chain_inputs_to_dependencies(&self, inputs: Option<hcl::Expression>){

        // csrf_token = chain.csrf_token


    }
}
