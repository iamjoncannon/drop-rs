use anyhow::anyhow;
use hcl::{self, Block};
use serde::{Deserialize, Serialize};

use crate::parser::block_type::env::DropEnvironment;
use crate::parser::block_type::module::DropModule;
use crate::parser::constants::*;
use crate::parser::{ hcl_block::HclBlock};

use super::block_type::call::CallBlock;
use super::block_type::run::RunBlock;
use super::{ drop_id::DropId, hcl_block::HclObject};

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

#[derive(Deserialize, Serialize, Debug)]
pub enum DropBlockType {
    Call(CallBlock),
    Module(Option<HclObject>),
    Environment(HclObject), 
    Run(RunBlock),
                            // Chain(ChainBlock),
                            // Object(ObjectBlock),
}

#[derive(Copy, Clone, Deserialize, Serialize, Debug, PartialEq, Eq, Hash)]
pub enum DropResourceType {
    Call,
    Module,
    Environment,
    Run,
    // Input,
    // Chain,
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

        let invalid_drop_file =
            !NON_MODULE_BLOCK_TYPES.contains(block_type) && module_declaration.is_none();

        if invalid_drop_file {
            let panic_msg = format!(
                "{}- please add a module declaration (e.g. {}) to file {}.",
                "Invalid drop_file", "'mod = nasa'", file_name
            );
            panic!("{panic_msg}")
        }

        let module_declaration = module_declaration.unwrap_or("mod");

        // get drop_id
        let labels = HclBlock::get_block_labels(&hcl_block);

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

        let call_block = |method: &str| {
            CallBlock::get_drop_block(
                hcl_block.clone(),
                DropId::get_call_drop_id(method, module_declaration, block_title),
                file_name,
            )
        };

        match block_type {
            POST_BLOCK_KEY => call_block("post"),
            GET_BLOCK_KEY => call_block("get"),
            PUT_BLOCK_KEY => call_block("patch"),
            PATCH_BLOCK_KEY => call_block("put"),
            DELETE_BLOCK_KEY => call_block("delete"),

            MOD_BLOCK_KEY | GLOBAL_MOD_BLOCK_KEY => DropModule::get_drop_block(
                hcl_block,
                get_drop_id(DropResourceType::Module),
                file_name,
            ),

            ENVIRONMENT_BLOCK_KEY => DropEnvironment::get_drop_block(
                hcl_block,
                get_drop_id(DropResourceType::Environment),
                file_name,
            ),

            RUN_BLOCK_KEY => {
                let drop_id_struct = DropId::new(
                    Some(module_declaration.to_string()),
                    DropResourceType::Run,
                    Some("run".to_string()),
                    block_title,
                );

                RunBlock::get_run_block(hcl_block, drop_id_struct, file_name)
            }
            // CHAIN_BLOCK_KEY => {
            //     let drop_id_struct = DropId::new(
            //         Some(module_declaration.to_string()),
            //         DropResourceType::Chain,
            //         Some("chain".to_string()),
            //         block_title,
            //     );

            //     get_chain_block(hcl_block, drop_id_struct, file_name)
            // }
            _ => Err(anyhow!("invalid block type: {block_type} in {file_name}")),
        }
    }
}
