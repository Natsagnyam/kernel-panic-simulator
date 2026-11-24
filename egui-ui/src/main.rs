// Author: Natska Natsagnaym NAMKHAI
// License: Open Source (MIT OR GPL-2.0-only)

mod ml_detector;

use eframe::egui::{self, Color32, Margin, Pos2, RichText, Rounding, Stroke, Vec2};
use ml_detector::MlAnomalyDetector;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::process::Command;
use std::time::{Duration, Instant};

const AUTHOR_NAME: &str = "Natska Natsagnaym NAMKHAI";
const LICENSE_LABEL: &str = "Open Source (MIT OR GPL-2.0-only)";
const TRIGGER_PATH: &str = "/sys/kernel/panic_lab/trigger";
const METRICS_PATH: &str = "/sys/kernel/panic_lab/metrics";
const EXPECTED_RACE_TOTAL: u64 = 1_000_000;

fn send_kernel_trigger(cmd: &str) -> Result<String, String> {
    let mut file = OpenOptions::new()
        .write(true)
        .open(TRIGGER_PATH)
        .map_err(|e| format!("open {}: {}", TRIGGER_PATH, e))?;

    file.write_all(cmd.as_bytes())
        .map_err(|e| format!("write {}: {}", TRIGGER_PATH, e))?;

    Ok(format!("Wrote '{}' to {}", cmd, TRIGGER_PATH))
}

fn read_sysfs_file(path: &str) -> Result<String, String> {
    fs::read_to_string(path)
        .map(|s| s.trim_end().to_string())
        .map_err(|e| format!("read {}: {}", path, e))
}

fn read_dmesg_tail(filter_panic_lab: bool, max_lines: usize) -> String {
    let output = Command::new("dmesg")
        .args(["-T", "--level=emerg,alert,crit,err,warn,info"])
        .output();

    match output {
        Ok(out) => {
            if !out.status.success() {
                let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
                if stderr.contains("Operation not permitted") {
                    return format!(
                        "dmesg blocked ({}). Run with sudo/pkexec or set: sudo sysctl -w kernel.dmesg_restrict=0",
                        stderr
                    );
                }
                return format!("dmesg exited with {}: {}", out.status, stderr);
            }

            let text = String::from_utf8_lossy(&out.stdout);
            let mut lines: Vec<&str> = text
                .lines()
                .filter(|line| {
                    !filter_panic_lab
                        || line.contains("panic_lab")
                        || line.contains("race worker")
                        || line.contains("Call Trace")
                        || line.contains("BUG:")
                        || line.contains("Oops")
                })
                .collect();

            if lines.len() > max_lines {
                let start = lines.len() - max_lines;
                lines = lines[start..].to_vec();
            }

            if lines.is_empty() {
                if filter_panic_lab {
                    "No matching panic_lab / race worker / fault lines in dmesg yet.".to_string()
                } else {
                    "dmesg output is empty.".to_string()
                }
            } else {
                lines.join("\n")
            }
        }
        Err(e) => format!("failed to run dmesg: {}", e),
    }
}

fn parse_latest_race_worker(dmesg_text: &str) -> Option<String> {
    for line in dmesg_text.lines().rev() {
        if let Some(pos) = line.find("race worker ") {
            let summary = &line[pos..];
            if let Some(c_pos) = summary.find("counter=") {
                let num_str: String = summary[c_pos + 8..]
                    .chars()
                    .take_while(|c| c.is_ascii_digit())
                    .collect();
                if let Ok(actual) = num_str.parse::<u64>() {
                    let lost = EXPECTED_RACE_TOTAL.saturating_sub(actual);
                    return Some(format!(
                        "{} (expected={}, lost_updates={})",
                        summary, EXPECTED_RACE_TOTAL, lost
                    ));
                }
            }
            return Some(summary.to_string());
        }
    }
    None
}

struct Palette {
    bg: Color32,
    card_bg: Color32,
    card_border: Color32,
    text_primary: Color32,
    text_muted: Color32,
    btn_bg: Color32,
    btn_hover: Color32,
    btn_active: Color32,
    btn_border: Color32,
    danger_bg: Color32,
    danger_hover: Color32,
    danger_border: Color32,
    ok_color: Color32,
    err_color: Color32,
    log_bg: Color32,
}

