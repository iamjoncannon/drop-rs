/// these structures are used in 
/// various places during interpretation
/// and the values themselves are used
/// by the hcl parser to perform initial
/// input validation
pub mod call;
pub mod chain;
pub mod env;
pub mod module;
pub mod run;
use anyhow::anyhow;
use colored::Colorize;

use super::drop_id::DropId;

pub struct BlockParser {}

impl BlockParser {
    pub fn handle_block_parse_error(error_msg: &str, drop_id: &DropId, file_name: &str) -> anyhow::Error {

        let full_drop_id = drop_id.drop_id().unwrap().yellow();

        anyhow!("error parsing block {full_drop_id} in file {}: {}", file_name.yellow(), error_msg.red())
    }
}
