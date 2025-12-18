use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use gtk::prelude::*;

use relm4::{
    JoinHandle,
    actions::{AccelsPlus, RelmAction, RelmActionGroup},
    adw::prelude::*,
    prelude::*,
};

use relm4_components::simple_adw_combo_row::SimpleComboRow;

use tracing::error;

use crate::{
    dialogs::about::AboutDialog,
    icon_names,
    input::{ClickAction, InputHandler, MouseButton},
    interval::ClickInterval,
};

const DEFAULT_WINDOW_WIDTH: i32 = 440;
const DEFAULT_WINDOW_HEIGHT: i32 = 612;
const MIN_CLICK_DELAY_MS: u64 = 1;
const DEFAULT_TOGGLE_CLICKING_BIND: &str = "F6";

#[derive(Debug)]
pub enum IntervalField {
    Hours(f64),
    Minutes(f64),
    Seconds(f64),
    Milliseconds(f64),
}

#[derive(Debug)]
pub enum AppMsg {
    Start,
    Stop,
    Toggle,
    MouseButtonChanged(usize),
    ClickActionChanged(usize),
    IntervalChanged(IntervalField),
}

pub struct AppModel {
    mouse_button_row: Controller<SimpleComboRow<MouseButton>>,
    click_action_row: Controller<SimpleComboRow<ClickAction>>,
    mouse_button: MouseButton,
    click_action: ClickAction,
    interval: ClickInterval,
    is_running: bool,
    input_handler: Option<Arc<Mutex<InputHandler>>>,
    click_task: Option<JoinHandle<()>>,
}

impl AppModel {
    fn start_clicking(&mut self) {
        if self.is_running {
            return;
        }

        let Some(input_handler) = &self.input_handler else {
            error!("Input handler not initialized");
            return;
        };

        let delay = self.interval.total_milliseconds().max(MIN_CLICK_DELAY_MS);
        let button = self.mouse_button;
        let click_action = self.click_action;
        let input_handler = Arc::clone(input_handler);

        self.is_running = true;

        let handle = relm4::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(delay));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                interval.tick().await;
                if let Err(e) = input_handler.lock().unwrap().click(button, click_action) {
                    error!("Click error: {}", e);
                    break;
                }
            }
        });

        self.click_task = Some(handle);
    }

    fn stop_clicking(&mut self) {
        if !self.is_running {
            return;
        }

        self.is_running = false;

        if let Some(handle) = self.click_task.take() {
            handle.abort();
        }
    }
}

relm4::new_action_group!(pub(super) WindowActionGroup, "win");
relm4::new_stateless_action!(AboutAction, WindowActionGroup, "about");
relm4::new_stateless_action!(ToggleClickingAction, WindowActionGroup, "toggle_clicking");

#[relm4::component(pub)]
impl SimpleComponent for AppModel {
    type Init = ();
    type Input = AppMsg;
    type Output = ();

    menu! {
        primary_menu: {
            section! {
                "_About Click" => AboutAction,
            }
        }
    }