impl Palette {
    fn light() -> Self {
        Self {
            bg: Color32::from_rgb(241, 245, 249),
            card_bg: Color32::WHITE,
            card_border: Color32::from_rgb(203, 213, 225),
            text_primary: Color32::from_rgb(15, 23, 42),
            text_muted: Color32::from_rgb(71, 85, 105),
            btn_bg: Color32::from_rgb(226, 232, 240),
            btn_hover: Color32::from_rgb(203, 213, 225),
            btn_active: Color32::from_rgb(37, 99, 235),
            btn_border: Color32::from_rgb(100, 116, 139),
            danger_bg: Color32::from_rgb(254, 226, 226),
            danger_hover: Color32::from_rgb(254, 202, 202),
            danger_border: Color32::from_rgb(220, 38, 38),
            ok_color: Color32::from_rgb(21, 128, 61),
            err_color: Color32::from_rgb(185, 28, 28),
            log_bg: Color32::from_rgb(248, 250, 252),
        }
    }

    fn dark() -> Self {
        Self {
            bg: Color32::from_rgb(15, 23, 42),
            card_bg: Color32::from_rgb(30, 41, 59),
            card_border: Color32::from_rgb(71, 85, 105),
            text_primary: Color32::from_rgb(248, 250, 252),
            text_muted: Color32::from_rgb(148, 163, 184),
            btn_bg: Color32::from_rgb(51, 65, 85),
            btn_hover: Color32::from_rgb(71, 85, 105),
            btn_active: Color32::from_rgb(59, 130, 246),
            btn_border: Color32::from_rgb(148, 163, 184),
            danger_bg: Color32::from_rgb(127, 29, 29),
            danger_hover: Color32::from_rgb(153, 27, 27),
            danger_border: Color32::from_rgb(248, 113, 113),
            ok_color: Color32::from_rgb(74, 222, 128),
            err_color: Color32::from_rgb(248, 113, 113),
            log_bg: Color32::from_rgb(15, 23, 42),
        }
    }
}

fn apply_theme(ctx: &egui::Context, light_mode: bool) -> Palette {
    let p = if light_mode {
        Palette::light()
    } else {
        Palette::dark()
    };

    let mut visuals = if light_mode {
        egui::Visuals::light()
    } else {
        egui::Visuals::dark()
    };

    visuals.panel_fill = p.bg;
    visuals.window_fill = p.bg;
    visuals.override_text_color = Some(p.text_primary);
    visuals.widgets.inactive.bg_fill = p.btn_bg;
    visuals.widgets.inactive.weak_bg_fill = p.btn_bg;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.2, p.btn_border);
    visuals.widgets.inactive.rounding = Rounding::same(8.0);

    visuals.widgets.hovered.bg_fill = p.btn_hover;
    visuals.widgets.hovered.weak_bg_fill = p.btn_hover;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.5, p.text_primary);
    visuals.widgets.hovered.rounding = Rounding::same(8.0);

    visuals.widgets.active.bg_fill = p.btn_active;
    visuals.widgets.active.weak_bg_fill = p.btn_active;
    visuals.widgets.active.bg_stroke = Stroke::new(1.5, p.btn_active);
    visuals.widgets.active.rounding = Rounding::same(8.0);

    let mut style = (*ctx.style()).clone();
    style.visuals = visuals;
    style.spacing.button_padding = Vec2::new(14.0, 10.0);
    style.spacing.item_spacing = Vec2::new(10.0, 8.0);
    ctx.set_style(style);

    p
}

fn card_frame(p: &Palette) -> egui::Frame {
    egui::Frame::none()
        .fill(p.card_bg)
        .stroke(Stroke::new(1.0, p.card_border))
        .rounding(Rounding::same(10.0))
        .inner_margin(Margin::same(14.0))
}

