use power_manager::{read_batteries, Battery};
use std::time::{Duration, Instant};
struct MyApp {
    batteries: Vec<Battery>,
    error: Option<String>,
    last_updated: Instant,
}

impl MyApp {
    fn new() -> Self {
        match read_batteries() {
            Ok(batteries) => Self {
                batteries,
                error: None,
                last_updated: Instant::now(),
            },
            Err(e) => Self {
                batteries: Vec::new(),
                error: Some(format!("{:?}", e)),
                last_updated: Instant::now(),
            },
        }
    }

    // Re-reads battery data and updates
    fn refresh(&mut self) {
        match read_batteries() {
            Ok(batteries) => {
                self.batteries = batteries;
                self.error = None; 
            }
            Err(e) => {
                self.error = Some(format!("{:?}", e));
            }
        }
        self.last_updated = Instant::now();
    }
}

impl eframe::App for MyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
    ui.set_visuals(egui::Visuals {
        panel_fill: egui::Color32::from_rgb(24, 26, 32),
        override_text_color: Some(egui::Color32::from_rgb(220, 220, 220)),
        ..egui::Visuals::dark()
    });

    // Auto refresh every 5 seconds without blocking the UI thread
    if self.last_updated.elapsed() > Duration::from_secs(5) {
        self.refresh();
    }

    egui::CentralPanel::default().show(ui, |ui| {
        ui.heading("Is your battery fu**ed like mine?");
        if ui.button("Refresh now").clicked() {
            self.refresh();
        }
        ui.separator();

        if let Some(err) = &self.error {
            ui.colored_label(egui::Color32::RED, err);
        }

        for battery in &self.batteries {
            ui.group(|ui| {
                ui.label(format!("{}", battery.name));
                ui.label(format!("Status: {:?}", battery.status));
                ui.add(
                    egui::ProgressBar::new(battery.capacity as f32 / 100.0)
                        .text(format!("Charge: {}%", battery.capacity)),
                );
                
                match health_percent(battery) {
                    // Showing battery health status with color of navigation bar
                    Some(health) => {
                        let health_color = match health {
                            h if h >= 80.0 => egui::Color32::from_rgb(80, 200, 120), // green
                            h if h >= 50.0 => egui::Color32::from_rgb(230, 180, 60), // amber
                            _ => egui::Color32::from_rgb(220, 70, 70), // red
                        };

                        ui.add(
                            egui::ProgressBar::new(health / 100.0)
                                .fill(health_color)
                                .text(format!("Health: {health:.1}%")),
                        );
                    }
                    None => {
                        ui.label("Health: unknown");
                    }
                }

                match battery.cycle_count {
                    Some(n) => ui.label(format!("Cycles: {n}")),
                    None => ui.label("Cycles: unknown"),
                };
            });
        }
    });

    ui.request_repaint_after(Duration::from_secs(1));
}

}

// Battery health computing using this formual => charge_full / charge_full_design * 100
// charge_full and charge_full_design are all located at /sys/class/power_supply/your_battery_name
fn health_percent(battery: &Battery) -> Option<f32> {
    let full = battery.charge_full?;
    let design = battery.charge_full_design?;
    if design == 0 {
        return None;
    }
    Some(full as f32 / design as f32 * 100.0)
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport : egui::ViewportBuilder::default()
        .with_inner_size([874.0 , 180.0]) ,
        ..Default::default()
    };
    eframe::run_native(
        "Battery Manager",
        options,
        Box::new(|_cc| Ok(Box::new(MyApp::new()))),
    )
}