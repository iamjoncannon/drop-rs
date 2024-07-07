use hcl::Block;
use serde::{Deserialize, Serialize};

use crate::parser::{
    drop_block::{DropBlock, DropBlockType, DropResourceType},
    drop_id::DropId,
    hcl_block::HclObject,
};

#[derive(Deserialize, Serialize, Debug)]
pub struct DropEnvironment {}

impl DropEnvironment {
    pub fn get_drop_block(
        block: Block,
        drop_id: DropId,
        file_name: &str,
    ) -> Result<DropBlock, anyhow::Error> {
        let body_res: HclObject = hcl::from_body(block.body.clone())?;

        Ok(DropBlock::new(
            drop_id,
            DropBlockType::Environment(body_res),
            Some(block),
            file_name,
            DropResourceType::Environment,
        ))
    }
}
