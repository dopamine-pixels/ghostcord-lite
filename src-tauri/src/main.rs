#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod commands;
mod config;
mod ipc;
mod settings;
mod state;
mod windows;

fn main() {
    app::run();
}
