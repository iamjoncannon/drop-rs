use cli_table::{print_stdout, Cell, CellStruct, Table};
use colored::Colorize;
use serde_json::Value;
use thiserror::Error;

#[derive(Debug)]
pub enum AssertExpectedValueObjectOperator {
    Exist,
    NotExist,
    Contains(serde_json::Value),
    Length(serde_json::Value),
    StartsWith(serde_json::Value),
    LessThan(serde_json::Value),
    GreaterThan(serde_json::Value),
}

impl AssertExpectedValueObjectOperator {
    pub fn new(
        assertion_type_from_traversal: &str,
        expected_value: Option<serde_json::Value>,
    ) -> AssertExpectedValueObjectOperator {
        match assertion_type_from_traversal {
            "contains" => AssertExpectedValueObjectOperator::Contains(expected_value.unwrap()),
            "length" => AssertExpectedValueObjectOperator::Length(expected_value.unwrap()),
            "exist" => AssertExpectedValueObjectOperator::Exist,
            "not_exist" => AssertExpectedValueObjectOperator::NotExist,
            "starts_with" => AssertExpectedValueObjectOperator::StartsWith(expected_value.unwrap()),
            "less_than" => AssertExpectedValueObjectOperator::LessThan(expected_value.unwrap()),
            "greater_than" => AssertExpectedValueObjectOperator::GreaterThan(expected_value.unwrap()),
            _ => panic!("invalid assertion type- {assertion_type_from_traversal}- valid are 'contains', 'length', 'exist', 'not exist', 'less_than', 'greater_than'"),
        }
    }

    pub fn to_string<'a>(&self) -> &'a str {
        match self {
            AssertExpectedValueObjectOperator::Contains(_) => "contains",
            AssertExpectedValueObjectOperator::Length(_) => "array length",
            AssertExpectedValueObjectOperator::Exist => "exist",
            AssertExpectedValueObjectOperator::NotExist => "not exist",
            AssertExpectedValueObjectOperator::StartsWith(_) => "starts with",
            AssertExpectedValueObjectOperator::LessThan(_) => "less than",
            AssertExpectedValueObjectOperator::GreaterThan(_) => "greater than",
        }
    }
}

#[derive(Debug)]
pub enum AssertExpectedValue {
    Value(serde_json::Value),
    AssertValueObject(AssertExpectedValueObjectOperator),
}

impl AssertExpectedValue {
    pub fn to_string<'a>(&self) -> &'a str {
        match self {
            AssertExpectedValue::Value(_) => "equals",
            AssertExpectedValue::AssertValueObject(operator) => operator.to_string(),
        }
    }

    pub fn from_traversal(
        trav: &hcl::Traversal,
        err_prefix: &str,
        expected_value: Option<serde_json::Value>,
    ) -> AssertExpectedValue {
        let as_str = &trav.expr.to_string();

        assert!(as_str == "assert", "{err_prefix}-- {as_str:?}-- an assertion must begin with 'assert', e.g. 'assert.contains'");

        let operators = &trav.operators;

        assert!(operators.len() == 1, "{err_prefix}-- {as_str:?}-- an assertion can be one of 'assert.contain', 'assert.length', etc-- see docs");

        if let hcl::expr::TraversalOperator::GetAttr(assertion_type) = &operators[0] {
            return AssertExpectedValue::AssertValueObject(AssertExpectedValueObjectOperator::new(
                assertion_type,
                expected_value,
            ));
        }

        panic!("{err_prefix}: {trav:?} invalid")
    }
}

#[derive(Debug)]
pub struct Assert {
    pub traversal_key: hcl::Traversal,
    pub display_name: String,
    pub expected_value: AssertExpectedValue,
}

#[derive(Debug)]
pub struct AssertResultOutcomeTestFailure {
    pub expected_value: String,
    pub actual_result_value: String,
}

impl AssertResultOutcomeTestFailure {
    pub fn report(&self) -> String {
        format!(
            "{} expected: {} actual: {}",
            "FAILURE".red(),
            self.expected_value,
            self.actual_result_value
        )
    }
}

#[derive(Error, Debug)]
pub enum AssertionError {
    #[error("ExpectedValueNotNumber length assertion failure-- expected value not a number.")]
    ExpectedValueNotNumber {
        expected_value: Value,
        actual_result_value: Value,
    },

    #[error("ResultNotAnArray length assertion failure-- result not an array.")]
    ResultNotAnArray {
        expected_value: Value,
        actual_result_value: Value,
    },

    #[error("AssertionError::WalkError {msg}")]
    WalkError { msg: String },

    #[error("DeserializationError {msg}")]
    DeserializationError { msg: String },

    #[error("InvalidOutputError {msg}")]
    InvalidOutputError { msg: String },
}

#[derive(Debug)]
pub enum AssertResultOutcome {
    Success,
    TestFailure(AssertResultOutcomeTestFailure),
    FailureOnError(AssertionError),
}

impl AssertResultOutcome {
    pub fn report(&self) -> String {
        match self {
            AssertResultOutcome::Success => format!("{}", "Success".green()),
            AssertResultOutcome::TestFailure(failure) => failure.report(),
            AssertResultOutcome::FailureOnError(err) => err.to_string(),
        }
    }
}

#[derive(Debug)]
pub struct AssertResult {
    pub assert: Assert,
    pub outcome: AssertResultOutcome,
}

impl AssertResult {
    pub fn report_results(assert_results: Vec<AssertResult>) {
        let mut table = Vec::<Vec<CellStruct>>::new();

        for each in assert_results {
            let display_name = &each.assert.display_name;
            let display_name = display_name.cell();
            let operation = each.assert.expected_value.to_string();
            let result_printout = each.outcome.report().cell();
            let row = vec![display_name, operation.cell(), result_printout];
            table.push(row);
        }

        let table = table.table();

        assert!(print_stdout(table).is_ok());
    }
}