    view! {
        main_window = adw::ApplicationWindow {
            set_default_width: DEFAULT_WINDOW_WIDTH,
            set_default_height: DEFAULT_WINDOW_HEIGHT,
            set_resizable: false,

            #[wrap(Some)]
            set_content = &gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                adw::HeaderBar {
                    #[wrap(Some)]
                    set_title_widget = &adw::ViewSwitcher {
                        set_policy: adw::ViewSwitcherPolicy::Wide,
                        set_stack: Some(&view_stack),
                    },
                    pack_end = &gtk::MenuButton {
                        set_icon_name: icon_names::MENU,
                        set_menu_model: Some(&primary_menu)
                    },
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

                                    adw::SpinRow { set_title: "Hours",
                                        set_adjustment: Some(&gtk::Adjustment::new(0.0, 0.0, 23.0, 1.0, 1.0, 0.0)),
                                        connect_value_notify[sender] => move |s| {
                                            sender.input(AppMsg::IntervalChanged(IntervalField::Hours(s.value())))
                                        },
                                    },
                                    adw::SpinRow { set_title: "Minutes",
                                        set_adjustment: Some(&gtk::Adjustment::new(0.0, 0.0, 59.0, 1.0, 1.0, 0.0)),
                                        connect_value_notify[sender] => move |s| {
                                            sender.input(AppMsg::IntervalChanged(IntervalField::Minutes(s.value())))
                                        },
                                    },
                                    adw::SpinRow { set_title: "Seconds",
                                        set_adjustment: Some(&gtk::Adjustment::new(0.0, 0.0, 59.0, 1.0, 1.0, 0.0)),
                                        connect_value_notify[sender] => move |s| {
                                            sender.input(AppMsg::IntervalChanged(IntervalField::Seconds(s.value())))
                                        },
                                    },
                                    adw::SpinRow { set_title: "Milliseconds",
                                        set_adjustment: Some(&gtk::Adjustment::new(200.0, 0.0, 999.0, 1.0, 10.0, 0.0)),
                                        connect_value_notify[sender] => move |s| {
                                            sender.input(AppMsg::IntervalChanged(IntervalField::Milliseconds(s.value())))
                                        },
                                    },
                                },

                                add = &adw::PreferencesGroup {
                                    set_title: "Input",
                                    #[local_ref]
                                    mouse_button_row -> adw::ComboRow { set_title: "Mouse Button" },
                                    #[local_ref]
                                    click_action_row -> adw::ComboRow { set_title: "Click Type" },
                                },
                            }
                        }
                    }
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 10,
                    set_margin_all: 20,

                    adw::Clamp {
                        set_maximum_size: 500,

                        gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_spacing: 10,
                            set_homogeneous: true,

                            gtk::Button {
                                set_label: &format!("Start ({DEFAULT_TOGGLE_CLICKING_BIND})"),
                                add_css_class: "pill",
                                add_css_class: "suggested-action",
                                #[watch]
                                set_sensitive: !model.is_running,
                                connect_clicked => AppMsg::Start,
                            },

                            gtk::Button {
                                set_label: &format!("Stop ({DEFAULT_TOGGLE_CLICKING_BIND})"),
                                add_css_class: "pill",
                                #[watch]
                                set_sensitive: model.is_running,
                                connect_clicked => AppMsg::Stop,
                            },
                        },
                    },
                }
            }
        }
    }

    fn update(&mut self, msg: AppMsg, _sender: ComponentSender<Self>) {
        use AppMsg::*;
        match msg {
            Start => self.start_clicking(),
            Stop => self.stop_clicking(),
            Toggle => {
                if self.is_running {
                    self.stop_clicking();
                } else {
                    self.start_clicking();
                }
            }
            IntervalChanged(interval) => match interval {
                IntervalField::Hours(v) => self.interval.hours = v as u8,
                IntervalField::Minutes(v) => self.interval.minutes = v as u8,
                IntervalField::Seconds(v) => self.interval.seconds = v as u8,
                IntervalField::Milliseconds(v) => self.interval.milliseconds = v as u16,
            },
            MouseButtonChanged(i) => {
                self.mouse_button = [MouseButton::Left, MouseButton::Right, MouseButton::Middle]
                    .get(i)
                    .copied()
                    .unwrap_or(MouseButton::Left)
            }
            ClickActionChanged(i) => {
                self.click_action = [ClickAction::Single, ClickAction::Double]
                    .get(i)
                    .copied()
                    .unwrap_or(ClickAction::Single)
            }
        }
    }

    fn init(_init: (), _root: Self::Root, sender: ComponentSender<Self>) -> ComponentParts<Self> {
        let input_handler = InputHandler::new()
            .map(|c| Arc::new(Mutex::new(c)))
            .map_err(|e| error!("Failed to create clicker: {}", e))
            .ok();

        let mouse_button_row = SimpleComboRow::builder()
            .launch(SimpleComboRow {
                variants: vec![MouseButton::Left, MouseButton::Right, MouseButton::Middle],
                active_index: Some(0),
            })
            .forward(sender.input_sender(), AppMsg::MouseButtonChanged);

        let click_action_row = SimpleComboRow::builder()
            .launch(SimpleComboRow {
                variants: vec![ClickAction::Single, ClickAction::Double],
                active_index: Some(0),
            })
            .forward(sender.input_sender(), AppMsg::ClickActionChanged);

        let model = AppModel {
            mouse_button_row,
            click_action_row,
            mouse_button: MouseButton::Left,
            click_action: ClickAction::Single,
            interval: ClickInterval::default(),
            is_running: false,
            input_handler,
            click_task: None,
        };

        let mouse_button_row = model.mouse_button_row.widget();
        let click_action_row = model.click_action_row.widget();

        let widgets = view_output!();

        let app = relm4::main_application();
        app.set_accelerators_for_action::<ToggleClickingAction>(&[DEFAULT_TOGGLE_CLICKING_BIND]);

        let mut actions = RelmActionGroup::<WindowActionGroup>::new();

        let about_action = {
            RelmAction::<AboutAction>::new_stateless(move |_| {
                AboutDialog::builder().launch(()).detach();
            })
        };

        let toggle_action = {
            let sender = sender.clone();
            RelmAction::<ToggleClickingAction>::new_stateless(move |_| {
                sender.input(AppMsg::Toggle);
            })
        };

        actions.add_action(about_action);
        actions.add_action(toggle_action);
        actions.register_for_widget(&widgets.main_window);

        ComponentParts { model, widgets }
    }
}
