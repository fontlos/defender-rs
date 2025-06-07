#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use defender_rs::loader;

fn main() {
    loader::run();
}
