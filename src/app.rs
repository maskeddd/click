use eframe::egui::{self, Key, KeyboardShortcut, Modifiers};
use serde::{Deserialize, Serialize};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::{runtime::Runtime, sync::mpsc, task::JoinHandle};
use tracing::error;

use crate::{
    InputHandler,
    input::{ClickAction, Coordinates, MouseButton},
    interval::{IntervalMode, Jitter, TimeInterval},
};

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct ClickApp {
    interval_mode: IntervalMode,
    time_interval: TimeInterval,
    cps: u16,
    jitter: u16,
    use_jitter: bool,

    mouse_button: MouseButton,
    click_type: ClickAction,
    num_clicks: u32,
    use_num_clicks: bool,
    location: Coordinates,
    use_location: bool,

    start_shortcut: KeyboardShortcut,
    stop_shortcut: KeyboardShortcut,

    #[serde(skip)]
    #[serde(default)]
    clicker: ClickerState,
}

enum ClickerStatus {
    Completed,
    Error(String),
}

struct ClickerConfig {
    base_interval: Duration,
    mouse_button: MouseButton,
    click_action: ClickAction,
    use_jitter: bool,
    jitter: u16,
    use_location: bool,
    location: Coordinates,
    use_num_clicks: bool,
    num_clicks: u32,
}

async fn wait_for_tick(
    config: &ClickerConfig,
    jitter_gen: &mut Jitter,
    interval: &mut tokio::time::Interval,
    next_tick: &mut tokio::time::Instant,
) {
    if config.use_jitter && config.jitter > 0 {
        let delay = jitter_gen.next(config.base_interval, config.jitter);
        *next_tick += delay;
        tokio::time::sleep_until(*next_tick).await;
    } else {
        interval.tick().await;
    }
}

impl ClickerConfig {
    fn from_app(app: &ClickApp) -> Self {
        Self {
            base_interval: app.calculate_interval(),
            mouse_button: app.mouse_button,
            click_action: app.click_type,
            use_jitter: app.use_jitter,
            jitter: app.jitter,
            use_location: app.use_location,
            location: app.location,
            use_num_clicks: app.use_num_clicks,
            num_clicks: app.num_clicks,
        }
    }
}

struct ClickerState {
    stop_sender: Option<mpsc::Sender<()>>,
    status_receiver: Option<mpsc::Receiver<ClickerStatus>>,
    task_handle: Option<JoinHandle<()>>,
    runtime: Arc<Runtime>,
    input_handler: Option<Arc<Mutex<InputHandler>>>,
    is_running: bool,
}

impl Default for ClickerState {
    fn default() -> Self {
        let input_handler = InputHandler::new()
            .map(|h| Arc::new(Mutex::new(h)))
            .map_err(|e| error!("Failed to create input handler: {}", e))
            .ok();

        let runtime = Arc::new(Runtime::new().expect("Failed to create tokio runtime"));

        Self {
            stop_sender: None,
            status_receiver: None,
            task_handle: None,
            runtime,
            input_handler,
            is_running: false,
        }
    }
}

impl Default for ClickApp {
    fn default() -> Self {
        Self {
            interval_mode: IntervalMode::Time,
            time_interval: TimeInterval::default(),
            cps: 20,
            jitter: 0,
            use_jitter: false,
            mouse_button: MouseButton::Left,
            click_type: ClickAction::Single,
            num_clicks: 100,
            use_num_clicks: false,
            location: Coordinates::default(),
            use_location: false,
            start_shortcut: KeyboardShortcut::new(Modifiers::NONE, Key::F6),
            stop_shortcut: KeyboardShortcut::new(Modifiers::NONE, Key::F7),
            clicker: ClickerState::default(),
        }
    }
}

