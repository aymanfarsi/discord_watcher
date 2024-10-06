use std::sync::atomic::Ordering;

use egui::{menu, RichText, Ui, ViewportCommand, WindowLevel};

use super::app::AppModel;

pub fn render_top_bar(app: &mut AppModel, ui: &mut Ui) {
    ui.add_enabled_ui(true, |ui| {
        menu::bar(ui, |ui| {
            ui.menu_button("App", |ui| {
                if ui.button("Debug").clicked() {
                    app.show_debug_info.store(true, Ordering::Relaxed);
                    ui.close_menu();
                }
                if ui.button("Exit").clicked() {
                    ui.ctx().send_viewport_cmd(ViewportCommand::Close);
                    ui.close_menu();
                }
            });

            ui.label("|");

            ui.menu_button("Tools", |ui| {
                if ui.button("Clear").clicked() {
                    app.events.clear();
                    ui.close_menu();
                }

                ui.separator();

                let aot_text = format!(
                    "{} Always on Top",
                    if app.is_always_on_top {
                        egui_phosphor::regular::CHECK
                    } else {
                        ""
                    }
                );
                if ui.button(aot_text).clicked() {
                    app.is_always_on_top = !app.is_always_on_top;
                    ui.ctx().send_viewport_cmd(ViewportCommand::WindowLevel(
                        if app.is_always_on_top {
                            WindowLevel::AlwaysOnTop
                        } else {
                            WindowLevel::Normal
                        },
                    ));
                    ui.close_menu();
                }

                let clear_text = format!(
                    "{} Custom Frame",
                    if app.is_custom_frame {
                        egui_phosphor::regular::CHECK
                    } else {
                        ""
                    }
                );
                if ui.button(clear_text).clicked() {
                    app.is_custom_frame = !app.is_custom_frame;
                    ui.ctx()
                        .send_viewport_cmd(ViewportCommand::Decorations(!app.is_custom_frame));
                    ui.close_menu();
                }
            });

            ui.label("|");

            match app.bot_name {
                Some(ref name) => {
                    ui.label(RichText::new(format!("Bot connected ( {} )", name)).strong())
                }
                None => ui.label(RichText::new("Bot not connected").strikethrough()),
            };
        });
    });
}
