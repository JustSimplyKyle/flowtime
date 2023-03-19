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
use gtk::prelude::*;
use relm4::*;
use rodio::Sink;
use rodio::{Decoder, OutputStream};

use std::fs::File;
use std::io::BufReader;

// use relm4::{
//     gtk, Component, ComponentParts, ComponentSender, Controller, RelmApp, RelmWidgetExt,
//     SimpleComponent,
// };
use std::time::Duration;
// use std::io::{stdout, Write};
// use std::thread::sleep;

#[derive(PartialEq, Debug, Clone)]
enum TimerMode {
    Clock,
    CountDown,
    Stop,
    Pause(Box<TimerMode>),
}

#[derive(Debug)]
struct Timer {
    mode: TimerMode,
    time: Time,
    clicking: bool,
    restart: bool,
}
impl Timer {
    fn new() -> Timer {
        Timer {
            mode: TimerMode::Stop,
            time: Default::default(),
            clicking: false,
            restart: CFG.restart,
        }
    }
    fn tick(&mut self) -> bool {
        match self.mode {
            TimerMode::Clock => {
                self.time.increment_second();
                false
            }
            TimerMode::CountDown => {
                if self.time.second == 0 && self.time.minutes == 0 && self.time.hour == 0 {
                    if self.restart {
                        self.mode = TimerMode::Clock;
                    } else {
                        self.mode = TimerMode::Stop;
                    }
                    true
                } else {
                    self.time.decrement_second();
                    false
                }
            }
            TimerMode::Stop => false,
            TimerMode::Pause(_) => false,
        }
    }

    fn formatted_string(&self) -> String {
        self.time.formatted_string()
    }
}

#[derive(Debug)]
enum TimerMsg {
    ToggleFlowTime,
    ToggleBreak,
    SetRestart(bool),
}

#[derive(Debug)]
enum CommandMsg {
    Tick,
    Empty,
}

#[relm4::component]
impl Component for Timer {
    type Init = TimerMode;
    type Input = TimerMsg;
    type Output = ();
    type CommandOutput = CommandMsg;

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_valign: gtk::Align::Center,
            set_spacing: 10,


            gtk::Label {
                add_css_class: "mode",
                #[watch]
                set_visible: !matches!(&model.mode, TimerMode::Stop),
                #[watch]
                set_label: match &model.mode {
                    TimerMode::Clock => "Working Stage",
                    TimerMode::CountDown => "Free Time!",
                    TimerMode::Stop => "",
                    TimerMode::Pause(x) => match **x {
                        TimerMode::Clock => "Working Stage",
                        TimerMode::CountDown => "Free Time!",
                        TimerMode::Stop => "",
                        TimerMode::Pause(_) => "",
                    },
                },
            },

            gtk::Label {
                add_css_class: "clock",
                #[watch]
                set_label: &model.formatted_string(),
            },
            append = &gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_halign: gtk::Align::Center,
                set_spacing: 10,

                gtk::Button {
                    set_label: "Break",
                    add_css_class: "circular",
                    add_css_class: "break",
                    connect_clicked => TimerMsg::ToggleBreak,
                },
                gtk::Button {
                    add_css_class: "circular",
                    add_css_class: "flowtimetoggle",
                    #[watch]
                    set_label: match &model.mode {
                        TimerMode::Stop | TimerMode::Pause(_) => "",
                        _ => "󰏤"
                    },
                    connect_clicked => TimerMsg::ToggleFlowTime,
                },
            }
        }
    }
    // Initialize the UI.
    fn init(
        _mode: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Timer::new();
        relm4::set_global_css(
            r#"
            .clock {
                font-size: 40px;
                color: rgb(153,209,219);
            }
            .flowtimetoggle {
                font-size: 25px;   
                padding: 15px;
            }
            .mode {
                font-size: 15px;
                color: rgb(153,209,219);
            }
            .circular {
                color: rgb(153,209,219);
                padding: 10px;
            }
            .break {
                font-size: 20px;
            }
"#,
        );

        // Insert the macro code generation here
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>, _: &Self::Root) {
        match msg {
            TimerMsg::ToggleBreak => match &self.mode {
                TimerMode::Clock | TimerMode::Pause(_) | TimerMode::Stop => {
                    self.mode = TimerMode::CountDown;
                    self.time.set_time_by_second(self.time.get_second() / 5);
                }
                _ => (),
            },
            TimerMsg::ToggleFlowTime => match &self.mode {
                TimerMode::Stop => {
                    self.mode = TimerMode::Clock;
                    self.time.reset_time();
                    if !self.clicking {
                        sender.spawn_oneshot_command(|| CommandMsg::Tick);
                        self.clicking = true;
                    }
                }
                TimerMode::Clock => {
                    self.mode = TimerMode::Pause(Box::from(TimerMode::Clock));
                }
                TimerMode::CountDown => {
                    self.mode = TimerMode::Pause(Box::from(TimerMode::CountDown));
                }
                TimerMode::Pause(x) => match **x {
                    TimerMode::Clock => {
                        self.mode = TimerMode::Clock;
                    }
                    TimerMode::CountDown => {
                        self.mode = TimerMode::CountDown;
                    }
                    _ => unreachable!(),
                },
            },
            TimerMsg::SetRestart(x) => {
                self.restart = x;
            }
        }
    }
    fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        if let CommandMsg::Tick = message {
            // ticking logic handled by Timer
            if self.tick() {
                sender.spawn_oneshot_command(move || {
                    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
                    let file: BufReader<std::fs::File> =
                        BufReader::new(File::open("tone.wav").unwrap());
                    let source = Decoder::new(file).unwrap();
                    // Create a sink to play the audio
                    let sink = Sink::try_new(&stream_handle).unwrap();
                    sink.append(source);
                    sink.sleep_until_end();
                    CommandMsg::Empty
                });
            }
            sender.spawn_oneshot_command(|| {
                std::thread::sleep(Duration::from_millis(1000));
                CommandMsg::Tick
            });
        };
    }
}

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
                connect_state_notify[sender] => move |switch| {
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

use lazy_static::lazy_static;

lazy_static! {
    static ref CFG: Config = confy::load("flowtime", Some("flowtime")).unwrap();
}

fn main() {
    let app = RelmApp::new("Flowtime");
    app.run::<MainApp>(AppMode::FlowTime);
}
