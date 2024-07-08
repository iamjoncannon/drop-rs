use hcl::Block;
use serde::{Deserialize, Serialize};

use crate::parser::{
    drop_block::DropBlock,
    drop_id::DropId,
    types::{DropBlockType, DropResourceType},
};

use super::BlockParser;

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ChainBlock {
    pub nodes: Vec<hcl::Traversal>,
}

impl ChainBlock {
    pub fn get_chain_block(
        hcl_block: Block,
        drop_id: DropId,
        file_name: &str,
    ) -> Result<DropBlock, anyhow::Error> {
        let chain_block: Result<ChainBlock, hcl::Error> = hcl::from_body(hcl_block.body.clone());

        if chain_block.is_err() {
            let error_msg = chain_block.unwrap_err().to_string();

            Err(BlockParser::handle_block_parse_error(
                &error_msg, &drop_id, file_name,
            ))
        } else {
            Ok(DropBlock::new(
                drop_id,
                DropBlockType::Chain(chain_block.unwrap()),
                Some(hcl_block),
                file_name,
                DropResourceType::Chain,
            ))
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ChainNode {
    pub hit: hcl::Traversal,
    pub inputs: Option<hcl::Expression>,
    pub outputs: Option<hcl::Object<hcl::ObjectKey, hcl::Traversal>>,
    pub assert: Option<hcl::Object<hcl::ObjectKey, hcl::Expression>>,
}

impl ChainNode {
    pub fn get_chain_node_block(
        hcl_block: Block,
        drop_id: DropId,
        file_name: &str,
    ) -> Result<DropBlock, anyhow::Error> {
        let chain_node_block: Result<ChainNode, hcl::Error> =
            hcl::from_body(hcl_block.body.clone());

        if chain_node_block.is_err() {
            let error_msg = chain_node_block.unwrap_err().to_string();

            Err(BlockParser::handle_block_parse_error(
                &error_msg, &drop_id, file_name,
            ))
        } else {
            Ok(DropBlock::new(
                drop_id,
                DropBlockType::ChainNode(chain_node_block.unwrap()),
                Some(hcl_block),
                file_name,
                DropResourceType::ChainNode,
            ))
        }
    }
}