impl ClickApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        }
    }

    fn perform_click(
        input_handler: &Arc<Mutex<InputHandler>>,
        config: &ClickerConfig,
    ) -> Result<(), String> {
        input_handler
            .lock()
            .map_err(|e| format!("Lock error: {e}"))
            .and_then(|mut handler| {
                if config.use_location {
                    handler.click_at(config.location, config.mouse_button, config.click_action)
                } else {
                    handler.click(config.mouse_button, config.click_action)
                }
                .map_err(|e| format!("Click failed: {e}"))
            })
    }

    fn calculate_interval(&self) -> Duration {
        match self.interval_mode {
            IntervalMode::Time => self.time_interval.to_duration(),
            IntervalMode::Cps => {
                let cps = self.cps.max(1) as u64;
                Duration::from_secs_f64(1.0 / cps as f64)
            }
        }
    }

    fn start_clicker(&mut self, ctx: &egui::Context) {
        if self.clicker.is_running {
            return;
        }

        let input_handler = match self.clicker.input_handler.as_ref() {
            Some(handler) => Arc::clone(handler),
            None => {
                error!("Input handler not available");
                return;
            }
        };

        let (stop_tx, mut stop_rx) = mpsc::channel::<()>(1);
        let (status_tx, status_rx) = mpsc::channel::<ClickerStatus>(1);

        let config = ClickerConfig::from_app(self);

        let repaint_ctx = ctx.clone();

        let handle = self.clicker.runtime.spawn(async move {
            let mut jitter_gen = Jitter::new();
            let mut click_count = 0u32;
            let mut interval = tokio::time::interval(config.base_interval);
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            interval.tick().await;
            let mut next_tick = tokio::time::Instant::now();

            loop {
                tokio::select! {
                    _ = wait_for_tick(&config, &mut jitter_gen, &mut interval, &mut next_tick) => {
                        if config.use_num_clicks && click_count >= config.num_clicks {
                            let _ = status_tx.send(ClickerStatus::Completed).await;
                            repaint_ctx.request_repaint();
                            break;
                        }

                        let result = ClickApp::perform_click(&input_handler, &config);

                        if let Err(e) = result {
                            let _ = status_tx.send(ClickerStatus::Error(e)).await;
                            repaint_ctx.request_repaint();
                            break;
                        }

                        click_count += 1;
                    }
                    _ = stop_rx.recv() => {
                        repaint_ctx.request_repaint();
                        break;
                    }
                }
            }
        });

        self.clicker.stop_sender = Some(stop_tx);
        self.clicker.status_receiver = Some(status_rx);
        self.clicker.task_handle = Some(handle);
        self.clicker.is_running = true;
    }

    fn stop_clicker(&mut self) {
        if !self.clicker.is_running {
            return;
        }

        if let Some(sender) = self.clicker.stop_sender.take() {
            let _ = sender.try_send(());
        }

        self.clicker.task_handle = None;
        self.clicker.status_receiver = None;
        self.clicker.is_running = false;
    }
}

impl Drop for ClickApp {
    fn drop(&mut self) {
        if let Some(handle) = self.clicker.task_handle.take() {
            handle.abort();
        }
    }
}

