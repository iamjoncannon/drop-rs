use hcl::Block;
use serde::{Deserialize, Serialize};

use crate::parser::{drop_block::DropBlock, drop_id::DropId, hcl_block::HclObject, types::{DropBlockType, DropResourceType}};

use super::BlockParser;


#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct CallBlock {
    base_url: hcl::Expression,
    path: Option<hcl::Expression>,
    headers: Option<Vec<hcl::Expression>>,
    body: Option<hcl::Expression>,
    after: Option<Vec<HclObject>>,
    outputs: Option<Vec<hcl::Traversal>>,
    inputs: Option<hcl::Expression>,
    assert: Option<hcl::Object<hcl::ObjectKey, hcl::Expression>>,
}

impl CallBlock {
    pub fn get_drop_block(
        block: Block,
        drop_id: DropId,
        file_name: &str,
    ) -> Result<DropBlock, anyhow::Error> {
        // we clone here because hcl::from_body requires an owned value, and we want to
        // pass the value to the drop block container structure
        let call_block = hcl::from_body(block.body.clone());

        if call_block.is_err() {

            let error_msg = call_block.unwrap_err().to_string();

            Err(BlockParser::handle_block_parse_error(&error_msg, &drop_id, file_name))

        } else {

        Ok(DropBlock::new(
            drop_id,
            DropBlockType::Call(call_block.unwrap()),
            Some(block),
            file_name,
            DropResourceType::Call,
        ))
    }
    }
}
