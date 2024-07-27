use inquire::{ui::RenderConfig, InquireError, Select};
use log::trace;

use crate::parser::{drop_id::DropId, GlobalDropConfigProvider};

pub struct DropDown {}

impl DropDown {

    #[log_attributes::log(debug, "{fn} {return:?}")]
    pub fn drop_down(input_drop_id_string: &String) -> String{

        let is_valid_drop_id = DropId::is_drop_id(input_drop_id_string);

        if is_valid_drop_id {
            input_drop_id_string.to_string()
        } else {
            let drop_down_selection = DropDown::run_dropdown(input_drop_id_string);

            drop_down_selection
        }
    }


    fn run_dropdown(selected_module: &str) -> String {
        // let block_map = drop_ctx.block_map();

        let drop_ids_in_env =
            GlobalDropConfigProvider::get().get_all_resource_type_in_modules(selected_module);

        let options: Vec<&str> = drop_ids_in_env.iter().map(String::as_str).collect();

        let question = "Select drop";

        // RenderConfig

        let selector = Select::new(question, options).with_page_size(50);

        let drop_id_of_call_to_hit: Result<&str, InquireError> = selector.prompt();

        match drop_id_of_call_to_hit {
            Ok(drop_id_of_call_to_hit) => drop_id_of_call_to_hit.to_string(),
            Err(err) => {
                trace!("matcher err {}", err);
                panic!("error selecting drop")
            }
        }
    }
}
