use crate::cmd::DropCommand;


pub struct OkCommand{
    pub drop_mod: Option<String>
}


impl DropCommand for OkCommand {

    fn announce(&self){}

    fn run(&self) {

    }
}