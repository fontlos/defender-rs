// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use defender_core::loader;

fn main() {
    loader::run();
}
