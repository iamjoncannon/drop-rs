use std::{collections::HashSet, fs, path::PathBuf, sync::OnceLock};

use drop_block::{DropBlock, DropBlockType, DropResourceType};
use drop_id::DropId;
use drop_module::DropModule;
use hcl::Body;
use log_derive::logfn;

pub mod call;
pub mod constants;
pub mod drop_block;
pub mod drop_env;
pub mod drop_id;
pub mod drop_module;
pub mod file_walker;
pub mod hcl_block;

static GLOBAL_DROP_CONFIG_PROVIDER: OnceLock<GlobalDropConfig> = OnceLock::new();

pub struct GlobalDropConfigProvider{}

impl GlobalDropConfigProvider{
    pub fn set(global_drop_config: GlobalDropConfig){
        let cell_set_result = GLOBAL_DROP_CONFIG_PROVIDER.set(global_drop_config);
        if cell_set_result.is_err() {
            log::error!(
                "Error setting global config provider: {:?}",
                cell_set_result.unwrap_err()
            )
        }
    }

    pub fn get() -> &'static GlobalDropConfig {
        GLOBAL_DROP_CONFIG_PROVIDER.get().unwrap()
    }
}

/// parsed hcl blocks prior
/// to evaluation by interpreter
#[derive(Debug)]
pub struct GlobalDropConfig {
    pub calls: Vec<DropBlock>,
    pub modules: Vec<DropBlock>,
    pub environments: Vec<DropBlock>,
}

impl GlobalDropConfig {
    fn new() -> GlobalDropConfig {
        GlobalDropConfig {
            calls: Vec::new(),
            modules: Vec::new(),
            environments: Vec::new(),
        }
    }

    #[log_attributes::log(debug, "GlobalDropConfig {fn}")]
    #[log_attributes::log(trace, "GlobalDropConfig {fn} output: {return:?}")]
    pub fn from_drop_files(drop_files: &Vec<PathBuf>) -> Result<GlobalDropConfig, anyhow::Error> {
        let mut unevaluated_blocks: Vec<DropBlock> = Vec::new();
        let mut file_level_module_declarations: HashSet<String> = HashSet::new();

        for path_buf in drop_files {
            let file_name = path_buf.file_name().unwrap().to_str().unwrap();

            let file = fs::read_to_string(path_buf)?;
            let body = hcl::from_str(&file)?;

            // warn if error in unevaluated block

            let mut res = GlobalDropConfig::collect_unevalulated_blocks(
                body,
                file_name,
                // &mut unevaluated_blocks,
                &mut file_level_module_declarations,
            )?;

            let errors: Vec<&anyhow::Error> = res
                .iter()
                .filter(|res| res.is_err())
                .map(|each| each.as_ref().unwrap_err())
                .collect();

            if !errors.is_empty() {
                errors.iter().for_each(|err|{
                    log::error!("error evaluating block: {:?}", err);
                })
            }

            let success: Vec<DropBlock> = res
                .drain(..)
                .filter(|res| !res.is_err())
                .map(|each| each.unwrap())
                .collect();

            unevaluated_blocks.extend(success);
        }

        for b in &unevaluated_blocks {
            if b.drop_id.as_ref().unwrap().resource_type == DropResourceType::Module {
                let module_block_id = &b.drop_id.as_ref().unwrap().resource_name;
                if file_level_module_declarations.contains(module_block_id) {
                    file_level_module_declarations.remove(module_block_id);
                }
            }
        }

        for file_level_module_declarations in &file_level_module_declarations {
            let drop_id = DropId::new(
                Some("mod".to_string()),
                DropResourceType::Module,
                None,
                file_level_module_declarations,
            );

            // new module
            let container = DropBlock::new(
                drop_id,
                DropBlockType::Module(None),
                None,
                "",
                DropResourceType::Module,
            );

            unevaluated_blocks.push(container);
        }

        let mut global_drop_config = GlobalDropConfig::new();

        for container in unevaluated_blocks.drain(..) {
            let rt = container.resource_type;

            match rt {
                DropResourceType::Call => global_drop_config.calls.push(container),
                DropResourceType::Module => global_drop_config.modules.push(container),
                DropResourceType::Environment => global_drop_config.environments.push(container),
            }
        }

        Ok(global_drop_config)
    }

    ///
    /// parse blocks and perform basic validation on properties,
    /// collect module declarations
    ///
    #[log_attributes::log(trace, "{fn} file_name: {file_name} output: {return:?}")]
    pub fn collect_unevalulated_blocks(
        file_body: hcl::Body,
        file_name: &str,
        blockless_module_declarations: &mut HashSet<String>,
    ) -> Result<Vec<Result<DropBlock, anyhow::Error>>, anyhow::Error> {
        let mut module_declaration: Option<&str> = None;

        let declaration = DropModule::get_module_declaration(&file_body, file_name);

        if let Some(dec) = declaration.clone() {
            blockless_module_declarations.insert(dec);
        }

        match declaration.as_ref() {
            Some(str) => {
                module_declaration = Some(str.as_str());
            }
            None => {
                // anyhow!("no module declaration")
            }
        }

        GlobalDropConfig::convert_hcl_blocks_to_drop_blocks(
            file_body,
            module_declaration,
            file_name,
        )
    }

    #[logfn(
        ok = "TRACE",
        err = "ERROR",
        fmt = "converted hcl blocks to drop blocks: {:?}",
        log_ts = true
    )]
    fn convert_hcl_blocks_to_drop_blocks(
        file_body: Body,
        module_declaration: Option<&str>,
        file_name: &str,
    ) -> Result<Vec<Result<DropBlock, anyhow::Error>>, anyhow::Error> {
        let drop_block_conatiners: Vec<Result<DropBlock, anyhow::Error>> = file_body
            .into_blocks()
            .map(|hcl_block| DropBlock::from_hcl_block(hcl_block, module_declaration, file_name))
            .collect();

        Ok(drop_block_conatiners)
    }
}
