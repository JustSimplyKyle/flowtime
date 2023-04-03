pub mod time;
pub use crate::time::Time;
pub mod timer;
pub use crate::timer::{Timer, TimerMode, TimerMsg};
use chrono::prelude::*;
use gtk::prelude::*;
use relm4::*;

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
                    set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 3,
                    #[watch]
                    set_visible: matches!(model.mode, AppMode::Statistics),

                    gtk::Box {
                        gtk::Label {
                            set_label: "Work"
                        },
                        add_css_class: "statcircle",
                        set_orientation: gtk::Orientation::Vertical,
                        set_valign: gtk::Align::Center,
                        gtk::Label {
                            #[watch]
                            set_label:    &second_to_formatted(current_stat(*CURRENT_MONTH).2),
                        },
                        gtk::Label {
                            #[watch]
                            set_label:
                                if current_stat(*CURRENT_MONTH).2 < 60 {
                                    "second"
                                } else if current_stat(*CURRENT_MONTH).2 < 3600 {
                                    "minute"
                                } else {
                                    "hour"
                                }
                        },
                    },
                    gtk::Box {
                        add_css_class: "statcircle",
                        set_orientation: gtk::Orientation::Vertical,
                        set_valign: gtk::Align::Center,
                        gtk::Label {
                            set_label: "Break"
                        },
                        gtk::Label {
                            #[watch]
                            set_label:    &second_to_formatted(current_stat(*CURRENT_MONTH).1)
                        },
                        gtk::Label {
                            #[watch]
                            set_label:
                                if current_stat(*CURRENT_MONTH).1 < 60 {
                                    "second"
                                } else if current_stat(*CURRENT_MONTH).1 < 3600 {
                                    "minute"
                                } else {
                                    "hour"
                                }
                        },
                    },
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
        relm4::set_global_css(
            r#"
            .statcircle {
              border-width: 8px;
              border-color: @accent_color;
              border-style: inset dotted;
              border-radius: 100%;
              box-shadow: 0px 1px 6px rgba(0, 0, 0, 0.07);
              padding: 100px;
            }
            "#,
        );
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

#[derive(Debug, Serialize, Deserialize)]
struct Stats {
    month_break_work: Vec<(u32, u32, u32)>,
}

impl std::default::Default for Stats {
    fn default() -> Self {
        Self {
            month_break_work: vec![(*CURRENT_MONTH, 0, 0)],
        }
    }
}

use lazy_static::lazy_static;

fn current_stat(month: u32) -> (u32, u32, u32) {
    let mut new_struct = stat!().month_break_work.clone();
    new_struct.push((*CURRENT_MONTH, 0, 0));
    match stat!()
        .month_break_work
        .iter()
        .filter(|(m, _, _)| m == &month)
        .nth(0)
    {
        Some(x) => *x,
        None => {
            confy::store(
                "flowtime",
                Some("statistics"),
                Stats {
                    month_break_work: new_struct,
                },
            )
            .unwrap();
            (*CURRENT_MONTH, 0, 0)
        }
    }
}
#[macro_export]
macro_rules! stat {
    () => {
        confy::load::<Stats>("flowtime", Some("statistics")).unwrap()
    };
}

fn second_to_formatted(t: u32) -> String {
    let (hour, minute, second) = (t / 3600, (t % 3600) / 60, t % 60);
    let mut output = String::new();

    if t < 60 {
        format!("{}", second)
    } else if t < 3600 {
        if minute < 10 {
            output.push_str(&format!("0{}:", minute));
        } else {
            output.push_str(&format!("{}:", minute));
        }
        if second < 10 {
            output.push_str(&format!("0{}", second));
        } else {
            output.push_str(&format!("{}", second));
        }
        output
    } else {
        if hour < 10 {
            output.push_str(&format!("0{}:", hour));
        } else {
            output.push_str(&format!("{}:", hour));
        }
        if minute < 10 {
            output.push_str(&format!("0{}:", minute));
        } else {
            output.push_str(&format!("{}:", minute));
        }
        if second < 10 {
            output.push_str(&format!("0{}", second));
        } else {
            output.push_str(&format!("{}", second));
        }
        output
    }
}

lazy_static! {
    static ref CFG: Config = confy::load("flowtime", Some("flowtime")).unwrap();
    static ref CURRENT_MONTH: u32 = Utc::now().month();
}

fn main() {
    let app = RelmApp::new("Flowtime");
    app.run::<MainApp>(AppMode::FlowTime);
}
