use crate::{
    call::DropCall,
    cmd::{ctx::CmdContext, DropCommand},
    interpreter::evaluate::Evaluator, parser::drop_id::DropId,
};
use colored::Colorize;

pub struct HitCommand {
    pub input_drop_id_string: String,
}

impl DropCommand for HitCommand {
    fn announce(&self) {
        log::info!(
            "hitting {} in environment {}\n",
            self.input_drop_id_string.yellow(),
            CmdContext::get_env().yellow()
        );
    }

    fn run(&self) {

        // initialize call- generate single DropRun
        // and run pool
        
        

        

        // RunPool::runner_pool().await;
    }
}
