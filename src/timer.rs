pub use crate::time::Time;
use crate::{cfg, stat, Config, Stats, CURRENT_MONTH};
use std::time::Duration;

use gtk::prelude::*;
use relm4::*;
use rodio::Sink;
use rodio::{Decoder, OutputStream};

use std::fs::File;
use std::io::BufReader;

#[derive(PartialEq, Debug, Clone)]
pub enum TimerMode {
    Clock,
    CountDown,
    Stop,
    Pause(Box<TimerMode>),
}

#[derive(Debug)]
pub struct Timer {
    pub mode: TimerMode,
    pub time: Time,
    pub clicking: bool,
}
impl Timer {
    fn new() -> Timer {
        Timer {
            mode: TimerMode::Stop,
            time: Default::default(),
            clicking: false,
        }
    }
    fn tick(&mut self) -> bool {
        match self.mode {
            TimerMode::Clock => {
                self.time.increment_second();
                false
            }
            TimerMode::CountDown => {
                if self.time.second == 0
                    && self.time.minutes == 0
                    && self.time.hour == 0
                    && self.mode != TimerMode::Clock
                {
                    if cfg!().restart {
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
pub enum TimerMsg {
    ToggleFlowTime,
    ToggleBreak,
    ResetSession,
}

#[derive(Debug)]
pub enum CommandMsg {
    Tick,
    Empty,
}

fn update_statistics(timer: &mut Timer, save_time: Option<(u32, u32)>) {
    let stats = stat!();
    for (conf_month, break_second, work_second) in stats.month_break_work.iter() {
        if &*CURRENT_MONTH == conf_month {
            confy::store(
                "flowtime",
                Some("statistics"),
                Stats {
                    month_break_work: vec![(
                        *conf_month,
                        save_time
                            .unwrap_or((break_second + timer.time.get_second() / 5, 0))
                            .0,
                        save_time
                            .unwrap_or((0, work_second + timer.time.get_second()))
                            .1,
                    )],
                },
            )
            .unwrap();
        } else if !stats
            .month_break_work
            .iter()
            .all(|(month, _, _)| month == &*CURRENT_MONTH)
        {
            let mut new_struct = stats.month_break_work.clone();
            new_struct.push((
                *CURRENT_MONTH,
                save_time
                    .unwrap_or((break_second + timer.time.get_second() / 5, 0))
                    .0,
                save_time
                    .unwrap_or((0, work_second + timer.time.get_second()))
                    .1,
            ));
            confy::store(
                "flowtime",
                Some("statistics"),
                Stats {
                    month_break_work: new_struct,
                },
            )
            .unwrap();
        }
    }
}

#[relm4::component(pub)]
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
            gtk::Box {
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
                    set_icon_name: match &model.mode {
                        TimerMode::Stop | TimerMode::Pause(_) => {
                            "media-playback-start"
                        },
                        _ => "media-playback-pause"
                    },
                    connect_clicked => TimerMsg::ToggleFlowTime,
                },
                gtk::Button {
                    add_css_class: "reset",
                    add_css_class: "circular",
                    set_label: "󰜉",
                    connect_clicked => TimerMsg::ResetSession,
                }
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
            .reset {
                font-size: 25px;
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
                    update_statistics(self, None);
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
            TimerMsg::ResetSession => {
                for (month, break_time, work) in stat!().month_break_work.iter() {
                    if *CURRENT_MONTH == *month {
                        if self.mode == TimerMode::CountDown {
                            self.mode = TimerMode::Clock;
                            if cfg!().reset_save {
                                update_statistics(
                                    self,
                                    Some((break_time - self.time.second, *work)),
                                );
                            }
                        } else if self.mode == TimerMode::Clock {
                            if cfg!().reset_save {
                                update_statistics(
                                    self,
                                    Some((*break_time, work + self.time.second)),
                                );
                            }
                        }
                    }
                }
                self.time.reset_time();
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
