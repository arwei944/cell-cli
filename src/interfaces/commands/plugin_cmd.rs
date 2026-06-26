use crate::application::plugin_service::PluginService;
use crate::domain::errors::CellResult;
use crate::interfaces::cli::*;
use lazy_static::lazy_static;
use std::sync::Mutex;

lazy_static! {
    static ref PLUGIN_SERVICE: Mutex<PluginService> = Mutex::new(PluginService::new());
}

pub fn cmd_plugin(args: PluginArgs) -> CellResult<()> {
    match args.sub {
        PluginSub::List {} => {
            let service = PLUGIN_SERVICE.lock().unwrap();
            let plugins = service.list_plugins();
            println!("{}", service.format_plugin_list(&plugins));
        }
        PluginSub::Load { path } => {
            let mut service = PLUGIN_SERVICE.lock().unwrap();
            let result = service.load_plugin(&path)?;
            println!("✅ 插件加载成功\n");
            println!("{}", service.format_plugin_status(&result));
        }
        PluginSub::Activate { id } => {
            let mut service = PLUGIN_SERVICE.lock().unwrap();
            let result = service.activate_plugin(&id)?;
            println!("✅ 插件激活成功\n");
            println!("{}", service.format_plugin_status(&result));
        }
        PluginSub::Deactivate { id } => {
            let mut service = PLUGIN_SERVICE.lock().unwrap();
            let result = service.deactivate_plugin(&id)?;
            println!("✅ 插件已停用\n");
            println!("{}", service.format_plugin_status(&result));
        }
        PluginSub::Status { id } => {
            let service = PLUGIN_SERVICE.lock().unwrap();
            let result = service.get_plugin_status(&id)?;
            println!("{}", service.format_plugin_status(&result));
        }
    }
    Ok(())
}
