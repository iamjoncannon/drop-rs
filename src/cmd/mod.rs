use cli::Command;


pub mod cli;
pub mod ctx;

pub struct CommandManager {}

impl CommandManager {
    pub fn get_command(command: &Command) -> Box<dyn DropCommand> {

        match command {
            Command::ok { drop_mod } => Box::new(OkCommand{ drop_mod: drop_mod.clone() }),
            Command::hit { drop_id } => todo!(),
            Command::give { drop_id } => todo!(),
            Command::secret { action, key, value } => todo!(),
            Command::list { resource_type } => todo!(),
        }
    }
}

pub trait DropCommand {
    fn run(self);
}

pub struct OkCommand{
    drop_mod: Option<String>
}


impl DropCommand for OkCommand {
    fn run(self) {

    }
}

pub struct HitCommand{}


impl DropCommand for HitCommand {
    fn run(self) {

    }
}