fn fault_button(ui: &mut egui::Ui, label: &str, subtitle: &str, severe: bool, p: &Palette) -> bool {
    let base_fill = if severe { p.danger_bg } else { p.btn_bg };
    let hover_fill = if severe { p.danger_hover } else { p.btn_hover };
    let border = if severe { p.danger_border } else { p.btn_border };

    let desired_size = Vec2::new(280.0, 58.0);
    let (rect, resp) = ui.allocate_exact_size(desired_size, egui::Sense::click());

    let fill = if resp.hovered() {
        hover_fill
    } else {
        base_fill
    };
    let stroke_width = if resp.hovered() { 2.0 } else { 1.4 };

    ui.painter().rect_filled(rect, Rounding::same(8.0), fill);
    ui.painter()
        .rect_stroke(rect, Rounding::same(8.0), Stroke::new(stroke_width, border));

    let title_galley = ui.painter().layout_no_wrap(
        label.to_string(),
        egui::FontId::proportional(14.0),
        p.text_primary,
    );
    let sub_galley = ui.painter().layout_no_wrap(
        subtitle.to_string(),
        egui::FontId::proportional(11.5),
        p.text_muted,
    );

    let title_pos = Pos2::new(rect.left() + 12.0, rect.top() + 9.0);
    let sub_pos = Pos2::new(rect.left() + 12.0, rect.top() + 31.0);

    ui.painter().galley(title_pos, title_galley, p.text_primary);
    ui.painter().galley(sub_pos, sub_galley, p.text_muted);

    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    resp.clicked()
}

struct PanicLabApp {
    detector: MlAnomalyDetector,
    light_mode: bool,
    applied_initial_maximize: bool,
    write_result: String,
    read_result: String,
    metrics_result: String,
    dmesg_tail: String,
    latest_race_summary: String,
    filter_panic_lab_dmesg: bool,
    dmesg_line_limit: usize,
    poll_until: Option<Instant>,
    last_poll: Instant,
}

impl Default for PanicLabApp {
    fn default() -> Self {
        let initial_read = read_sysfs_file(TRIGGER_PATH)
            .unwrap_or_else(|err| format!("[sysfs unavailable] {}", err));
        let initial_metrics = read_sysfs_file(METRICS_PATH)
            .unwrap_or_else(|err| format!("[metrics unavailable] {}", err));
        let filter_default = true;
        let limit_default = 60;
        let initial_dmesg = read_dmesg_tail(filter_default, limit_default);
        let latest_race_summary = parse_latest_race_worker(&initial_dmesg)
            .unwrap_or_else(|| "No race worker completion logged yet.".to_string());

        Self {
            detector: MlAnomalyDetector::default(),
            light_mode: true,
            applied_initial_maximize: false,
            write_result: "No trigger written yet.".to_string(),
            read_result: initial_read,
            metrics_result: initial_metrics,
            dmesg_tail: initial_dmesg,
            latest_race_summary,
            filter_panic_lab_dmesg: filter_default,
            dmesg_line_limit: limit_default,
            poll_until: None,
            last_poll: Instant::now(),
        }
    }
}

impl PanicLabApp {
    fn dispatch_fault(&mut self, button_label: &str, cmd: &str) {
        self.detector.record_dispatch(cmd);

        match send_kernel_trigger(cmd) {
            Ok(msg) => {
                self.write_result = format!("OK ({}): {}", button_label, msg);
            }
            Err(err) => {
                self.write_result = format!("ERR ({}): {}", button_label, err);
            }
        }

        if cmd == "race" {
            self.poll_until = Some(Instant::now() + Duration::from_secs(2));
        }

        self.refresh_all();
    }

    fn refresh_all(&mut self) {
        self.read_result = read_sysfs_file(TRIGGER_PATH)
            .unwrap_or_else(|err| format!("ERR read: {}", err));
        self.metrics_result = read_sysfs_file(METRICS_PATH)
            .unwrap_or_else(|err| format!("ERR metrics: {}", err));
        self.refresh_dmesg();
    }

    fn refresh_dmesg(&mut self) {
        self.dmesg_tail = read_dmesg_tail(self.filter_panic_lab_dmesg, self.dmesg_line_limit);
        if let Some(summary) = parse_latest_race_worker(&self.dmesg_tail) {
            self.latest_race_summary = summary;
        }
        self.last_poll = Instant::now();
    }
}

