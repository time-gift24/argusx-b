// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use argusx_common::{init_logging, Settings};

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 尝试加载配置（如果失败则使用默认配置）
    let settings = Settings::load_default().unwrap_or_else(|e| {
        eprintln!("Warning: Failed to load config: {}, using defaults", e);
        Settings::default()
    });

    // 初始化日志
    init_logging(
        &settings.logging.level,
        settings.logging.file.as_deref(),
        settings.logging.console,
    );

    tracing::info!("Starting tauri application");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
