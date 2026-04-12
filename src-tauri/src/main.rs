// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::env::consts::{ARCH, OS};
use std::process::Command;
use tauri::{path::BaseDirectory, Manager};

#[tauri::command]
fn decode(app: tauri::AppHandle, name: &str, private_key: &str, dist: &str) -> i32 {
    let exe_name = if OS == "windows" {
        "decode_log_file.exe"
    } else {
        "decode_log_file"
    };
    let rel = format!("binary/{}/{}/{}", OS, ARCH, exe_name);
    let path = app
        .path()
        .resolve(&rel, BaseDirectory::Resource)
        .expect("failed to resolve resource dir");
    let mut command = Command::new(path);
    command.args([private_key, name]);
    if !dist.is_empty() {
        command.arg(dist);
    }
    command.status().unwrap().code().unwrap()
}

#[tauri::command]
fn show_in_folder(path: String, opening: bool) {
    #[cfg(target_os = "windows")]
    {
        if opening {
            Command::new("explorer.exe").arg(&path).spawn().unwrap();
        } else {
            Command::new("explorer.exe")
                .args(["/select,", &path]) // The comma after select is not a typo
                .spawn()
                .unwrap();
        }
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("nautilus").arg(&path).spawn().unwrap();
    }

    #[cfg(target_os = "macos")]
    {
        if opening {
            Command::new("open").arg(&path).spawn().unwrap();
        } else {
            Command::new("open").args(["-R", &path]).spawn().unwrap();
        }
    }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![decode, show_in_folder])
        .run(tauri::generate_context!())
        .expect("error while running xlog-decoder");
}
