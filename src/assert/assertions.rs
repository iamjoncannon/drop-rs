use super::types::{
    AssertExpectedValueObjectOperator, AssertResultOutcome, AssertResultOutcomeTestFailure,
    AssertionError,
};

/// encapsulate logic for individual assertions
pub trait Assertion {
    fn run(
        actual_result_value: serde_json::Value,
        expected_value: serde_json::Value,
        operator: &AssertExpectedValueObjectOperator,
    ) -> AssertResultOutcome;
}

pub struct ContainAssertion {}

impl Assertion for ContainAssertion {
    fn run(
        actual_result_value: serde_json::Value,
        expected_value: serde_json::Value,
        _operator: &AssertExpectedValueObjectOperator,
    ) -> AssertResultOutcome {
        let res_as_str = actual_result_value.as_str();
        let expected_value_as_str = expected_value.as_str();

        if res_as_str.is_some()
            && expected_value_as_str.is_some()
            && res_as_str.unwrap().contains(expected_value_as_str.unwrap())
        {
            return AssertResultOutcome::Success;
        }

        AssertResultOutcome::TestFailure(AssertResultOutcomeTestFailure {
            expected_value: expected_value.to_string(),
            actual_result_value: actual_result_value.to_string(),
        })
    }
}

pub struct StringAssertion {}

impl Assertion for StringAssertion {
    fn run(
        actual_result_value: serde_json::Value,
        expected_value: serde_json::Value,
        operator: &AssertExpectedValueObjectOperator,
    ) -> AssertResultOutcome {
        let res_as_str = actual_result_value.as_str();
        let expected_value_as_str = expected_value.as_str();

        if res_as_str.is_some() && expected_value_as_str.is_some() {
            let result = match operator {
                AssertExpectedValueObjectOperator::Contains(_) => {
                    res_as_str.unwrap().contains(expected_value_as_str.unwrap())
                }
                AssertExpectedValueObjectOperator::StartsWith(_) => res_as_str
                    .unwrap()
                    .starts_with(expected_value_as_str.unwrap()),
                _ => todo!("will never enter this path"),
            };

            if result {
                return AssertResultOutcome::Success;
            }
        }

        AssertResultOutcome::TestFailure(AssertResultOutcomeTestFailure {
            expected_value: expected_value.to_string(),
            actual_result_value: actual_result_value.to_string(),
        })
    }
}

pub struct NumericalAssertion {}

impl Assertion for NumericalAssertion {
    fn run(
        actual_result_value: serde_json::Value,
        expected_value: serde_json::Value,
        operator: &AssertExpectedValueObjectOperator,
    ) -> AssertResultOutcome {
        let res_as_number = match &actual_result_value {
            serde_json::Value::Number(n) => n.as_i64(),
            serde_json::Value::String(s) => match s.parse::<i64>() {
                Ok(n) => Some(n),
                Err(_) => None,
            },
            _ => None,
        };

        let expected_value_as_number = expected_value.as_i64();

        if res_as_number.is_some() && expected_value_as_number.is_some() {
            let response = res_as_number.unwrap();

            let expected_value = expected_value_as_number.unwrap();

            let result = match operator {
                AssertExpectedValueObjectOperator::LessThan(_) => response < expected_value,
                AssertExpectedValueObjectOperator::GreaterThan(_) => response > expected_value,
                _ => todo!("will never enter this path"),
            };

            if result {
                return AssertResultOutcome::Success;
            }
        }
        AssertResultOutcome::TestFailure(AssertResultOutcomeTestFailure {
            expected_value: expected_value.to_string(),
            actual_result_value: actual_result_value.to_string(),
        })
    }
}

pub struct ArrayLengthAssertion {}

impl Assertion for ArrayLengthAssertion {
    fn run(
        actual_result_value: serde_json::Value,
        expected_value: serde_json::Value,
        _operator: &AssertExpectedValueObjectOperator,
    ) -> AssertResultOutcome {
        if let Some(res_as_arr) = actual_result_value.as_array() {
            if let Ok(actual_len) = i64::try_from(res_as_arr.len()) {
                if let Some(expected_value_as_int) = expected_value.as_i64() {
                    if actual_len == expected_value_as_int {
                        return AssertResultOutcome::Success;
                    }

                    return AssertResultOutcome::TestFailure(AssertResultOutcomeTestFailure {
                        expected_value: expected_value_as_int.to_string(),
                        actual_result_value: actual_len.to_string(),
                    });
                }

                return AssertResultOutcome::FailureOnError(
                    AssertionError::ExpectedValueNotNumber {
                        expected_value,
                        actual_result_value,
                    },
                );
            }

            return AssertResultOutcome::FailureOnError(AssertionError::ExpectedValueNotNumber {
                expected_value,
                actual_result_value,
            });
        }

        AssertResultOutcome::FailureOnError(AssertionError::ResultNotAnArray {
            expected_value,
            actual_result_value,
        })
    }
}
