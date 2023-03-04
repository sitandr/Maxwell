use eframe::emath;
use egui::{Painter, Rect, Pos2};

use crate::physics::{Simulation, BoxStructure, MaxwellType};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    #[serde(skip)]
    simulation: Simulation,
    #[serde(skip)]
    paused: bool,

    temperature: f32,
    balls_n: u16,
    radius: f32
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            temperature: 1.0,
            balls_n: 30,
            radius: 0.01,
            simulation:  Simulation::new(BoxStructure::new(MaxwellType::Tennis)),
            paused: false
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            let mut app: Self =  eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
            app.simulation.random_initiation(app.balls_n, app.temperature, app.radius);
            app
        }
        else{
            Default::default()
        }
    }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let Self {temperature, simulation, balls_n: n_balls, radius, ..} = self;

        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        _frame.close();
                    }
                });
            });
        });

        egui::SidePanel::left("side_panel").show(ctx, |ui| {

            ui.checkbox(&mut self.paused, "Paused");
            ui.add(egui::Slider::new(temperature, 0.0..=5.0).text("Temperature"));
            ui.add(egui::Slider::new(n_balls, 0..=1000).text("Balls number"));
            ui.add(egui::Slider::new(radius, 0.0..=0.1).text("Ball radius"));

            if ui.button("Regenerate").clicked() {
                simulation.random_initiation(*n_balls, *temperature, *radius);
            }

            let (left, right) = simulation.structure.count_balls(&simulation);
            ui.label(format!("\nLeft side: {} balls,\nRight side: {} balls", left, right));
            ui.label(format!("Left chamber density: {:.1}", (left as f32)/(*n_balls as f32)*100.0));

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.label("powered by ");
                    ui.hyperlink_to("egui", "https://github.com/emilk/egui");
                    ui.label(" and ");
                    ui.hyperlink_to(
                        "eframe",
                        "https://github.com/emilk/egui/tree/master/crates/eframe",
                    );
                    ui.label(".");
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            if !self.paused{
                simulation.step(0.01);
                ui.ctx().request_repaint();
            }
            let mut rect = ui.available_rect_before_wrap();
            if rect.height() > rect.width(){
                rect.set_width(rect.height())
            }
            else{
                rect.set_height(rect.width())
            }
            
            let painter = Painter::new(
                ui.ctx().clone(),
                ui.layer_id(),
                rect,
            );
            let rect = painter.clip_rect();
            let to_screen = emath::RectTransform::from_to(
                Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
                rect,
            );
            simulation.paint(&painter, to_screen);
            // Make sure we allocate what we used (everything)
            ui.expand_to_include_rect(painter.clip_rect());
            egui::warn_if_debug_build(ui);
        });

        if false {
            egui::Window::new("Window").show(ctx, |ui| {
                ui.label("Windows can be moved by dragging them.");
                ui.label("They are automatically sized based on contents.");
                ui.label("You can turn on resizing and scrolling if you like.");
                ui.label("You would normally choose either panels OR windows.");
            });
        }
    }
}
