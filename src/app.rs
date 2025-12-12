use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};

use gtk::prelude::*;
use relm4::adw::prelude::*;
use relm4::prelude::*;

use crate::{
    click::{Clicker, MouseButton},
    icon_names,
    interval::{ClickInterval, IntervalField},
};

#[derive(Debug)]
pub enum AppMsg {
    Start,
    Stop,

    IntervalChanged { field: IntervalField, value: f64 },
    MouseButtonChanged(u32),
}

#[tracker::track]
pub struct AppModel {
    is_running: bool,
    mouse_button: MouseButton,
    interval: ClickInterval,

    #[tracker::no_eq]
    clicker: Option<Arc<Mutex<Clicker>>>,

    #[tracker::no_eq]
    click_thread: Option<thread::JoinHandle<()>>,

    #[tracker::no_eq]
    should_stop: Arc<AtomicBool>,
}

impl AppModel {
    fn create_spin_row(
        title: &str,
        value: f64,
        min: f64,
        max: f64,
        field: IntervalField,
        sender: &ComponentSender<Self>,
    ) -> adw::SpinRow {
        let adjustment = gtk::Adjustment::new(value, min, max, 1.0, 1.0, 0.0);
        let row = adw::SpinRow::new(Some(&adjustment), 1.0, 0);
        row.set_title(title);

        let sender = sender.clone();
        row.connect_changed(move |r| {
            sender.input(AppMsg::IntervalChanged {
                field,
                value: r.value(),
            });
        });

        row
    }
}

#[relm4::component(pub)]
impl SimpleComponent for AppModel {
    type Init = ();
    type Input = AppMsg;
    type Output = ();

    view! {
        adw::ApplicationWindow {
            set_default_width: 500,
            set_default_height: 640,
            set_resizable: false,

            #[wrap(Some)]
            set_content = &gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                adw::HeaderBar {
                    #[wrap(Some)]
                    set_title_widget = &adw::ViewSwitcher {
                        set_policy: adw::ViewSwitcherPolicy::Wide,
                        set_stack: Some(&view_stack),
                    }
                },

                #[name = "view_stack"]
                adw::ViewStack {
                    set_vexpand: true,

                    add_titled_with_icon[Some("clicker"), "Clicker", icon_names::MOUSE_CLICK] = &gtk::ScrolledWindow {
                        adw::Clamp {
                            set_maximum_size: 600,

                            adw::PreferencesPage {
                                set_title: "Clicker",

                                add = &adw::PreferencesGroup {
                                    set_title: "Interval",

                                    #[local_ref]
                                    hours_row -> adw::SpinRow {},

                                    #[local_ref]
                                    minutes_row -> adw::SpinRow {},

                                    #[local_ref]
                                    seconds_row -> adw::SpinRow {},

                                    #[local_ref]
                                    milliseconds_row -> adw::SpinRow {},
                                },

                                add = &adw::PreferencesGroup {
                                    set_title: "Input",

                                    adw::ComboRow {
                                        set_title: "Mouse Button",
                                        set_model: Some(&gtk::StringList::new(&[
                                            "Left", "Right", "Middle"
                                        ])),
                                        connect_selected_notify[sender] => move |row| {
                                            sender.input(AppMsg::MouseButtonChanged(row.selected()))
                                        }
                                    },

                                    adw::ComboRow {
                                        set_title: "Click Type",
                                        set_model: Some(&gtk::StringList::new(&[
                                            "Single", "Hold"
                                        ])),
                                    },
                                },
                            }
                        }
                    }
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 10,
                    set_margin_all: 30,

                    adw::Clamp {
                        set_maximum_size: 500,

                        gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_spacing: 10,
                            set_homogeneous: true,

                            gtk::Button {
                                set_label: "Start",
                                add_css_class: "pill",
                                add_css_class: "suggested-action",
                                #[watch]
                                set_sensitive: !model.is_running,
                                connect_clicked => AppMsg::Start,
                            },

                            gtk::Button {
                                set_label: "Stop",
                                add_css_class: "pill",
                                #[watch]
                                set_sensitive: model.is_running,
                                connect_clicked => AppMsg::Stop,
                            },
                        }
                    }
                }
            }
        }
    }

    fn update(&mut self, msg: AppMsg, _sender: ComponentSender<Self>) {
        self.reset();
        match msg {
            AppMsg::Start => {
                if let Some(clicker) = self.clicker.clone() {
                    self.set_is_running(true);

                    let delay = self.interval.total_milliseconds();
                    let button = self.mouse_button;

                    self.should_stop.store(false, Ordering::Relaxed);
                    let should_stop = self.should_stop.clone();

                    if let Ok(mut c) = clicker.lock() {
                        c.set_delay(delay);
                    }

                    let handle = thread::spawn(move || {
                        while !should_stop.load(Ordering::Relaxed) {
                            thread::sleep(Duration::from_millis(delay));

                            if should_stop.load(Ordering::Relaxed) {
                                break;
                            }

                            match clicker.lock() {
                                Ok(mut c) => {
                                    if let Err(e) = c.click(button, crate::click::ClickType::Single)
                                    {
                                        eprintln!("Click error: {}", e);
                                        break;
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Failed to lock clicker: {}", e);
                                    break;
                                }
                            }
                        }
                    });

                    self.click_thread = Some(handle);
                }
            }
            AppMsg::Stop => {
                self.set_is_running(false);

                self.should_stop.store(true, Ordering::Relaxed);

                if let Some(handle) = self.click_thread.take() {
                    let _ = handle.join();
                }
            }
            AppMsg::IntervalChanged { field, value } => match field {
                IntervalField::Hours => self.interval.hours = value as u8,
                IntervalField::Minutes => self.interval.minutes = value as u8,
                IntervalField::Seconds => self.interval.seconds = value as u8,
                IntervalField::Milliseconds => self.interval.milliseconds = value as u16,
            },
            AppMsg::MouseButtonChanged(index) => {
                self.mouse_button = match index {
                    1 => MouseButton::Right,
                    2 => MouseButton::Middle,
                    _ => MouseButton::Left,
                }
            }
        }
    }

    fn init(
        _: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let clicker = match Clicker::new(200) {
            Ok(c) => Some(Arc::new(Mutex::new(c))),
            Err(e) => {
                eprintln!("Failed to create clicker: {}", e);
                None
            }
        };

        let model = AppModel {
            is_running: false,
            tracker: 0,
            interval: ClickInterval {
                hours: 0,
                minutes: 0,
                seconds: 0,
                milliseconds: 200,
            },
            mouse_button: MouseButton::Left,
            clicker,
            click_thread: None,
            should_stop: Arc::new(AtomicBool::new(false)),
        };

        let hours_row = Self::create_spin_row(
            "Hours",
            model.interval.hours as f64,
            0.0,
            23.0,
            IntervalField::Hours,
            &sender,
        );
        let minutes_row = Self::create_spin_row(
            "Minutes",
            model.interval.minutes as f64,
            0.0,
            59.0,
            IntervalField::Minutes,
            &sender,
        );
        let seconds_row = Self::create_spin_row(
            "Seconds",
            model.interval.seconds as f64,
            0.0,
            59.0,
            IntervalField::Seconds,
            &sender,
        );
        let milliseconds_row = Self::create_spin_row(
            "Milliseconds",
            model.interval.milliseconds as f64,
            5.0,
            999.0,
            IntervalField::Milliseconds,
            &sender,
        );

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}
