#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

#[cfg(target_os = "windows")]
#[allow(non_snake_case)]
mod gui;

#[cfg(target_os = "windows")]
fn main() {
    if let Err(error) = gui::run() {
        gui::show_fatal_error(&error.to_string());
    }
}

#[cfg(not(target_os = "windows"))]
fn main() {
    eprintln!("Rustpad-X GUI currently runs on Windows.");
}
