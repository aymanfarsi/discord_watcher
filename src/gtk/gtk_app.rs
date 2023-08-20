use crate::enums::ChannelMessage;
use gtk::prelude::*;
use gtk::prelude::{BoxExt, GtkWindowExt};
use relm4::factory::FactoryVecDeque;
use relm4::gtk::Settings;
use relm4::prelude::*;
use relm4::{gtk, ComponentParts, ComponentSender, RelmWidgetExt, SimpleComponent};
use tokio::sync::mpsc::Receiver;
// use serenity::futures::channel::mpsc::Receiver;

pub fn start_gtk_app(rx: Receiver<ChannelMessage>) {
    let app = RelmApp::new("com.evildave.discord_watcher");
    app.run::<App>(rx);
}

// ================================================================================================

#[derive(Debug)]
struct UserStruct {
    username: String,
    channel_name: String,
}

#[relm4::factory]
impl FactoryComponent for UserStruct {
    type Init = UserStruct;
    type Input = ();
    type Output = ();
    type CommandOutput = ();
    type ParentInput = AppMsg;
    type ParentWidget = gtk::ListBox;

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,

            #[name(label)]
            gtk::Label {
                set_label: &format!("{} ({})", self.username, self.channel_name),
                set_hexpand: true,
                set_halign: gtk::Align::Start,
                set_margin_all: 12,
            },
        }
    }

    fn pre_view() {
        let attrs = widgets.label.attributes().unwrap_or_default();
        widgets.label.set_attributes(Some(&attrs));
    }

    fn init_model(app: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self {
            username: app.username,
            channel_name: app.channel_name,
        }
    }
}

// ================================================================================================

#[derive(Debug)]
enum AppMsg {
    AddEntry(UserStruct),
    DeleteEntry(String),
}

#[derive(Debug)]
struct App {
    users: FactoryVecDeque<UserStruct>,
}

#[relm4::component]
impl SimpleComponent for App {
    type Init = Receiver<ChannelMessage>;
    type Input = AppMsg;
    type Output = ();

    view! {
        main_window = gtk::ApplicationWindow {
            set_width_request: 360,
            set_title: Some("Discord Watcher"),

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_margin_all: 12,
                set_spacing: 6,

                gtk::ScrolledWindow {
                    set_hscrollbar_policy: gtk::PolicyType::Never,
                    set_min_content_height: 360,
                    set_vexpand: true,

                    #[local_ref]
                    task_list_box -> gtk::ListBox {}
                }
            }

        }
    }

    fn update(&mut self, msg: AppMsg, _sender: ComponentSender<Self>) {
        match msg {
            AppMsg::AddEntry(new_user) => {
                self.users.guard().push_back(new_user);
            }
            AppMsg::DeleteEntry(username) => {
                let idx = self
                    .users
                    .guard()
                    .iter()
                    .position(|user| user.username == username)
                    .unwrap();
                match self.users.guard().remove(idx) {
                    Some(_) => {}
                    None => eprintln!("Index out of bounds!"),
                }
            }
        }
    }

    fn init(
        rx: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        Settings::default()
            .unwrap()
            .set_gtk_application_prefer_dark_theme(true);

        let model = App {
            users: FactoryVecDeque::new(gtk::ListBox::default(), sender.input_sender()),
        };

        let task_list_box = model.users.widget();

        let widgets = view_output!();

        let mut rx = rx;
        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                match msg {
                    ChannelMessage::UserJoinedChannel(username, channel_name) => {
                        sender.input(AppMsg::AddEntry(UserStruct {
                            username,
                            channel_name,
                        }));
                    }
                    ChannelMessage::UserLeftChannel(username) => {
                        sender.input(AppMsg::DeleteEntry(username));
                    }
                }
            }
        });

        ComponentParts { model, widgets }
    }
}
