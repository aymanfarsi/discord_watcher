use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use egui::{
    vec2, Align2, Button, Color32, FontDefinitions, FontId, Id, LayerId, Margin, RichText,
    ScrollArea, Sense, Shadow, Stroke, Ui, UiStackInfo, ViewportBuilder, ViewportCommand,
    ViewportId,
};
use egui_struct::EguiStruct;
use tokio::sync::mpsc::Receiver;

use crate::{discord::CustomVoiceState, enums::ChannelMessage};

use super::top_bar::render_top_bar;

#[derive(Debug, Clone)]
struct DebugVoiceState {
    old_state: CustomVoiceState,
    new_state: CustomVoiceState,
}

pub struct AppModel {
    pub bot_name: Option<String>,

    pub events: Vec<String>,

    pub is_always_on_top: bool,
    pub is_custom_frame: bool,

    pub show_debug_info: Arc<AtomicBool>,
    debug_events: Vec<DebugVoiceState>,

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

            show_debug_info: Arc::new(AtomicBool::new(false)),
            debug_events: vec![],

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
                }
                ChannelMessage::UserJoinedChannel(name, channel) => {
                    self.events
                        .insert(0, format!("{} joined {}", name, channel));
                }
                ChannelMessage::UserAlreadyInChannel(name, channel) => {
                    self.events
                        .insert(0, format!("{} is already in {}", name, channel));
                }
                ChannelMessage::UserLeftChannel(name, channel) => {
                    self.events.insert(0, format!("{} left {}", name, channel));
                }
                ChannelMessage::UserDeafened(name, channel) => {
                    self.events
                        .insert(0, format!("{} deafened in {}", name, channel));
                }
                ChannelMessage::UserUndeafened(name, channel) => {
                    self.events
                        .insert(0, format!("{} undeafened in {}", name, channel));
                }
                ChannelMessage::UserMuted(name, channel) => {
                    self.events
                        .insert(0, format!("{} muted in {}", name, channel));
                }
                ChannelMessage::UserUnmuted(name, channel) => {
                    self.events
                        .insert(0, format!("{} unmuted in {}", name, channel));
                }
                ChannelMessage::UserMoved(name, old_channel, new_channel) => {
                    self.events.insert(
                        0,
                        format!("{} moved from {} to {}", name, old_channel, new_channel),
                    );
                }
                ChannelMessage::Custom(event) => {
                    self.events.insert(0, event);
                }

                ChannelMessage::DebugData(old_state, new_state) => {
                    self.debug_events.push(DebugVoiceState {
                        old_state,
                        new_state,
                    });
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
                    let info = UiStackInfo::default();
                    let available_rect = ctx.available_rect();
                    let layer_id = LayerId::background();
                    let id = Id::new("central_panel");
                    let clip_rect = ctx.screen_rect();
                    let panel_ui =
                        Ui::new(ctx.clone(), layer_id, id, available_rect, clip_rect, info);
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

        // ! Debug info
        if self.show_debug_info.load(Ordering::Relaxed) {
            let show_deferred_viewport = self.show_debug_info.clone();
            let is_custom_frame = self.is_custom_frame;
            let debug_events = self.debug_events.clone();

            ctx.show_viewport_deferred(
                ViewportId::from_hash_of("debug_info_viewport"),
                ViewportBuilder::default()
                    .with_title("Debug Info")
                    .with_inner_size([200.0, 100.0]),
                move |ctx, class| {
                    assert!(
                        class == egui::ViewportClass::Deferred,
                        "This egui backend doesn't support multiple viewports"
                    );

                    egui::CentralPanel::default()
                        .frame({
                            if is_custom_frame {
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
                                let info = UiStackInfo::default();
                                let available_rect = ctx.available_rect();
                                let layer_id = LayerId::background();
                                let id = Id::new("central_panel_debug");
                                let clip_rect = ctx.screen_rect();
                                let panel_ui = Ui::new(
                                    ctx.clone(),
                                    layer_id,
                                    id,
                                    available_rect,
                                    clip_rect,
                                    info,
                                );
                                egui::Frame::central_panel(panel_ui.style())
                            }
                        })
                        .show(ctx, |ui| {
                            // ! Custom frame
                            if is_custom_frame {
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

                            // ! Title
                            ui.allocate_ui(vec2(ui.available_size_before_wrap().x, 30.), |ui| {
                                ui.centered_and_justified(|ui| {
                                    ui.label(
                                        RichText::new("Debug Info").strong().heading().size(20.),
                                    );
                                });
                            });

                            ui.separator();

                            // ! Events list
                            ScrollArea::new([false, true])
                                .auto_shrink([false; 2])
                                .drag_to_scroll(true)
                                .show(ui, |ui| {
                                    let font_size = 16.;
                                    for (idx, event) in debug_events.iter().enumerate() {
                                        event.clone().old_state.show_top(
                                            ui,
                                            RichText::new("Old State").strong().size(font_size),
                                            None,
                                        );

                                        event.clone().new_state.show_top(
                                            ui,
                                            RichText::new("New State").strong().size(font_size),
                                            None,
                                        );

                                        if idx < debug_events.len() - 1 {
                                            ui.separator();
                                        }
                                    }
                                });
                        });

                    if ctx.input(|i| i.viewport().close_requested()) {
                        show_deferred_viewport.store(false, Ordering::Relaxed);
                    }
                },
            );
        }
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

    let max_rect = title_bar_rect.shrink(4.0);
    ui.allocate_ui_at_rect(max_rect, |ui| {
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
