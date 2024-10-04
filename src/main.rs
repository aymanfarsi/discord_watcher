use std::env;

use egui::egui_main::start_egui;

pub mod egui {
    pub mod egui_app;
    pub mod egui_main;
    pub mod top_bar;
}
mod discord;
mod enums;
mod models;
mod utils;

fn main() {
    let args = env::args().nth(1);

    match args.as_deref() {
        Some("egui") => {
            start_egui().expect("Failed to start egui");
        }
        Some("gtk") => {
            eprintln!("GTK is not supported yet. Please use `egui` as an argument.");
        }
        Some(_) | None => {
            println!("Defaulting to egui");

            start_egui().expect("Failed to start egui");
        }
    }
}
