use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use gtk::prelude::*;
use relm4::{JoinHandle, actions::RelmActionGroup, adw::prelude::*};
use relm4::{actions::RelmAction, prelude::*};
use relm4_components::simple_adw_combo_row::SimpleComboRow;
use tokio_util::sync::CancellationToken;
use tracing::error;

use crate::{
    click::{ClickType, Clicker, MouseButton},
    dialogs::about::AboutDialog,
    icon_names,
    interval::ClickInterval,
};

const DEFAULT_WINDOW_WIDTH: i32 = 440;
const DEFAULT_WINDOW_HEIGHT: i32 = 612;
const MIN_CLICK_DELAY_MS: u64 = 5;

#[derive(Debug)]
pub enum AppMsg {
    Start,
    Stop,
    MouseButtonChanged(usize),
    ClickTypeChanged(usize),
    HoursChanged(f64),
    MinutesChanged(f64),
    SecondsChanged(f64),
    MillisecondsChanged(f64),
}

pub struct AppModel {
    mouse_button_row: Controller<SimpleComboRow<MouseButton>>,
    click_type_row: Controller<SimpleComboRow<ClickType>>,
    mouse_button: MouseButton,
    click_type: ClickType,
    interval: ClickInterval,
    is_running: bool,
    clicker: Option<Arc<Mutex<Clicker>>>,
    cancel_token: Option<CancellationToken>,
    click_task: Option<JoinHandle<()>>,
    runtime: tokio::runtime::Runtime,
}

impl AppModel {
    fn start_clicking(&mut self) {
        if self.is_running {
            return;
        }
        let Some(clicker) = &self.clicker else {
            error!("Clicker not initialized");
            return;
        };

        let delay = self.interval.total_milliseconds().max(MIN_CLICK_DELAY_MS);
        let button = self.mouse_button;
        let click_type = self.click_type;
        let clicker = Arc::clone(clicker);
        let cancel_token = CancellationToken::new();

        if let Ok(mut c) = clicker.lock() {
            c.set_delay(delay);
        }

        let token_clone = cancel_token.clone();
        self.is_running = true;

        let handle = self.runtime.spawn(async move {
            let mut next_tick = tokio::time::Instant::now();
            loop {
                next_tick += Duration::from_millis(delay);
                tokio::select! {
                    _ = tokio::time::sleep_until(next_tick) => {
                        if let Ok(mut c) = clicker.lock() {
                            if let Err(e) = c.click(button, click_type) {
                                error!("Click error: {}", e);
                                break;
                            }
                        }
                    }
                    _ = token_clone.cancelled() => break,
                }
            }
        });

        self.click_task = Some(handle);
        self.cancel_token = Some(cancel_token);
    }

    fn stop_clicking(&mut self) {
        if !self.is_running {
            return;
        }

        self.is_running = false;

        if let Some(token) = &self.cancel_token {
            token.cancel();
        }

        if let Some(handle) = self.click_task.take() {
            handle.abort();
        }

        self.cancel_token = None;
    }
}

relm4::new_action_group!(pub(super) WindowActionGroup, "win");
relm4::new_stateless_action!(AboutAction, WindowActionGroup, "about");

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
                        set_stack: Some(&clicker_stack),
                    },
                    pack_end = &gtk::MenuButton {
                        set_icon_name: icon_names::MENU,
                        set_menu_model: Some(&primary_menu)
                    },
                },

                #[name = "clicker_stack"]
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
                                        connect_value_notify[sender] => move |s| sender.input(AppMsg::HoursChanged(s.value())),
                                    },
                                    adw::SpinRow { set_title: "Minutes",
                                        set_adjustment: Some(&gtk::Adjustment::new(0.0, 0.0, 59.0, 1.0, 1.0, 0.0)),
                                        connect_value_notify[sender] => move |s| sender.input(AppMsg::MinutesChanged(s.value())),
                                    },
                                    adw::SpinRow { set_title: "Seconds",
                                        set_adjustment: Some(&gtk::Adjustment::new(0.0, 0.0, 59.0, 1.0, 1.0, 0.0)),
                                        connect_value_notify[sender] => move |s| sender.input(AppMsg::SecondsChanged(s.value())),
                                    },
                                    adw::SpinRow { set_title: "Milliseconds",
                                        set_adjustment: Some(&gtk::Adjustment::new(200.0, 0.0, 999.0, 1.0, 10.0, 0.0)),
                                        connect_value_notify[sender] => move |s| sender.input(AppMsg::MillisecondsChanged(s.value())),
                                    },
                                },

                                add = &adw::PreferencesGroup {
                                    set_title: "Input",
                                    #[local_ref]
                                    mouse_button_row -> adw::ComboRow { set_title: "Mouse Button" },
                                    #[local_ref]
                                    click_type_row -> adw::ComboRow { set_title: "Click Type" },
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
            HoursChanged(v) => self.interval.hours = v as u8,
            MinutesChanged(v) => self.interval.minutes = v as u8,
            SecondsChanged(v) => self.interval.seconds = v as u8,
            MillisecondsChanged(v) => self.interval.milliseconds = v as u16,
            MouseButtonChanged(i) => {
                self.mouse_button = [MouseButton::Left, MouseButton::Right, MouseButton::Middle]
                    .get(i)
                    .copied()
                    .unwrap_or(MouseButton::Left)
            }
            ClickTypeChanged(i) => {
                self.click_type = [ClickType::Single, ClickType::Double]
                    .get(i)
                    .copied()
                    .unwrap_or(ClickType::Single)
            }
        }
    }

    fn init(_init: (), _root: Self::Root, sender: ComponentSender<Self>) -> ComponentParts<Self> {
        let clicker = Clicker::new(200)
            .map(|c| Arc::new(Mutex::new(c)))
            .map_err(|e| error!("Failed to create clicker: {}", e))
            .ok();

        let mouse_button_row = SimpleComboRow::builder()
            .launch(SimpleComboRow {
                variants: vec![MouseButton::Left, MouseButton::Right, MouseButton::Middle],
                active_index: Some(0),
            })
            .forward(sender.input_sender(), AppMsg::MouseButtonChanged);

        let click_type_row = SimpleComboRow::builder()
            .launch(SimpleComboRow {
                variants: vec![ClickType::Single, ClickType::Double],
                active_index: Some(0),
            })
            .forward(sender.input_sender(), AppMsg::ClickTypeChanged);

        let model = AppModel {
            mouse_button_row,
            click_type_row,
            mouse_button: MouseButton::Left,
            click_type: ClickType::Single,
            interval: ClickInterval::default(),
            is_running: false,
            clicker,
            cancel_token: None,
            click_task: None,
            runtime: tokio::runtime::Runtime::new().unwrap(),
        };

        let mouse_button_row = model.mouse_button_row.widget();
        let click_type_row = model.click_type_row.widget();

        let widgets = view_output!();

        let mut actions = RelmActionGroup::<WindowActionGroup>::new();

        let about_action = {
            RelmAction::<AboutAction>::new_stateless(move |_| {
                AboutDialog::builder().launch(()).detach();
            })
        };

        actions.add_action(about_action);
        actions.register_for_widget(&widgets.main_window);

        ComponentParts { model, widgets }
    }
}
