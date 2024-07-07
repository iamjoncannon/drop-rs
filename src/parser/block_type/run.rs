use serde::{Deserialize, Serialize};

use crate::parser::{drop_block::DropBlock, drop_id::DropId, types::{DropBlockType, DropResourceType}};


#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct RunBlock {
    pub hit: hcl::Traversal,
    pub inputs: hcl::Expression,
    pub outputs: Option<Vec<hcl::Traversal>>,
    pub assert: Option<hcl::Object<hcl::ObjectKey, hcl::Expression>>,
}

impl RunBlock {

    pub fn get_run_block(hcl_block: hcl::Block, drop_id: DropId, file_name: &str) -> Result<DropBlock, anyhow::Error> {
        let run_block = hcl::from_body(hcl_block.body.clone())?;
    
        Ok(DropBlock::new(
            drop_id,
            DropBlockType::Run(run_block),
            Some(hcl_block),
            file_name,
            DropResourceType::Run)
        )
    }

    pub fn get_drop_id_of_hit(&self) -> String {
        let hit_traversal = &self.hit;

        let mut drop_id = match &hit_traversal.expr {
            hcl::Expression::Variable(exp) => vec![exp.to_string()],
            _ => panic!("invalid hit value passed"),
        };

        for operator in &hit_traversal.operators {
            if let hcl::TraversalOperator::GetAttr(attr) = operator {
                let as_str = attr.as_str();
                drop_id.push(as_str.to_string());
            }
        }

        drop_id.join(".")
    }
}