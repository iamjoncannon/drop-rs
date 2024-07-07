use anyhow::anyhow;
use hcl::{self, Block};
use serde::{Deserialize, Serialize};

use crate::parser::block_type::env::DropEnvironment;
use crate::parser::block_type::module::DropModule;
use crate::parser::constants::*;
use crate::parser::hcl_block::HclBlock;

use super::block_type::call::CallBlock;
use super::block_type::chain::{ChainBlock, ChainNode};
use super::block_type::run::RunBlock;
use super::types::{DropBlockType, DropResourceType};
use super::{drop_id::DropId, hcl_block::HclObject};

static NON_MODULE_BLOCK_TYPES: &str = "global mod environment";
static NO_LABEL_BLOCKS: &str = "global";
pub static CALL_BLOCKS: &str = "get post put delete run chain";

// structured data from user input hcl block
#[derive(Debug)]
pub struct DropBlock {
    pub drop_id: Option<DropId>,
    pub drop_block: DropBlockType,
    pub hcl_block: Option<hcl::Block>,
    pub file_name: String,
    pub resource_type: DropResourceType,
    pub error: Option<anyhow::Error>,
}

impl DropBlock {
    pub fn new(
        drop_id: DropId,
        drop_block: DropBlockType,
        hcl_block: Option<hcl::Block>,
        file_name: &str,
        resource_type: DropResourceType,
    ) -> DropBlock {
        DropBlock {
            drop_id: Some(drop_id),
            drop_block,
            hcl_block,
            file_name: file_name.to_string(),
            resource_type,
            error: None,
        }
    }

    pub fn from_hcl_block(
        hcl_block: Block,
        module_declaration: Option<&str>,
        file_name: &str,
    ) -> Result<DropBlock, anyhow::Error> {
        let block_type = &HclBlock::get_block_type(&hcl_block);
        let block_type = block_type.as_str();

        DropBlock::validate_drop_file(block_type, &module_declaration, file_name);

        let module_declaration = module_declaration.unwrap_or("mod");

        // get drop_id
        let labels = HclBlock::get_block_labels(&hcl_block);

        // todo- move into method
        let block_title = if labels.is_empty() {
            if !NO_LABEL_BLOCKS.contains(block_type) {
                println!("{file_name} block {block_type} must have a label");
                std::process::exit(1);
            }

            match block_type {
                GLOBAL_MOD_BLOCK_KEY => GLOBAL_MOD_BLOCK_KEY,
                _ => "",
            }
        } else {
            labels[0].as_str()
        };

        // todo- move into method
        let drop_id: String = if CALL_BLOCKS.contains(block_type) {
            format!("{module_declaration}.{block_type}.{block_title}")
        } else {
            format!("{module_declaration}.{block_title}")
        };

        assert!(
            labels.len() <= 1,
            "invalid mod block labels: {file_name} {drop_id}"
        );

        let get_drop_id = |resource_type: DropResourceType| {
            DropId::new(
                Some(module_declaration.to_string()),
                resource_type,
                None,
                block_title,
            )
        };

        let drop_resource_type = DropResourceType::from_string(block_type, file_name)?;

        match drop_resource_type {
            DropResourceType::Call => CallBlock::get_drop_block(
                hcl_block.clone(),
                DropId::get_call_drop_id(block_type, module_declaration, block_title),
                file_name,
            ),
            DropResourceType::Module => DropModule::get_drop_block(
                hcl_block,
                get_drop_id(DropResourceType::Module),
                file_name,
            ),
            DropResourceType::Environment => DropEnvironment::get_drop_block(
                hcl_block,
                get_drop_id(DropResourceType::Environment),
                file_name,
            ),
            DropResourceType::Run => {
                let drop_id_struct = DropId::new(
                    Some(module_declaration.to_string()),
                    DropResourceType::Run,
                    Some("run".to_string()),
                    block_title,
                );

                RunBlock::get_run_block(hcl_block, drop_id_struct, file_name)
            }
            DropResourceType::Chain => {
                let drop_id_struct = DropId::new(
                    Some(module_declaration.to_string()),
                    DropResourceType::Chain,
                    Some("chain".to_string()),
                    block_title,
                );

                ChainBlock::get_chain_block(hcl_block, drop_id_struct, file_name)
            }
            DropResourceType::ChainNode => {
                let drop_id_struct = DropId::new(
                    Some(module_declaration.to_string()),
                    DropResourceType::ChainNode,
                    Some("chain_node".to_string()),
                    block_title,
                );

                ChainNode::get_chain_node_block(hcl_block, drop_id_struct, file_name)
            },
        }
    }

    fn validate_drop_file(block_type: &str, module_declaration: &Option<&str>, file_name: &str) {
        let invalid_drop_file =
            !NON_MODULE_BLOCK_TYPES.contains(block_type) && module_declaration.is_none();

        if invalid_drop_file {
            let panic_msg = format!(
                "{}- please add a module declaration (e.g. {}) to file {}.",
                "Invalid drop_file", "'mod = nasa'", file_name
            );
            log::trace!("");
            println!("{panic_msg}");
            std::process::exit(1)
        }
    }
}