impl eframe::App for ClickApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let was_running = self.clicker.is_running;

        // Check for status updates
        if let Some(receiver) = &mut self.clicker.status_receiver {
            let mut last_status = None;
            while let Ok(status) = receiver.try_recv() {
                last_status = Some(status);
            }

            if let Some(status) = last_status {
                match status {
                    ClickerStatus::Completed => self.stop_clicker(),
                    ClickerStatus::Error(e) => {
                        error!("Clicker error: {}", e);
                        self.stop_clicker();
                    }
                }
            }
        }

        // Handle keyboard shortcuts
        ctx.input_mut(|i| {
            if i.consume_shortcut(&self.start_shortcut) && !self.clicker.is_running {
                self.start_clicker(ctx);
            }
            if i.consume_shortcut(&self.stop_shortcut) && self.clicker.is_running {
                self.stop_clicker();
                ctx.request_repaint();
            }
        });

        if was_running && !self.clicker.is_running {
            ctx.request_repaint();
        }

        // Bottom panel
        egui::TopBottomPanel::bottom("bottom_panel")
            .frame(
                egui::Frame::default()
                    .fill(egui::Color32::from_rgb(26, 26, 26))
                    .inner_margin(egui::Margin::same(12)),
            )
            .show(ctx, |ui| {
                ui.columns_const(|[left, right]| {
                    left.add_enabled_ui(!self.clicker.is_running, |ui| {
                        if ui
                            .add_sized(
                                [0.0, 36.0],
                                egui::Button::new(format!(
                                    "Start ({})",
                                    ctx.format_shortcut(&self.start_shortcut)
                                )),
                            )
                            .clicked()
                        {
                            self.start_clicker(ctx);
                        }
                    });

                    right.add_enabled_ui(self.clicker.is_running, |ui| {
                        if ui
                            .add_sized(
                                [0.0, 36.0],
                                egui::Button::new(format!(
                                    "Stop ({})",
                                    ctx.format_shortcut(&self.stop_shortcut)
                                )),
                            )
                            .clicked()
                        {
                            self.stop_clicker();
                            ctx.request_repaint();
                        }
                    });
                });

                egui::warn_if_debug_build(ui);
            });

        // Main configuration panel
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_enabled_ui(!self.clicker.is_running, |ui| {
                // Interval section
                ui.heading("Interval");
                ui.add_space(6.0);

                egui::Grid::new("interval_grid")
                    .num_columns(2)
                    .spacing([10.0, 4.0])
                    .show(ui, |ui| {
                        ui.radio_value(&mut self.interval_mode, IntervalMode::Time, "Time:");
                        ui.add_enabled_ui(self.interval_mode == IntervalMode::Time, |ui| {
                            ui.horizontal(|ui| {
                                ui.label("H:");
                                ui.add(
                                    egui::DragValue::new(&mut self.time_interval.hours)
                                        .speed(0.1)
                                        .range(0..=23),
                                );
                                ui.label("M:");
                                ui.add(
                                    egui::DragValue::new(&mut self.time_interval.minutes)
                                        .speed(0.1)
                                        .range(0..=59),
                                );
                                ui.label("S:");
                                ui.add(
                                    egui::DragValue::new(&mut self.time_interval.seconds)
                                        .speed(0.1)
                                        .range(0..=59),
                                );
                                ui.label("MS:");
                                ui.add(
                                    egui::DragValue::new(&mut self.time_interval.milliseconds)
                                        .speed(1.0)
                                        .range(0..=999),
                                );
                            });
                        });
                        ui.end_row();

                        ui.radio_value(&mut self.interval_mode, IntervalMode::Cps, "Target CPS:");
                        ui.add_enabled_ui(self.interval_mode == IntervalMode::Cps, |ui| {
                            ui.add(
                                egui::DragValue::new(&mut self.cps)
                                    .speed(0.1)
                                    .range(1..=1000),
                            );
                        });
                        ui.end_row();
                    });

                ui.add(egui::Separator::default().spacing(18.0));

                // Behavior section
                ui.heading("Behavior");
                ui.add_space(6.0);

                egui::Grid::new("behavior_grid")
                    .num_columns(2)
                    .spacing([10.0, 4.0])
                    .show(ui, |ui| {
                        ui.label("Mouse button:");
                        egui::ComboBox::from_id_salt("mouse_button")
                            .selected_text(self.mouse_button.to_string())
                            .show_ui(ui, |ui| {
                                for variant in MouseButton::all() {
                                    ui.selectable_value(
                                        &mut self.mouse_button,
                                        variant,
                                        variant.to_string(),
                                    );
                                }
                            });
                        ui.end_row();

                        ui.label("Click type:");
                        egui::ComboBox::from_id_salt("click_type")
                            .selected_text(self.click_type.to_string())
                            .show_ui(ui, |ui| {
                                for variant in ClickAction::all() {
                                    ui.selectable_value(
                                        &mut self.click_type,
                                        variant,
                                        variant.to_string(),
                                    );
                                }
                            });
                        ui.end_row();

                        ui.checkbox(&mut self.use_num_clicks, "Repeat only:");
                        ui.add_enabled_ui(self.use_num_clicks, |ui| {
                            ui.add(
                                egui::DragValue::new(&mut self.num_clicks)
                                    .speed(1.0)
                                    .range(1..=u32::MAX)
                                    .suffix(" clicks"),
                            );
                        });
                        ui.end_row();
                    });

                ui.add(egui::Separator::default().spacing(18.0));

                // Extra section
                ui.heading("Extra");
                ui.add_space(6.0);

                egui::Grid::new("extra_grid")
                    .num_columns(2)
                    .spacing([10.0, 4.0])
                    .show(ui, |ui| {
                        ui.checkbox(&mut self.use_jitter, "Random delay:");
                        ui.add_enabled_ui(self.use_jitter, |ui| {
                            ui.add(
                                egui::DragValue::new(&mut self.jitter)
                                    .speed(0.1)
                                    .range(0..=1000)
                                    .prefix("± ")
                                    .suffix(" ms"),
                            );
                        });
                        ui.end_row();

                        ui.checkbox(&mut self.use_location, "Location:");
                        ui.add_enabled_ui(self.use_location, |ui| {
                            ui.horizontal(|ui| {
                                ui.label("X:");
                                ui.add(
                                    egui::DragValue::new(&mut self.location.x)
                                        .speed(1.0)
                                        .range(0..=i32::MAX),
                                );
                                ui.label("Y:");
                                ui.add(
                                    egui::DragValue::new(&mut self.location.y)
                                        .speed(1.0)
                                        .range(0..=i32::MAX),
                                );
                            });
                        });
                        ui.end_row();
                    });
            });
        });
    }
}
