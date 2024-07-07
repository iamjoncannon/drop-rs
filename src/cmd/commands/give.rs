use crate::{
    call::DropCall,
    cmd::{commands::hit::HitCommand, ctx::CmdContext, dropdown::DropDown, DropCommand},
    interpreter::evaluate::Evaluator,
    parser::{
        drop_block::DropBlock,
        drop_id::{CallType, DropId},
        hcl_block::HclBlock,
    },
    runner::{drop_run::DropRun, run_pool::RunPool},
};
use colored::Colorize;
use futures::Future;
use hcl::eval::{Context, Evaluate};
use indexmap::IndexMap;
use std::pin::Pin;

#[derive(Debug)]
pub struct GiveCommand {
    pub input_drop_id_string: String,
}

impl DropCommand for GiveCommand {
    fn announce(&self) {
        println!(
            "giving {} in environment {}\n",
            self.input_drop_id_string.yellow(),
            CmdContext::get_env().yellow()
        );
    }

    #[log_attributes::log(debug, "{fn} {self:?}")]
    fn run(&mut self) -> Pin<Box<dyn Future<Output = ()>>> {
        let drop_id = DropDown::drop_down(&self.input_drop_id_string);

        let call_type = DropId::get_call_type_from_raw_drop_id(&drop_id);

        // initialize call- generate single DropRun
        // and run pool

        let hit_command = HitCommand {
            input_drop_id_string: drop_id.to_string(),
        };

        let mut drop_run = match call_type {
            CallType::Hit => hit_command.run_call(),
            CallType::Run => hit_command.run_run(),
            CallType::Chain => todo!(),
        };

        // hit and run return 1 drop_run
        if drop_run.len() == 1 {
            let evaluated_block = drop_run[0].evaluate_call_block_with_blank_inputs();

            let serialized_body_res = hcl::to_string(&evaluated_block);

            match serialized_body_res {
                Ok(body) => {
                    println!("{}", body.yellow());
                }
                Err(err) => {
                    panic!("error printing hcl for {} Err-- {:?}", drop_id, err)
                }
            }
        }

        Box::pin(async {})
    }
}
