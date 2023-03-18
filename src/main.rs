// use crossterm::{
//     cursor,
//     event::{self, Event},
//     execute,
//     terminal::{Clear, ClearType},
//     QueueableCommand,
// };
// use gtk::prelude::{BoxExt, ButtonExt, GtkWindowExt, OrientableExt, WidgetExt};
use gtk::prelude::*;
use relm4::*;
// use relm4::{
//     gtk, Component, ComponentParts, ComponentSender, Controller, RelmApp, RelmWidgetExt,
//     SimpleComponent,
// };
use std::time::Duration;
// use std::io::{stdout, Write};
// use std::thread::sleep;

#[derive(Default, Debug, Clone)]
struct Time {
    second: u16,
    minutes: u16,
    hour: u16,
}

impl Time {
    fn increment_second(&mut self) {
        self.second += 1;
        self.check_carry();
    }
    fn check_carry(&mut self) {
        if self.second == 59 {
            self.second = 0;
            self.minutes += 1;
        }
        if self.minutes == 59 {
            self.minutes = 0;
            self.hour += 1;
        }
        if self.hour == 23 {
            self.hour = 0;
        }
    }
    fn decrement_second(&mut self) {
        if self.second == 0 && self.minutes > 0 {
            self.second = 59;
            self.minutes -= 1;
        } else if self.minutes == 0 && self.hour > 0 {
            self.minutes = 59;
            self.hour -= 1;
        } else {
            self.second -= 1;
        }
    }
    fn formatted_string(&self) -> String {
        let mut output = String::new();
        if self.hour < 10 {
            output.push_str(&format!("0{}:", self.hour));
        } else {
            output.push_str(&format!("{}:", self.hour));
        }
        if self.minutes < 10 {
            output.push_str(format!("0{}:", self.minutes).as_str());
        } else {
            output.push_str(&format!("{}:", self.minutes));
        }
        if self.second < 10 {
            output.push_str(format!("0{}", self.second).as_str());
        } else {
            output.push_str(&format!("{}", self.second));
        }
        output
    }
    fn set_time_by_second(&mut self, seconds: u16) {
        self.hour = seconds / 3600;
        self.minutes = (seconds % 3600) / 60;
        self.second = seconds % 60;
    }
    fn get_second(&self) -> u16 {
        self.second + self.minutes * 60 + self.hour * 3600
    }
    fn reset_time(&mut self) {
        self.second = 0;
        self.minutes = 0;
        self.hour = 0;
    }
}

#[derive(PartialEq, Debug, Clone)]
enum Mode {
    Clock,
    CountDown,
    Stop,
    Pause(Box<Mode>),
}
#[derive(Debug)]
struct Timer {
    mode: Mode,
    time: Time,
    clicking: bool,
    restart: bool,
}
impl Timer {
    fn new() -> Timer {
        Timer {
            mode: Mode::Stop,
            time: Default::default(),
            clicking: false,
            restart: false,
        }
    }
    fn tick(&mut self) {
        match self.mode {
            Mode::Clock => self.time.increment_second(),
            Mode::CountDown => {
                if self.time.second == 0 && self.time.minutes == 0 && self.time.hour == 0 {
                    if self.restart {
                        self.mode = Mode::Clock;
                    } else {
                        self.mode = Mode::Stop;
                    }
                    self.clicking = false;
                    self.time.reset_time();
                } else {
                    self.time.decrement_second()
                }
            }
            Mode::Stop => {}
            Mode::Pause(_) => {}
        }
    }

    fn set_time(&mut self, t: Time) {
        match self.mode {
            Mode::Clock => panic!("Can't set time on clock mode"),
            Mode::CountDown => self
                .time
                .set_time_by_second(t.second + t.minutes * 60 + t.hour * 3600),
            Mode::Stop => panic!("Can't set time with Stops"),
            Mode::Pause(_) => panic!("Can't set time with Pauses"),
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
}

#[relm4::component]
impl Component for Timer {
    type Init = Mode;
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
                set_visible: !matches!(&model.mode, Mode::Stop),
                #[watch]
                set_label: match &model.mode {
                    Mode::Clock => "Working Stage",
                    Mode::CountDown => "Free Time!",
                    Mode::Stop => "",
                    Mode::Pause(x) => match **x {
                        Mode::Clock => "Working Stage",
                        Mode::CountDown => "Free Time!",
                        Mode::Stop => "",
                        Mode::Pause(_) => "",
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
                        Mode::Stop | Mode::Pause(_) => "",
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
                Mode::Clock | Mode::Pause(_) | Mode::Stop => {
                    self.mode = Mode::CountDown;
                    self.time.set_time_by_second(self.time.get_second() / 5);
                    if !self.clicking {
                        sender.spawn_oneshot_command(|| CommandMsg::Tick);
                        self.clicking = true;
                    }
                }
                _ => (),
            },
            TimerMsg::ToggleFlowTime => match &self.mode {
                Mode::Stop => {
                    self.mode = Mode::Clock;
                    self.time.reset_time();
                    if !self.clicking {
                        sender.spawn_oneshot_command(|| CommandMsg::Tick);
                        self.clicking = true;
                    }
                }
                Mode::Clock => {
                    self.mode = Mode::Pause(Box::from(Mode::Clock));
                }
                Mode::CountDown => {
                    self.mode = Mode::Pause(Box::from(Mode::CountDown));
                }
                Mode::Pause(x) => {
                    self.mode = *x.clone();
                }
            },
            TimerMsg::SetRestart(x) => {
                self.restart = x;
                dbg!(&self);
            }
        }
    }
    fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            CommandMsg::Tick => {
                if self.mode != Mode::Stop
                    && self.mode != Mode::Pause(Box::from(Mode::Clock))
                    && self.mode != Mode::Pause(Box::from(Mode::CountDown))
                {
                    self.tick();
                    sender.spawn_oneshot_command(|| {
                        std::thread::sleep(Duration::from_millis(1000));
                        CommandMsg::Tick
                    });
                }
            }
        };
    }
}

struct HeaderModel;

#[derive(Debug)]
enum HeaderOutput {
    FlowTime,
    Settings,
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
                connect_state_notify[sender] => move |switch| {
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
                },
            ),
            main: Timer::builder().launch(Mode::Stop).detach(),
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

fn main() {
    let app = RelmApp::new("Flowtime");
    app.run::<MainApp>(AppMode::FlowTime);
}
