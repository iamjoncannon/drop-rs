use hcl::Block;
use hcl::Body;

use crate::parser::drop_block::DropBlock;
use crate::parser::drop_block::DropBlockType;
use crate::parser::drop_block::DropResourceType;
use crate::parser::drop_id::DropId;
use crate::parser::hcl_block::HclObject;


pub struct DropModule {}

impl DropModule {
    pub fn get_drop_block(
        block: Block,
        drop_id: DropId,
        file_name: &str,
    ) -> Result<DropBlock, anyhow::Error> {
        let body_res: HclObject = hcl::from_body(block.body.clone())?;

        Ok(DropBlock::new(
            drop_id,
            DropBlockType::Module(Some(body_res)),
            Some(block),
            file_name,
            DropResourceType::Module,
        ))
    }

    pub fn get_module_declaration(file_body: &Body, _file_name: &str) -> Option<String> {
        let first_attr = file_body.attributes().next();

        if let Some(attr) = first_attr {
            let key = attr.key();

            // issue- invalid file starts with a
            // attribute declaraiton thats not 'mod'
            if key != "mod" {
                // return error
                // trace!("{file_name} invalid module declaration {key}");
                // panic!()
                return None;
            }

            // there is variable attribute declaration, aka "mod = "
            if let hcl::Expression::Variable(var) = attr.expr() {
                let as_str = var.as_str();
                Some(as_str.to_owned())
            } else {
                // some other invalid mod

                // return error
                // trace!("{file_name} no module declaration");
                // panic!()
                None
            }
        } else {
            // return error
            // trace!("{file_name} no module declaration");
            None
            // panic!()
        }
    }
}
