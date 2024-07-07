use serde::{Deserialize, Serialize};
use super::{block_type::{call::CallBlock, chain::{ChainBlock, ChainNode}, run::RunBlock}, constants::*, hcl_block::HclObject};
use anyhow::anyhow;

// DropBlockType and DropResourceType have identical members
// 
// DropResourceType is used in certain enumeration
// contexts to support exhaustive matching without having
// to instantiate the underlying concrete structure

#[derive(Deserialize, Serialize, Debug)]
pub enum DropBlockType {
    Call(CallBlock),
    Module(Option<HclObject>),
    Environment(HclObject),
    Run(RunBlock),
    Chain(ChainBlock),
    ChainNode(ChainNode),
}


#[derive(Copy, Clone, Deserialize, Serialize, Debug, PartialEq, Eq, Hash)]
pub enum DropResourceType {
    Call,
    Module,
    Environment,
    Run,
    Chain,
    ChainNode,
}

impl DropResourceType {
    pub fn from_string(raw_block_type: &str, file_name: &str) -> Result<DropResourceType, anyhow::Error> {

        match raw_block_type {
            POST_BLOCK_KEY => Ok(DropResourceType::Call),
            GET_BLOCK_KEY => Ok(DropResourceType::Call),
            PUT_BLOCK_KEY => Ok(DropResourceType::Call),
            PATCH_BLOCK_KEY => Ok(DropResourceType::Call),
            DELETE_BLOCK_KEY => Ok(DropResourceType::Call),
            MOD_BLOCK_KEY | GLOBAL_MOD_BLOCK_KEY =>  Ok(DropResourceType::Module),
            ENVIRONMENT_BLOCK_KEY =>  Ok(DropResourceType::Environment),
            RUN_BLOCK_KEY =>  Ok(DropResourceType::Run),
            CHAIN_BLOCK_KEY => Ok(DropResourceType::Chain),
            CHAIN_NODE_KEY => Ok(DropResourceType::ChainNode),
            _ => Err(anyhow!("invalid block type: '{raw_block_type}' in {file_name}")),
        }
    }
}
