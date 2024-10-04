use egui::{
    vec2, Align2, Button, Color32, FontDefinitions, FontId, Id, LayerId, Margin, RichText,
    ScrollArea, Sense, Shadow, Stroke, Ui, UiBuilder, ViewportCommand,
};
use tokio::sync::mpsc::Receiver;

use crate::enums::ChannelMessage;

use super::top_bar::render_top_bar;

pub struct AppModel {
    pub bot_name: Option<String>,

    pub events: Vec<String>,

    pub is_always_on_top: bool,
    pub is_custom_frame: bool,

    rx: Receiver<ChannelMessage>,
}

impl AppModel {
    pub fn new(cc: &eframe::CreationContext<'_>, rx: Receiver<ChannelMessage>) -> Self {
        let mut fonts = FontDefinitions::default();
        egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
        cc.egui_ctx.set_fonts(fonts);

        AppModel {
            bot_name: None,
            events: vec![],

            is_always_on_top: false,
            is_custom_frame: false,

            rx,
        }
    }
}

impl eframe::App for AppModel {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Ok(rx) = self.rx.try_recv() {
            match rx {
                ChannelMessage::BotConnected(ready) => {
                    self.bot_name = Some(ready.user.name);
                    ctx.request_repaint();
                }
                ChannelMessage::UserJoinedChannel(name, channel) => {
                    self.events.push(format!("{} joined {}", name, channel));
                    ctx.request_repaint();
                }
                ChannelMessage::UserAlreadyInChannel(name, channel) => {
                    self.events
                        .push(format!("{} is already in {}", name, channel));
                    ctx.request_repaint();
                }
                ChannelMessage::UserLeftChannel(name) => {
                    self.events.push(format!("{} left a channel", name));
                    ctx.request_repaint();
                }
            }

            ctx.request_repaint();
        }

        // ! Render events
        egui::CentralPanel::default()
            .frame({
                if self.is_custom_frame {
                    egui::Frame {
                        fill: ctx.style().visuals.window_fill(),
                        rounding: 10.0.into(),
                        stroke: Stroke {
                            width: 0.5,
                            color: Color32::LIGHT_GRAY,
                        },
                        outer_margin: 0.0.into(),
                        inner_margin: Margin::same(8.0),
                        shadow: Shadow::default(),
                    }
                } else {
                    let available_rect = ctx.available_rect();
                    let layer_id = LayerId::background();
                    let id = Id::new("central_panel");
                    let ui_builder = UiBuilder::new().max_rect(available_rect);
                    let panel_ui = Ui::new(ctx.clone(), layer_id, id, ui_builder);
                    egui::Frame::central_panel(panel_ui.style())
                }
            })
            .show(ctx, |ui| {
                // ! Custom frame
                if self.is_custom_frame {
                    let app_rect = ui.max_rect();

                    let title_bar_height = 32.0;
                    let title_bar_rect = {
                        let mut rect = app_rect;
                        rect.max.y = rect.min.y + title_bar_height;
                        rect
                    };
                    title_bar_ui(ui, title_bar_rect, "Discord Watcher");
                    ui.add_space(1.5);
                    ui.separator();
                    ui.add_space(3.);
                }

                // ! Render top bar
                render_top_bar(self, ui);

                // ! Title
                ui.allocate_ui(vec2(ui.available_size_before_wrap().x, 30.), |ui| {
                    ui.centered_and_justified(|ui| {
                        ui.label(RichText::new("Discord Events").strong().heading().size(20.));
                    });
                });

                ui.separator();

                // ! Events list
                ScrollArea::new([false, true])
                    .auto_shrink([false; 2])
                    .drag_to_scroll(true)
                    .show(ui, |ui| {
                        let font_size = 16.;
                        for event in self.events.iter() {
                            let text = if event.contains("joined") {
                                RichText::new(event).strong()
                            } else if event.contains("left") {
                                RichText::new(event).strikethrough()
                            } else {
                                RichText::new(event).small()
                            };
                            ui.allocate_ui(vec2(ui.available_size_before_wrap().x, 15.), |ui| {
                                ui.label(text.size(font_size));
                            });
                        }
                    });
            });
    }
}

fn title_bar_ui(ui: &mut Ui, title_bar_rect: egui::Rect, title: &str) {
    let painter = ui.painter();

    let title_bar_response = ui.interact(title_bar_rect, Id::new("title_bar"), Sense::click());

    // Paint the title:
    painter.text(
        title_bar_rect.center(),
        Align2::CENTER_CENTER,
        title,
        FontId::proportional(20.0),
        ui.style().visuals.text_color(),
    );

    // Paint the line under the title:
    // painter.line_segment(
    //     [
    //         title_bar_rect.left_bottom() + vec2(1.0, 0.0),
    //         title_bar_rect.right_bottom() + vec2(-1.0, 0.0),
    //     ],
    //     ui.visuals().widgets.noninteractive.bg_stroke,
    // );

    // Interact with the title bar (drag to move window):
    if title_bar_response.double_clicked() {
        let is_maximized = ui.input(|i| i.viewport().maximized).unwrap_or(false);
        ui.ctx()
            .send_viewport_cmd(ViewportCommand::Maximized(!is_maximized));
    } else if title_bar_response.is_pointer_button_down_on() {
        ui.ctx().send_viewport_cmd(ViewportCommand::StartDrag);
    }

    let ui_builder = UiBuilder::new().max_rect(title_bar_rect);
    ui.allocate_new_ui(ui_builder, |ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            ui.visuals_mut().button_frame = false;
            ui.add_space(8.0);
            close_maximize_minimize(ui);
        });
    });
}

fn close_maximize_minimize(ui: &mut egui::Ui) {
    let button_height = 12.0;

    let close_response = ui
        .add(Button::new(
            RichText::new(egui_phosphor::regular::X).size(button_height),
        ))
        .on_hover_text("Close the window");
    if close_response.clicked() {
        ui.ctx().send_viewport_cmd(ViewportCommand::Close);
    }

    let is_maximized = ui.input(|i| i.viewport().maximized).unwrap_or(false);
    if is_maximized {
        let maximized_response = ui
            .add(Button::new(RichText::new("🗗").size(button_height)))
            .on_hover_text("Restore window");
        if maximized_response.clicked() {
            ui.ctx()
                .send_viewport_cmd(ViewportCommand::Maximized(false));
        }
    } else {
        let maximized_response = ui
            .add(Button::new(
                RichText::new(egui_phosphor::regular::CORNERS_OUT).size(button_height),
            ))
            .on_hover_text("Maximize window");
        if maximized_response.clicked() {
            ui.ctx().send_viewport_cmd(ViewportCommand::Maximized(true));
        }
    }

    let minimized_response = ui
        .add(Button::new(RichText::new("🗕").size(button_height)))
        .on_hover_text("Minimize the window");
    if minimized_response.clicked() {
        ui.ctx().send_viewport_cmd(ViewportCommand::Minimized(true));
    }
}
