//! The desktop entry point; Android enters through the library.

// No console window beside the app in a Windows release build.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    dpg_app_lib::run()
}
