use std::sync::OnceLock;
use anyhow::anyhow;

pub mod cli;

static GLOBAL_CMD_CTX_PROVIDER: OnceLock<CmdContext> = OnceLock::new();

#[derive(Debug)]
pub struct CmdContext{
    pub input_drop_id: &'static str,
    // pub drop_id: DropId
    // pub dropfile_dir: String
    // Cli
    // Commands
}

impl CmdContext {

    pub fn set(cmd: CmdContext){

        let cell_set_result = GLOBAL_CMD_CTX_PROVIDER.set(cmd);

        if cell_set_result.is_err() {
            log::error!(
                "Error setting global cmd context provider: {:?}",
                cell_set_result.unwrap_err()
            )
        }
    }

    pub fn get() -> Result<&'static CmdContext, anyhow::Error> {
        let res = GLOBAL_CMD_CTX_PROVIDER.get();

        if let Some(ctx) = res {
            Ok(ctx)
        } else {
            Err(anyhow!("error accessing global cmd ctx"))
        }
    }

    pub fn get_target_id() -> Result<&'static str, anyhow::Error> {
        let cmd = CmdContext::get()?;
        Ok(cmd.input_drop_id)
    }
}
