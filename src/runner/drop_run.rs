use hcl::{
    eval::{self, Context},
    Attribute, Block, Value,
};
use indexmap::IndexMap;

use crate::{
    call::DropCall,
    constants::{CALL_AFTER, CALL_ASSERT, CALL_OUTPUT, INPUT_OBJECT_VAR_PREFIX},
    interpreter::{evaluate::Evaluator, scope::Scope},
    parser::{drop_block::DropBlock, drop_id::DropId},
};

/// DropRun manages evaluating the
/// final call block before execution
#[derive(Debug)]
pub struct DropRun {
    pub call_drop_container: &'static DropBlock,
    pub call_block_overwrites: Option<CallBlockOverWrites>,
    pub input_index_map: IndexMap<String, Value>,
    pub env_var_scope: Context<'static>,
    pub depends_on: Option<Vec<String>>,
}

impl DropRun {
    #[log_attributes::log(debug, "{fn}")]
    pub fn get_drop_call(&mut self, inputs_from_dependencies: IndexMap<String, Value>) -> DropCall {
        log::trace!("init DropRun get_drop_call: {self:?} inputs_from_dependencies: {inputs_from_dependencies:?}");

        let evaluated_block = self.evaluate_call_block_with_inputs(inputs_from_dependencies);

        let drop_id: DropId = self.call_drop_container.drop_id.as_ref().unwrap().clone();

        // overwrite assert and output for call block
        let drop_call = match &self.call_block_overwrites {
            Some(call_block_overwrites) => DropCall::from_call_and_run_hcl_block(
                &evaluated_block,
                call_block_overwrites,
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

#[derive(Debug)]
pub struct CallBlockOverWrites {
    pub assert_attr: Option<Attribute>,
    pub output_attr: Option<Attribute>,
    pub after_attr: Option<Attribute>,
}

impl<'a> CallBlockOverWrites {
    pub fn new() -> CallBlockOverWrites {
        CallBlockOverWrites {
            assert_attr: None,
            output_attr: None,
            after_attr: None,
        }
    }

    pub fn get_attr_for_key(
        &'a self,
        evaluated_hcl_block: &'a Block,
        key: &str,
    ) -> Vec<&'a Attribute> {
        let target_attr: Vec<&Attribute> = evaluated_hcl_block
            .body()
            .attributes()
            .into_iter()
            .filter(|attr| attr.key() == key)
            .collect();

        target_attr
    }

    pub fn get_attr_res(&self, target_attr: Vec<&Attribute>) -> Option<Attribute> {
        if target_attr.is_empty() {
            return None;
        }

        Some(target_attr[0].to_owned())
    }

    pub fn from_run_block(&self, evaluated_run_hcl_block: Block) -> CallBlockOverWrites {
        let get_attr =
            |key: &str| self.get_attr_res(self.get_attr_for_key(&evaluated_run_hcl_block, key));

        CallBlockOverWrites {
            assert_attr: get_attr(CALL_ASSERT),
            output_attr: get_attr(CALL_OUTPUT),
            after_attr: get_attr(CALL_AFTER),
        }
    }

    pub fn from_chain_block(&self, evaluated_chain_hcl_block: Block) -> CallBlockOverWrites {
        let get_attr =
            |key: &str| self.get_attr_res(self.get_attr_for_key(&evaluated_chain_hcl_block, key));

        // the chain node is an object so we can
        // assign a key to each output
        // for the call itself, we need to
        // transform into a Vec<hcl::Traversal>
        // hcl::Object<hcl::ObjectKey, hcl::Traversal>

        let output_attr_vec = self.get_attr_for_key(&evaluated_chain_hcl_block, CALL_OUTPUT);

        let mut output_attr: Option<Attribute> = None;

        if !output_attr_vec.is_empty() {
            let attr = output_attr_vec[0];

            if let hcl::Expression::Object(exp_as_obj) = &attr.expr {
                let mut traversals = Vec::new();

                for (key, mut value_as_exp) in exp_as_obj {
                    traversals.push(value_as_exp.to_owned());
                }

                output_attr = Some(hcl::Attribute::new("outputs", traversals))
            }
        };

        CallBlockOverWrites {
            assert_attr: get_attr(CALL_ASSERT),
            output_attr,
            after_attr: get_attr(CALL_AFTER),
        }
    }
}
