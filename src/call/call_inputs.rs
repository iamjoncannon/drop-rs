use indexmap::IndexMap;

use crate::parser::hcl_block::HclBlock;

use super::DropCall;

impl DropCall {
    pub fn process_input_block(&self, attr: &hcl::Attribute) {
        let mut input_index_map = IndexMap::<String, hcl::Value>::new();

        if let hcl::Expression::Object(expr_as_obj) = attr.expr() {
            for (key, val) in expr_as_obj {
                let key_as_str = key.to_string();

                input_index_map.insert(key_as_str, HclBlock::value_from_expr(val.to_owned()));
            }
        }
    }
}
