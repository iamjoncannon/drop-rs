use hcl::Block;
use serde::{Deserialize, Serialize};

use crate::parser::{
    drop_block::DropBlock,
    drop_id::DropId,
    hcl_block::HclObject,
    types::{DropBlockType, DropResourceType},
};

use super::BlockParser;

#[derive(Deserialize, Serialize, Debug)]
pub struct DropEnvironment {}

impl DropEnvironment {
    pub fn get_drop_block(
        block: Block,
        drop_id: DropId,
        file_name: &str,
    ) -> Result<DropBlock, anyhow::Error> {
        let body_res = hcl::from_body(block.body.clone());

        if body_res.is_err() {
            let error_msg = body_res.unwrap_err().to_string();

            Err(BlockParser::handle_block_parse_error(
                &error_msg, &drop_id, file_name,
            ))
        } else {
            Ok(DropBlock::new(
                drop_id,
                DropBlockType::Environment(body_res.unwrap()),
                Some(block),
                file_name,
                DropResourceType::Environment,
            ))
        }
    }
}
