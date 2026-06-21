//! Marvis Office — Tauri desktop application entry point.

// Prevents an additional console window on Windows in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    marvis_tauri_lib::run();
}
