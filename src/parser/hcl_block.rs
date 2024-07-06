use hcl::{expr::TraversalOperator, Block, BlockLabel, Expression, Value};
use anyhow::anyhow;
pub type HclObject = hcl::Object<hcl::ObjectKey, hcl::Expression>;

/// various helper methods to operate on Hcl structures
pub struct HclBlock {}

impl HclBlock {
    pub fn get_block_type(hcl_block: &Block) -> String {
        hcl_block.identifier().to_string()
    }

    pub fn get_block_labels(hcl_block: &Block) -> Vec<BlockLabel> {
        // this clones the value, we clone here to prevent
        // having to clone the entire block later
        hcl_block.labels().to_vec()
    }

    pub fn value_from_expr(expr: Expression) -> Value {
        match expr {
            Expression::Null => Value::Null,
            Expression::Bool(b) => Value::Bool(b),
            Expression::Number(n) => Value::Number(n),
            Expression::String(s) => Value::String(s),
            Expression::Array(array) => array.into_iter().collect(),
            Expression::Object(object) => object.into_iter().collect(),
            Expression::TemplateExpr(expr) => Value::String(expr.to_string()),
            Expression::Parenthesis(expr) => Value::from(*expr),
            // Expression::Raw(raw) => Value::String(raw.into()),
            _other => Value::String("unimplemented".to_string()),
        }
    }

    pub fn format_hcl_raw_string(raw_string: String) -> String {
        if !raw_string.contains('\"') {
            return raw_string;
        }
    
        let len = raw_string.len();
    
        if len < 3 {
            return String::new();
        }
    
        let end = len - 1;
    
        raw_string[1..end].to_string()
    }

    pub fn traversal_to_string(traversal: &hcl::Traversal) -> String {
        let mut first_term = match &traversal.expr {
            Expression::Variable(exp) => vec![exp.to_string()],
            _ => panic!("invalid hit value passed"),
        };
    
        for operator in &traversal.operators {
            match operator {
                TraversalOperator::GetAttr(attr) => {
                    let as_str = attr.as_str();
                    first_term.push(as_str.to_string());
                }
                TraversalOperator::Index(expr) => match expr {
                    hcl::Expression::String(str) => {
                        first_term.push(str.to_string());
                    }
                    hcl::Expression::Number(num) => {
                        first_term.push(num.to_string());
                    }
                    _ => todo!(),
                },
                TraversalOperator::AttrSplat
                | TraversalOperator::FullSplat
                | TraversalOperator::LegacyIndex(_) => {}
            }
        }
    
        first_term.join(".")
    }

    pub fn hcl_expression_to_serde_value(
        hcl_expression: &hcl::Expression,
    ) -> Result<serde_json::Value, anyhow::Error> {
        match hcl_expression {
            Expression::Number(num) => Ok(serde_json::Value::from(num.as_i64())),
            Expression::String(as_str) => Ok(serde_json::Value::String(as_str.to_string())),
            Expression::Bool(bool) => Ok(serde_json::Value::Bool(*bool)),
            _ => Err(anyhow!(
                "Error deserializing value: {:?}",
                hcl_expression.to_string()
            )),
        }
    }
    
}
