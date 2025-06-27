#![cfg_attr(true, windows_subsystem = "windows")]

use defender_core::loader;

fn main() {
    loader::run();
}
