use hcl::ObjectKey;
use vecmap::VecMap;

use crate::{assert::types::{Assert, AssertExpectedValue}, parser::hcl_block::HclBlock, record::response_walker::{OutputType, ResponseWalker}};

use super::DropCall;


impl DropCall {
    /// mangages processing the configuration for each assertion
    /// `CallAssertion` manages running the assertions
    /// `CallAssertions` manage the assertion logic itself
    pub fn process_assert_block(&mut self, attr: &hcl::Attribute) {
        let drop_id = &self.drop_id.drop_id().unwrap();
        if let hcl::Expression::Object(assert_block) = attr.expr() {
            for entry in assert_block {
                let (key, value) = entry;
                let traversal_key = DropCall::process_assert_key(key, drop_id);
                let expected_value = DropCall::process_assert_block_value(value, drop_id, key);

                self.asserts.push(Assert {
                    traversal_key,
                    display_name: key.to_string(),
                    expected_value,
                });
            }
        } else {
            // we will never enter this block since we validated previously
            panic!("assert block must be a set of valid assert objects")
        }
    }

    pub fn process_assert_key(key: &hcl::ObjectKey, drop_id: &str) -> hcl::Traversal {
        let throw_msg = format!("error in assert block for {drop_id}, {key} must be a property of- response.body, response.header");
        match key {
            hcl::ObjectKey::Expression(expr) => {
                if let hcl::Expression::Traversal(trav) = expr {
                    if let OutputType::InvalidOutput(err) = ResponseWalker::get_output_variant(trav)
                    {
                        panic!("error in assert block for {drop_id}: {err}")
                    }
                    *trav.to_owned()
                } else {
                    panic!("{throw_msg}");
                }
            }
            hcl::ObjectKey::Identifier(_) => {
                panic!("{throw_msg}");
            }
            _ => panic!("{throw_msg}"),
        }
    }

    pub fn process_assert_block_value(
        assert_block_expected_value: &hcl::Expression,
        drop_id: &str,
        assert_block_target_traversal_key: &ObjectKey,
    ) -> AssertExpectedValue {
        let err_prefix =
            format!("error in {drop_id} assert block at {assert_block_target_traversal_key}");

        match assert_block_expected_value {
            // response.headers["set-cookie"] = assert.exists
            hcl::Expression::Traversal(trav) => DropCall::process_assert_block_value_variable(trav, &err_prefix),

            // response.headers["set-cookie"]   = { assert.contains = "x-session-id" }
            hcl::Expression::Object(obj) => DropCall::process_assert_block_value_object(
                obj,
                drop_id,
                assert_block_target_traversal_key,
                &err_prefix,
            ),

            // response.body.status = "running"
            hcl::Expression::String(_) | hcl::Expression::Number(_) | hcl::Expression::Bool(_) => {
                let try_deserialize_value =
                    HclBlock::hcl_expression_to_serde_value(assert_block_expected_value);

                assert!(try_deserialize_value.is_ok(), "{err_prefix} The value {assert_block_expected_value:?} is not valid for assertion.");

                AssertExpectedValue::Value(try_deserialize_value.unwrap())
            }

            _ => panic!("{err_prefix} The value {assert_block_expected_value:?} is not valid for assertion.")
        }
    }

    pub fn process_assert_block_value_variable(
        trav: &hcl::Traversal,
        err_prefix: &str,
    ) -> AssertExpectedValue {
        AssertExpectedValue::from_traversal(trav, err_prefix, None)
    }

    pub fn process_assert_block_value_object(
        obj: &VecMap<hcl::ObjectKey, hcl::Expression>,
        drop_id: &str,
        assert_block_target_traversal_key: &hcl::ObjectKey,
        err_prefix: &str,
    ) -> AssertExpectedValue {
        assert!(
            obj.keys().len() == 1,
            "error in {drop_id} assert block- only one assertion is allowed per key: {assert_block_target_traversal_key}"
        );

        // assert.contains
        match obj.keys().next().unwrap() {
            hcl::ObjectKey::Expression(expr) => {
                if let hcl::Expression::Traversal(trav) = expr {
                    // validated in above assert
                    let value = obj.values().next().unwrap();
                    let try_deserialize_value = HclBlock::hcl_expression_to_serde_value(value);

                    assert!(
                        try_deserialize_value.is_ok(),
                        "{err_prefix} The value {value:?} is not valid for assertion."
                    );

                    return AssertExpectedValue::from_traversal(
                        trav,
                        err_prefix,
                        Some(try_deserialize_value.unwrap()),
                    );
                }

                panic!("{err_prefix}: {expr:?} invalid")
            }
            _ => panic!("{err_prefix}: {obj:?} invalid assertion value"),
        };
    }
}
