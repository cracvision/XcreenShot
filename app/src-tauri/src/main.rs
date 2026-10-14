// Release builds have no console window on Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    xcreenshot_app_lib::run()
}
