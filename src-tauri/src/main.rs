// Prevents an extra console window from opening on Windows in release
// builds -- has no effect on Linux/macOS.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    stemloft_lib::run();
}
