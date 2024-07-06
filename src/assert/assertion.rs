use colored::Colorize;
use isahc::http::HeaderMap;

use crate::{parser::hcl_block::HclBlock, record::response_walker::{JsonWalkError, OutputType, ResponseWalker}};

use super::{
    assertions::{ArrayLengthAssertion, Assertion, NumericalAssertion, StringAssertion},
    types::{
        Assert, AssertExpectedValue, AssertExpectedValueObjectOperator, AssertResult,
        AssertResultOutcome, AssertResultOutcomeTestFailure, AssertionError,
    },
};

pub struct CallAssertion {}

/// manages running the assertion
impl CallAssertion {
    pub fn run_assertions(
        drop_id: &str,
        asserts: Vec<Assert>,
        response_string: &str,
        response_headers: &HeaderMap,
    ) {
        println!("\n{drop_id} {}", "assertions".yellow());

        let mut assert_results = Vec::<AssertResult>::new();
        for assert in asserts {
            let assert_result_outcome =
                CallAssertion::run_assertion(drop_id, &assert, response_string, response_headers);

            assert_results.push({
                AssertResult {
                    assert,
                    outcome: assert_result_outcome,
                }
            });
        }

        if !assert_results.is_empty() {
            AssertResult::report_results(assert_results);
        }
    }

    fn run_assertion(
        drop_id: &str,
        assert: &Assert,
        response_string: &str,
        response_headers: &HeaderMap,
    ) -> AssertResultOutcome {
        let traversal_to_test = &assert.traversal_key;

        let output_variant = ResponseWalker::get_output_variant(traversal_to_test);

        match output_variant {
            OutputType::EntireBody | OutputType::Body => CallAssertion::run_assertion_obj(
                assert,
                drop_id,
                response_string,
                traversal_to_test,
            ),
            OutputType::InvalidOutput(err) => {
                AssertResultOutcome::FailureOnError(AssertionError::InvalidOutputError {
                    msg: err.to_string(),
                })
            }
            OutputType::EntireHeader | OutputType::Header => CallAssertion::run_assertion_headers(
                drop_id,
                assert,
                response_headers,
                traversal_to_test,
            ),
        }
    }

    fn run_assertion_obj(
        assert: &Assert,
        drop_id: &str,
        response_string: &str,
        traversal_to_test: &hcl::Traversal,
    ) -> AssertResultOutcome {
        let response_body_as_possible_json =
            ResponseWalker::deserialize_response_json(response_string);

        match response_body_as_possible_json {
            Ok(response_value) => {
                let walk_result =
                    ResponseWalker::walk_json_output(&response_value, traversal_to_test, drop_id);

                match walk_result {
                    Ok(response_walker_result) => {
                        let actual_result_value = response_walker_result.result_value;

                        CallAssertion::run_assertion_value(actual_result_value, assert)
                    }
                    Err(walk_err) => CallAssertion::handle_walk_err_assertions(assert, &walk_err),
                }
            }
            Err(err) => AssertResultOutcome::FailureOnError(AssertionError::DeserializationError {
                msg: format!("deserialization error: {err}"),
            }),
        }
    }

    fn run_assertion_headers(
        drop_id: &str,
        assert: &Assert,
        response_headers: &HeaderMap,
        traversal_to_test: &hcl::Traversal,
    ) -> AssertResultOutcome {
        let walk_result = ResponseWalker::get_response_header_value(
            response_headers,
            traversal_to_test,
            drop_id.to_string(),
        );

        match walk_result {
            Ok(header_result_value) => {
                CallAssertion::run_assertion_value(header_result_value, assert)
            }
            Err(walk_err) => CallAssertion::handle_walk_err_assertions(assert, &walk_err),
        }
    }

    fn run_assertion_value(
        result_value: serde_json::Value,
        assert: &Assert,
    ) -> AssertResultOutcome {
        match &assert.expected_value {
            AssertExpectedValue::Value(expected_value) => {
                if result_value == *expected_value {
                    return AssertResultOutcome::Success;
                }
                AssertResultOutcome::TestFailure(AssertResultOutcomeTestFailure {
                    expected_value: expected_value.to_string(),
                    actual_result_value: result_value.to_string(),
                })
            }
            AssertExpectedValue::AssertValueObject(expected_value_object) => {
                match expected_value_object {
                    AssertExpectedValueObjectOperator::Contains(expected_value)
                    | AssertExpectedValueObjectOperator::StartsWith(expected_value) => {
                        StringAssertion::run(
                            result_value,
                            expected_value.clone(),
                            expected_value_object,
                        )
                    }
                    AssertExpectedValueObjectOperator::Length(expected_value) => {
                        ArrayLengthAssertion::run(
                            result_value,
                            expected_value.clone(),
                            expected_value_object,
                        )
                    }
                    AssertExpectedValueObjectOperator::Exist => AssertResultOutcome::Success,
                    AssertExpectedValueObjectOperator::NotExist => {
                        AssertResultOutcome::TestFailure(AssertResultOutcomeTestFailure {
                            expected_value: HclBlock::traversal_to_string(&assert.traversal_key),
                            actual_result_value: String::new(),
                        })
                    }
                    AssertExpectedValueObjectOperator::LessThan(expected_value)
                    | AssertExpectedValueObjectOperator::GreaterThan(expected_value) => {
                        NumericalAssertion::run(
                            result_value,
                            expected_value.clone(),
                            expected_value_object,
                        )
                    }
                }
            }
        }
    }

    fn handle_walk_err_assertions(
        assert: &Assert,
        walk_err: &JsonWalkError<'_>,
    ) -> AssertResultOutcome {
        match &assert.expected_value {
            AssertExpectedValue::AssertValueObject(expected_value_object) => {
                match expected_value_object {
                    AssertExpectedValueObjectOperator::Exist => {
                        AssertResultOutcome::TestFailure(AssertResultOutcomeTestFailure {
                            expected_value: HclBlock::traversal_to_string(&assert.traversal_key),
                            actual_result_value: String::new(),
                        })
                    }
                    AssertExpectedValueObjectOperator::NotExist => AssertResultOutcome::Success,
                    _ => AssertResultOutcome::FailureOnError(AssertionError::WalkError {
                        msg: format!("walk_err: {walk_err}"),
                    }),
                }
            }
            AssertExpectedValue::Value(_) => {
                AssertResultOutcome::FailureOnError(AssertionError::WalkError {
                    msg: format!("walk_err: {walk_err}"),
                })
            }
        }
    }
}
