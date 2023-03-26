// use crossterm::{
//     cursor,
//     event::{self, Event},
//     execute,
//     terminal::{Clear, ClearType},
//     QueueableCommand,
// };
// use gtk::prelude::{BoxExt, ButtonExt, GtkWindowExt, OrientableExt, WidgetExt};
pub mod time;
pub use crate::time::Time;
pub mod timer;
pub use crate::timer::{Timer, TimerMode, TimerMsg};
use chrono::prelude::*;
use gtk::prelude::*;
use relm4::*;

// use relm4::{
//     gtk, Component, ComponentParts, ComponentSender, Controller, RelmApp, RelmWidgetExt,
//     SimpleComponent,
// };
// use std::io::{stdout, Write};
// use std::thread::sleep;
struct HeaderModel;
#[derive(Debug)]
enum HeaderOutput {
    FlowTime,
    Settings,
    Statistics,
}

#[relm4::component]
impl SimpleComponent for HeaderModel {
    type Input = ();
    type Init = ();
    type Output = HeaderOutput;

    view! {
        #[root]
        gtk::HeaderBar {
            #[wrap(Some)]
            set_title_widget = &gtk::Box {
                add_css_class: "linked",
                set_spacing: 10,
                #[name = "group"]
                gtk::ToggleButton {
                    set_label: "FlowTime",
                    set_active: true,
                    connect_toggled[sender] => move |btn| {
                        if btn.is_active() {
                            sender.output(HeaderOutput::FlowTime).unwrap()
                        }
                    },
                },
                gtk::ToggleButton {
                    set_label: "Settings",
                    set_group: Some(&group),
                    connect_toggled[sender] => move |btn| {
                        if btn.is_active() {
                            sender.output(HeaderOutput::Settings).unwrap()
                        }
                    },
                },
                gtk::ToggleButton {
                    set_label: "Statistics",
                    set_group: Some(&group),
                    connect_toggled[sender] => move |btn| {
                        if btn.is_active() {
                            sender.output(HeaderOutput::Statistics).unwrap()
                        }
                    },
                },
            }
        }
    }
    fn init(
        _params: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = HeaderModel;
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}

#[derive(Debug, Clone)]
struct SettingsModel;

#[derive(Debug)]
enum SettingsMsg {
    AutoRestart(bool),
}

#[relm4::component]
impl SimpleComponent for SettingsModel {
    type Input = ();
    type Init = ();
    type Output = SettingsMsg;

    view! {
        gtk::Box {
            set_spacing: 10,
            gtk::Switch {
                set_active: CFG.restart,
                connect_state_notify => move |switch| {
                confy::store("flowtime", Some("flowtime"), Config { restart: switch.is_active() }).unwrap();
                    sender.output(SettingsMsg::AutoRestart(switch.is_active())).unwrap();
                },
            },
            gtk::Label {
                set_label: "Auto start Flowtime session after the break has ended."
            }
        }
    }
    fn init(
        _params: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = SettingsModel;
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }
}
#[derive(Debug, Clone)]
struct StatsticsModel;

#[relm4::component]
impl SimpleComponent for StatsticsModel {
    type Input = ();
    type Init = ();
    type Output = ();

    view! {
        gtk::Box {
            set_spacing: 10,
        }
    }
    fn init(
        _params: Self::Init,
        root: &Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = StatsticsModel;
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }
}

#[derive(Debug, PartialEq)]
enum AppMode {
    FlowTime,
    Settings,
    Statistics,
}

#[derive(Debug)]
enum MainAppMsg {
    SetMode(AppMode),
    SetRestart(bool),
}

struct MainApp {
    mode: AppMode,
    header: Controller<HeaderModel>,
    main: Controller<Timer>,
    setting: Controller<SettingsModel>,
}

#[relm4::component]
impl SimpleComponent for MainApp {
    type Init = AppMode;
    type Input = MainAppMsg;
    type Output = ();
    view! {
        gtk::ApplicationWindow {
            set_titlebar: Some(model.header.widget()),
            set_title: Some("Flowtime"),
            set_default_size: (386,311),
            gtk::Box {
                set_valign: gtk::Align::Center,
                set_halign: gtk::Align::Center,
                set_spacing: 10,
                gtk::Box {
                    #[watch]
                    set_visible: matches!(model.mode, AppMode::FlowTime),
                    model.main.widget(),
                },
                gtk::Box {
                    #[watch]
                    set_visible: matches!(model.mode, AppMode::Settings),
                    model.setting.widget(),
                },
                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    #[watch]
                    set_visible: matches!(model.mode, AppMode::Statistics),
                    gtk::Label {
                        #[watch]
                        set_label:    &format!("{:?}",second_to_formatted(stats().work_second)),
                    },
                    gtk::Label {
                        #[watch]
                        set_label:    &format!("{:?}",&second_to_formatted(stats().break_second)),
                    }
                }
            }
        }
    }
    fn init(
        params: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = MainApp {
            mode: params,
            header: HeaderModel::builder().launch(()).forward(
                sender.input_sender(),
                |msg| match msg {
                    HeaderOutput::FlowTime => MainAppMsg::SetMode(AppMode::FlowTime),
                    HeaderOutput::Settings => MainAppMsg::SetMode(AppMode::Settings),
                    HeaderOutput::Statistics => MainAppMsg::SetMode(AppMode::Statistics),
                },
            ),
            main: Timer::builder().launch(TimerMode::Stop).detach(),
            setting: SettingsModel::builder()
                .launch(())
                .forward(sender.input_sender(), |msg| match msg {
                    SettingsMsg::AutoRestart(x) => MainAppMsg::SetRestart(x),
                }),
        };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }
    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            MainAppMsg::SetMode(mode) => {
                self.mode = mode;
            }
            MainAppMsg::SetRestart(x) => self.main.sender().send(TimerMsg::SetRestart(x)).unwrap(),
        }
    }
}

use serde_derive::{Deserialize, Serialize};
#[derive(Default, Serialize, Deserialize)]
struct Config {
    restart: bool,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct Stats {
    month: u32,
    break_second: u32,
    work_second: u32,
}

use lazy_static::lazy_static;
fn stats() -> Stats {
    confy::load("flowtime", Some("statistics")).unwrap()
}
fn second_to_formatted(t: u32) -> (u32, u32, u32) {
    (t / 3600, (t % 3600) / 60, t % 60)
}

lazy_static! {
    static ref CFG: Config = confy::load("flowtime", Some("flowtime")).unwrap();
    static ref CURRENT_MONTH: u32 = Utc::now().month();
}

fn main() {
    let app = RelmApp::new("Flowtime");
    app.run::<MainApp>(AppMode::FlowTime);
}
