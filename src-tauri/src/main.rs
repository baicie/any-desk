// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod remote;

use futures;
use once_cell;
use parking_lot;
use remote::{ConnectionInfo, InputEvent, NetworkManager, RemoteDesktop};
use std::env;
use std::thread;
use tauri::State;

#[derive(Debug, Default)]
enum AppMode {
    #[default]
    Controller,
    Controlled,
}

// 修改 AppState 结构
#[derive(Default)]
struct AppState {
    mode: AppMode,
}

// 使用静态变量存储网络和远程桌面实例
static NETWORK: once_cell::sync::Lazy<parking_lot::Mutex<Option<NetworkManager>>> =
    once_cell::sync::Lazy::new(|| parking_lot::Mutex::new(None));
static REMOTE: once_cell::sync::Lazy<parking_lot::Mutex<Option<RemoteDesktop>>> =
    once_cell::sync::Lazy::new(|| parking_lot::Mutex::new(None));

#[tauri::command]
async fn create_connection(_state: State<'_, AppState>) -> Result<ConnectionInfo, String> {
    let mut network = NetworkManager::new().await.map_err(|e| e.to_string())?;
    let connection_info = network.create_offer().await.map_err(|e| e.to_string())?;

    *NETWORK.lock() = Some(network);

    Ok(connection_info)
}

#[tauri::command]
async fn accept_connection(
    _state: State<'_, AppState>,
    info: ConnectionInfo,
) -> Result<ConnectionInfo, String> {
    let mut network = NetworkManager::new().await.map_err(|e| e.to_string())?;
    let answer_info = network
        .accept_offer(info)
        .await
        .map_err(|e| e.to_string())?;

    *NETWORK.lock() = Some(network);

    // 初始化远程桌面
    let remote = RemoteDesktop::new().await.map_err(|e| e.to_string())?;
    *REMOTE.lock() = Some(remote);

    Ok(answer_info)
}

#[tauri::command]
fn complete_connection(
    _state: State<'_, AppState>,
    answer_info: ConnectionInfo,
) -> Result<(), String> {
    let guard = NETWORK.lock();
    let network = guard.as_ref().ok_or("No network")?;
    network
        .complete_connection(answer_info)
        .map_err(|e| e.to_string())?;
    drop(guard);

    // 使用 block_on 来执行异步初始化
    let remote = futures::executor::block_on(RemoteDesktop::new()).map_err(|e| e.to_string())?;
    *REMOTE.lock() = Some(remote);

    Ok(())
}

#[tauri::command]
async fn send_input(event: InputEvent) -> Result<(), String> {
    let event = event.clone();
    thread::spawn(move || {
        if let Some(remote) = REMOTE.lock().as_ref() {
            let _ = remote.handle_input(event);
        }
    });
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mode = if args.get(1).map(|s| s.as_str()) == Some("--controlled") {
        AppMode::Controlled
    } else {
        AppMode::Controller
    };

    tauri::Builder::default()
        .manage(AppState { mode })
        .invoke_handler(tauri::generate_handler![
            create_connection,
            accept_connection,
            complete_connection,
            send_input,
            get_app_mode,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
async fn get_app_mode(state: State<'_, AppState>) -> Result<String, String> {
    Ok(match state.mode {
        AppMode::Controller => "controller".to_string(),
        AppMode::Controlled => "controlled".to_string(),
    })
}
