use eframe::egui;
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::{runtime::Runtime, sync::mpsc::Sender, task::JoinHandle};
use tracing::error;

use crate::{
    InputHandler,
    input::{ClickAction, MouseButton},
    interval::{IntervalMode, Jitter, TimeInterval},
};

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct ClickApp {
    interval_mode: IntervalMode,
    time_interval: TimeInterval,
    cps: u16,
    jitter: u16,
    use_jitter: bool,

    mouse_button: MouseButton,
    click_type: ClickAction,

    #[serde(skip)]
    #[serde(default)]
    clicker: ClickerState,
}

struct ClickerState {
    stop_sender: Option<Sender<()>>,
    task_handle: Option<JoinHandle<()>>,
    runtime: Arc<Runtime>,
    input_handler: Option<Arc<Mutex<InputHandler>>>,
    is_running: bool,
}

impl Default for ClickerState {
    fn default() -> Self {
        let input_handler = InputHandler::new()
            .map(|c| Arc::new(Mutex::new(c)))
            .map_err(|e| error!("Failed to create clicker: {}", e))
            .ok();

        let tokio_runtime = Arc::new(Runtime::new().expect("Failed to create tokio runtime"));

        Self {
            stop_sender: None,
            task_handle: None,
            runtime: tokio_runtime,
            is_running: false,
            input_handler,
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

    fn calculate_interval(&self) -> Duration {
        match self.interval_mode {
            IntervalMode::Time => self.time_interval.to_duration(),
            IntervalMode::Cps => {
                let cps = self.cps.max(1) as u64;
                Duration::from_secs_f64(1.0 / cps as f64)
            }
        }
    }

    fn start_clicker(&mut self) {
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

        let (stop_tx, mut stop_rx) = tokio::sync::mpsc::channel::<()>(1);
        let base_interval = self.calculate_interval();
        let mouse_button = self.mouse_button;
        let click_action = self.click_type;
        let use_jitter = self.use_jitter;
        let jitter = self.jitter;

        let mut human_jitter = Jitter::new();

        let handle = self.clicker.runtime.spawn(async move {
            // Create initial interval with base duration
            let mut interval = tokio::time::interval(base_interval);
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            // Skip the first immediate tick
            interval.tick().await;

            loop {
                // Compute humanized jittered interval
                let current_interval = if use_jitter && jitter > 0 {
                    human_jitter.next(base_interval, jitter)
                } else {
                    base_interval
                };

                // Recreate interval if the duration changed significantly
                // (only needed when jitter is in use)
                if use_jitter && jitter > 0 {
                    interval = tokio::time::interval(current_interval);
                    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
                    interval.tick().await;
                }

                tokio::select! {
                    _ = interval.tick() => {
                        let click_result = input_handler
                            .lock()
                            .map_err(|e| format!("Lock error: {e}"))
                            .and_then(|mut handler| {
                                handler
                                    .click(mouse_button, click_action)
                                    .map_err(|e| format!("Click failed: {e}"))
                            });

                        if let Err(e) = click_result {
                            error!("{e}");
                            break;
                        }
                    }
                    _ = stop_rx.recv() => {
                        break;
                    }
                }
            }
        });

        self.clicker.stop_sender = Some(stop_tx);
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
        egui::TopBottomPanel::bottom("bottom_panel")
            .frame(
                egui::Frame::default()
                    .fill(egui::Color32::from_rgb(26, 26, 26))
                    .inner_margin(egui::Margin {
                        left: 12,
                        right: 12,
                        top: 12,
                        bottom: 12,
                    }),
            )
            .show(ctx, |ui| {
                ui.columns_const(|[left, right]| {
                    left.add_enabled_ui(!self.clicker.is_running, |ui| {
                        if ui
                            .add_sized([0.0, 36.0], egui::Button::new("Start"))
                            .clicked()
                        {
                            self.start_clicker();
                        }
                    });

                    right.add_enabled_ui(self.clicker.is_running, |ui| {
                        if ui
                            .add_sized([0.0, 36.0], egui::Button::new("Stop"))
                            .clicked()
                        {
                            self.stop_clicker();
                        }
                    });
                });

                egui::warn_if_debug_build(ui);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
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
                                    .range(0..=23)
                                    .max_decimals(0),
                            );

                            ui.label("M:");
                            ui.add(
                                egui::DragValue::new(&mut self.time_interval.minutes)
                                    .speed(0.1)
                                    .range(0..=59)
                                    .max_decimals(0),
                            );

                            ui.label("S:");
                            ui.add(
                                egui::DragValue::new(&mut self.time_interval.seconds)
                                    .speed(0.1)
                                    .range(0..=59)
                                    .max_decimals(0),
                            );

                            ui.label("MS:");
                            ui.add(
                                egui::DragValue::new(&mut self.time_interval.milliseconds)
                                    .speed(1.0)
                                    .range(0..=999)
                                    .max_decimals(0),
                            );
                        });
                    });
                    ui.end_row();

                    ui.radio_value(&mut self.interval_mode, IntervalMode::Cps, "Target CPS:");
                    ui.add_enabled_ui(self.interval_mode == IntervalMode::Cps, |ui| {
                        ui.add(
                            egui::DragValue::new(&mut self.cps)
                                .speed(0.1)
                                .range(0..=1000)
                                .max_decimals(0),
                        );
                    });
                    ui.end_row();

                    ui.checkbox(&mut self.use_jitter, "Random delay: ");
                    ui.add_enabled_ui(self.use_jitter, |ui| {
                        ui.add(
                            egui::DragValue::new(&mut self.jitter)
                                .speed(0.1)
                                .range(0..=1000)
                                .max_decimals(0)
                                .prefix("± ")
                                .suffix(" ms"),
                        );
                    });
                    ui.end_row();
                });

            ui.add(egui::Separator::default().spacing(18.0));

            ui.heading("Behavior");
            ui.add_space(6.0);

            egui::Grid::new("input_grid")
                .num_columns(2)
                .spacing([10.0, 4.0])
                .show(ui, |ui| {
                    ui.label("Mouse button:");
                    egui::ComboBox::from_id_salt("mouse_button_combo")
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
                    egui::ComboBox::from_id_salt("click_type_combo")
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
                });
        });
    }
}