impl eframe::App for PanicLabApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.applied_initial_maximize {
            self.applied_initial_maximize = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(true));
        }

        if let Some(deadline) = self.poll_until {
            if Instant::now() < deadline {
                if self.last_poll.elapsed() >= Duration::from_millis(150) {
                    self.refresh_dmesg();
                }
                ctx.request_repaint_after(Duration::from_millis(150));
            } else {
                self.poll_until = None;
                self.refresh_dmesg();
            }
        }

        let p = apply_theme(ctx, self.light_mode);

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                card_frame(&p).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label(
                                RichText::new("Kernel Panic Detector & Fault Lab")
                                    .size(22.0)
                                    .strong()
                                    .color(p.text_primary),
                            );
                            ui.label(
                                RichText::new(format!(
                                    "Author: {}  |  Licence: {}",
                                    AUTHOR_NAME, LICENSE_LABEL
                                ))
                                .size(12.5)
                                .color(p.text_muted),
                            );
                        });

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let theme_label = if self.light_mode {
                                "Switch to Dark Mode"
                            } else {
                                "Switch to Light Mode"
                            };
                            if ui.button(theme_label).clicked() {
                                self.light_mode = !self.light_mode;
                            }
                            if ui.button("Refresh All").clicked() {
                                self.refresh_all();
                            }
                            if ui.button("Reset ML Telemetry").clicked() {
                                self.detector.reset();
                            }
                            ui.checkbox(&mut self.detector.enabled, "Enable ML Anomaly Detector");
                        });
                    });

                    if self.write_result.contains("No such file or directory")
                        || self.read_result.contains("No such file or directory")
                    {
                        ui.add_space(8.0);
                        ui.colored_label(
                            p.err_color,
                            RichText::new(
                                "Module not loaded: run `make -C kernel && sudo insmod kernel/panic_lab.ko` first.",
                            )
                            .strong(),
                        );
                    } else if self.write_result.contains("Permission denied")
                        || self.read_result.contains("Permission denied")
                    {
                        ui.add_space(8.0);
                        ui.colored_label(
                            p.err_color,
                            RichText::new(
                                "Permission denied: run with `sudo ./egui-ui/target/debug/panic-detector-egui`.",
                            )
                            .strong(),
                        );
                    }
                });

                ui.add_space(10.0);

                card_frame(&p).show(ui, |ui| {
                    ui.label(
                        RichText::new("Let me panic KERNEL")
                            .size(16.0)
                            .strong()
                            .color(p.text_primary),
                    );
                    ui.label(
                        RichText::new("Select a fault test below to write directly to /sys/kernel/panic_lab/trigger")
                            .size(12.0)
                            .color(p.text_muted),
                    );
                    ui.add_space(8.0);

                    ui.horizontal_wrapped(|ui| {
                        if fault_button(
                            ui,
                            "Let me try a memory leak",
                            "kmalloc(4096, GFP_KERNEL) without kfree",
                            false,
                            &p,
                        ) {
                            self.dispatch_fault("Let me try a memory leak", "memleak");
                        }

                        if fault_button(
                            ui,
                            "Let me try a race condition",
                            "Two unsynchronized kthreads hammering shared state",
                            false,
                            &p,
                        ) {
                            self.dispatch_fault("Let me try a race condition", "race");
                        }

                        if fault_button(
                            ui,
                            "Let me try a number / 0",
                            "Trigger integer divide-by-zero trap (#DE)",
                            true,
                            &p,
                        ) {
                            self.dispatch_fault("Let me try a number / 0", "divzero");
                        }

                        if fault_button(
                            ui,
                            "Let me try an out-of-bounds C pointer",
                            "Dereference invalid address (0xdeadbeef)",
                            true,
                            &p,
                        ) {
                            self.dispatch_fault("Let me try an out-of-bounds C pointer", "badptr");
                        }
                    });

                    ui.add_space(10.0);
                    ui.label(RichText::new("Latest Race Worker Telemetry (from dmesg):").strong().size(12.5));
                    ui.monospace(&self.latest_race_summary);
                });

                ui.add_space(10.0);

                card_frame(&p).show(ui, |ui| {
                    match self.detector.score() {
                        Some(assessment) => {
                            let badge_color = if assessment.composite_score >= 0.80 {
                                p.err_color
                            } else if assessment.composite_score >= 0.45 {
                                Color32::from_rgb(217, 119, 6)
                            } else {
                                p.ok_color
                            };

                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new("ML Anomaly Detector: ENABLED")
                                        .strong()
                                        .size(15.0),
                                );
                                ui.colored_label(
                                    badge_color,
                                    RichText::new(format!("[{}]", assessment.verdict)).strong(),
                                );
                            });

                            ui.add_space(6.0);
                            ui.add(
                                egui::ProgressBar::new(assessment.composite_score.clamp(0.0, 1.0))
                                    .text(format!("Composite Anomaly Score: {:.2}", assessment.composite_score)),
                            );

                            ui.add_space(6.0);
                            ui.monospace(format!(
                                "Leaked: {} B ({}x) | leak_score={:.2} | div0_score={:.2} | badptr_score={:.2} | composite={:.2}",
                                self.detector.telemetry.leaked_bytes,
                                self.detector.telemetry.leak_events,
                                assessment.leak_growth_score,
                                assessment.divzero_score,
                                assessment.badptr_score,
                                assessment.composite_score,
                            ));
                        }
                        None => {
                            ui.label(
                                RichText::new("ML Anomaly Detector: DISABLED (raw fault-injection test mode)")
                                    .color(p.text_muted),
                            );
                        }
                    }
                });

                ui.add_space(10.0);

                card_frame(&p).show(ui, |ui| {
                    ui.label(RichText::new("Sysfs & Live Kernel Metrics").size(15.0).strong());
                    ui.add_space(6.0);

                    let write_color = if self.write_result.starts_with("OK") {
                        p.ok_color
                    } else if self.write_result.starts_with("ERR") {
                        p.err_color
                    } else {
                        p.text_primary
                    };

                    ui.label(RichText::new("Write Result:").strong().size(12.5));
                    ui.colored_label(write_color, RichText::new(&self.write_result).monospace());

                    ui.add_space(6.0);
                    ui.label(RichText::new("Read Result (/sys/kernel/panic_lab/trigger):").strong().size(12.5));
                    ui.monospace(&self.read_result);

                    ui.add_space(6.0);
                    ui.label(RichText::new("Live Kernel Metrics (/sys/kernel/panic_lab/metrics):").strong().size(12.5));
                    egui::Frame::none()
                        .fill(p.log_bg)
                        .stroke(Stroke::new(1.0, p.card_border))
                        .rounding(Rounding::same(6.0))
                        .inner_margin(Margin::same(8.0))
                        .show(ui, |ui| {
                            ui.set_min_width(ui.available_width());
                            ui.monospace(&self.metrics_result);
                        });
                });

                ui.add_space(10.0);

                card_frame(&p).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("dmesg Log Tail").size(15.0).strong());
                        if ui
                            .checkbox(&mut self.filter_panic_lab_dmesg, "Filter panic_lab / fault lines")
                            .changed()
                        {
                            self.refresh_dmesg();
                        }
                        ui.label("Lines:");
                        if ui
                            .add(egui::Slider::new(&mut self.dmesg_line_limit, 10..=200))
                            .changed()
                        {
                            self.refresh_dmesg();
                        }
                        if ui.button("Refresh dmesg").clicked() {
                            self.refresh_dmesg();
                        }
                    });

                    ui.add_space(6.0);
                    egui::Frame::none()
                        .fill(p.log_bg)
                        .stroke(Stroke::new(1.0, p.card_border))
                        .rounding(Rounding::same(6.0))
                        .inner_margin(Margin::same(10.0))
                        .show(ui, |ui| {
                            egui::ScrollArea::vertical()
                                .max_height(260.0)
                                .stick_to_bottom(true)
                                .show(ui, |ui| {
                                    ui.set_min_width(ui.available_width());
                                    ui.monospace(&self.dmesg_tail);
                                });
                        });
                });
            });
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_maximized(true),
        ..Default::default()
    };
    eframe::run_native(
        "Kernel Panic Detector",
        options,
        Box::new(|_cc| Ok(Box::new(PanicLabApp::default()))),
    )
}
