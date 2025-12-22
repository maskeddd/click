use eframe::egui;
use eframe::egui::{Vec2, Visuals};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::{runtime::Runtime, sync::mpsc::Sender, task::JoinHandle};
use tracing::error;

use crate::{
    InputHandler,
    input::{ClickAction, MouseButton},
    interval::{IntervalMode, TimeInterval},
};

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct ClickApp {
    interval_mode: IntervalMode,
    mouse_button: MouseButton,
    click_type: ClickAction,

    time_interval: TimeInterval,
    cps: u16,

    #[serde(skip)]
    #[serde(default)]
    clicker: ClickerState,

    #[serde(skip)]
    render_stage: Stage,
}

#[derive(Clone, Copy)]
enum Stage {
    PreRender(isize),
    FirstRender(Vec2),
    FirstResize(Vec2),
    Initialized(Vec2),
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
            mouse_button: MouseButton::Left,
            click_type: ClickAction::Single,
            cps: 20,
            time_interval: TimeInterval::default(),
            clicker: ClickerState::default(),
            render_stage: Stage::PreRender(2),
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
            IntervalMode::CPS => {
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
        let interval_duration = self.calculate_interval();
        let mouse_button = self.mouse_button;
        let click_action = self.click_type;

        let handle = self.clicker.runtime.spawn(async move {
            let mut interval = tokio::time::interval(interval_duration);
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            interval.tick().await;

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        match input_handler.lock() {
                            Ok(mut handler) => {
                                if let Err(e) = handler.click(mouse_button, click_action) {
                                    error!("Click failed: {}", e);
                                    break;
                                }
                            }
                            Err(e) => {
                                error!("Failed to acquire lock: {}", e);
                                break;
                            }
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

    // ======= UI Drawing helpers =======

    fn show_top_menu(&self, ctx: &egui::Context, ui: &mut egui::Ui) {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Quit").clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });
            ui.add_space(16.0);
        });
    }

    fn show_interval_settings(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("interval_grid")
            .num_columns(2)
            .spacing([10.0, 4.0])
            .show(ui, |ui| {
                ui.radio_value(&mut self.interval_mode, IntervalMode::Time, "Time:");
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
                ui.end_row();

                ui.radio_value(&mut self.interval_mode, IntervalMode::CPS, "Target CPS:");
                ui.add(
                    egui::DragValue::new(&mut self.cps)
                        .speed(0.1)
                        .range(0..=1000)
                        .max_decimals(0),
                );
                ui.end_row();
            });
    }

    fn show_input_settings(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("input_grid")
            .num_columns(2)
            .spacing([10.0, 4.0])
            .show(ui, |ui| {
                ui.label("Mouse Button:");
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

                ui.label("Click Type:");
                egui::ComboBox::from_id_salt("click_type_combo")
                    .selected_text(self.click_type.to_string())
                    .show_ui(ui, |ui| {
                        for variant in ClickAction::all() {
                            ui.selectable_value(&mut self.click_type, variant, variant.to_string());
                        }
                    });
                ui.end_row();
            });
    }

    fn show_control_buttons(&mut self, ui: &mut egui::Ui) {
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
    }

    fn render_ui(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            self.show_top_menu(ctx, ui);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.show_interval_settings(ui);

            ui.add(egui::Separator::default().spacing(18.0));

            self.show_input_settings(ui);

            ui.add(egui::Separator::default().spacing(18.0));

            self.show_control_buttons(ui);

            egui::warn_if_debug_build(ui);
        });
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
        match self.render_stage {
            Stage::PreRender(mut count) => {
                self.render_ui(ctx);
                count -= 1;
                if count > 0 {
                    self.render_stage = Stage::PreRender(count);
                } else {
                    self.render_stage = Stage::FirstRender(ctx.used_size());
                }
            }
            Stage::FirstRender(size) => {
                self.render_ui(ctx);
                self.render_stage = Stage::FirstResize(size);
            }
            Stage::FirstResize(size) => {
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(size));
                self.render_stage = Stage::Initialized(size);
            }
            Stage::Initialized(_size) => {
                self.render_ui(ctx);
            }
        }
    }
}
