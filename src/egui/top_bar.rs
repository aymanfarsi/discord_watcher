use egui::{menu, RichText, Ui};

use super::egui_app::AppModel;

pub fn render_top_bar(app: &mut AppModel, ui: &mut Ui, frame: &mut eframe::Frame) {
    ui.add_enabled_ui(true, |ui| {
        menu::bar(ui, |ui| {
            ui.menu_button("App", |ui| {
                if ui.button("Exit").clicked() {
                    frame.close();
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
                    frame.set_always_on_top(app.is_always_on_top);
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
                    frame.set_decorations(!app.is_custom_frame);
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
