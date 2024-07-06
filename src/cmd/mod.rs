use std::pin::Pin;

use cli::Command;
use commands::{hit::HitCommand, ok::OkCommand};
use futures::{future::BoxFuture, Future};


pub mod cli;
pub mod ctx;
pub mod commands;

pub trait DropCommand {
    fn announce(&self);
    fn run(&self) -> Pin<Box<dyn Future<Output = ()>>>;
}

/// converts clap cli command into
/// drop command structure
pub struct CommandManager {}

impl CommandManager {
    pub fn get_command(command: &Command) -> Box<dyn DropCommand> {

        log::debug!("command {:?}", command);

        match command {
            Command::hit { drop_id } => Box::new(HitCommand{ input_drop_id_string: drop_id.to_string() }),
            Command::give { drop_id } => todo!(),

            Command::ok { drop_mod } => Box::new(OkCommand{ drop_mod: drop_mod.clone() }),
            Command::secret { action, key, value } => todo!(),
            Command::list { resource_type } => todo!(),
        }
    }
}

