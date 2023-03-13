use std::vec;

use eframe::emath;
use egui::{Painter, Rect, Pos2, Stroke, Color32, plot::{Plot, Line, PlotPoints}};

use crate::physics::{Simulation, MaxwellType};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    #[serde(skip)]
    simulation: Simulation,
    #[serde(skip)]
    paused: bool,
    #[serde(skip)]
    points: Vec<(f64, f64)>,
    #[serde(skip)]
    time: f64,


    temperature: f32,
    balls_n: u16,
    radius: f32,
    filter_height: f32,
    filter_temperature: f32,
    filter_constant: f32,
    filter_type: MaxwellType,
    wall_width: f32,
    collisions: bool,

    measure_time: f64,
    current_frames: u32,
    current_sum: f64

}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            collisions: true,
            time: 0.0,
            points: vec![],
            temperature: 0.5,
            balls_n: 40,
            radius: 0.01,
            filter_type: MaxwellType::Tennis,
            filter_height: 0.3,
            filter_temperature: 1.0,
            simulation:  Simulation::new(),
            wall_width: 0.05,
            filter_constant: 0.1,
            paused: false,

            measure_time: 0.3,
            current_frames: 0,
            current_sum: 0.0
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
            app.simulation.random_initiation(app.balls_n, app.temperature, app.radius, app.filter_height, app.filter_type, app.collisions, app.wall_width);
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
        let Self {temperature, simulation, balls_n: n_balls, radius, filter_height, filter_type, points, time, ..} = self;

        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
        egui::TopBottomPanel::top("panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        _frame.close();
                    }
                });
            });
        });
        
        let mut density: f64 = 0.0;

        egui::Window::new("Parameters").show(ctx, |ui| {

            ui.checkbox(&mut self.paused, "Paused");
            if ui.input(|i| i.key_pressed(egui::Key::Space)) {
                self.paused = !self.paused;
            }
            ui.checkbox(&mut self.collisions, "Collisions");
            ui.end_row();
            ui.add(egui::Slider::new(&mut self.measure_time, 0.01..=1.0).text("Measuring time"));
            ui.add(egui::Slider::new(temperature, 0.0..=3.0).text("Temperature"));
            ui.add(egui::Slider::new(n_balls, 0..=1000).text("Balls number"));
            ui.add(egui::Slider::new(radius, 0.0..=0.03).text("Ball radius"));
            ui.add(egui::Slider::new(filter_height, 0.0..=1.0).text("Filter height"));
            ui.add(egui::Slider::new(&mut self.wall_width, 0.0..=0.1).text("Wall width"));

            egui::ComboBox::from_label("Filter type:")
                .selected_text(match filter_type {
                    MaxwellType::Diode => "Diode",
                    MaxwellType::Temperature {..} => "Temperature",
                    MaxwellType::Tennis => "Tennis",
                    MaxwellType::Empty => "Empty",
                    MaxwellType::PhaseConserving {..} => "Phase conserving",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(filter_type, MaxwellType::Diode, "Diode");
                    ui.selectable_value(filter_type, MaxwellType::Temperature { t: self.filter_temperature}, "Temperature");
                    ui.selectable_value(filter_type, MaxwellType::Tennis, "Tennis");
                    ui.selectable_value(filter_type, MaxwellType::PhaseConserving { c: self.filter_constant }, "Phase conserving");
                    ui.selectable_value(filter_type, MaxwellType::Empty, "Empty");
                }
            );

            if let MaxwellType::Temperature { t } = filter_type{
                ui.add(egui::Slider::new(t, 0.0..=5.0).text("Filter temperature"));
            }
            else if let MaxwellType::PhaseConserving { c } = filter_type{
                ui.add(egui::Slider::new(c, 0.0..=1.0).text("Filter constant"));
            }
            

            if ui.button("Regenerate").clicked() {
                simulation.random_initiation(*n_balls, *temperature, *radius, *filter_height, *filter_type, self.collisions, self.wall_width);
                points.clear();
                *time = 0.0;
            }

            let (left_count, right_symbol) = simulation.structure.count_balls(&simulation);
            ui.label(format!("\nLeft side: {} balls,\nRight side: {} balls", left_count, right_symbol));
            density = (left_count as f64)/((left_count + right_symbol) as f64)*100.0;
            ui.label(format!("Left chamber density: {:.1} %", density));

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.label("© ");
                    ui.hyperlink_to(
                        "sitandr",
                        "https://github.com/sitandr",
                    );
                    ui.label(", 2023");
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            if !self.paused{
                self.time += 0.01;
                simulation.step(0.01);
                ui.ctx().request_repaint();

                self.current_frames += 1;
                self.current_sum += density;

                if self.time % (0.01 * self.current_frames as f64) >= self.measure_time{
                    //if points.last().map_or(true, |p| density != p.1){
                    points.push((self.time, self.current_sum/self.current_frames as f64));
                    self.current_sum = 0.0;
                    self.current_frames = 0;
                }
            }
            let mut rect = ui.available_rect_before_wrap();
            if rect.height() > rect.width(){
                rect.set_height(rect.width())
            }
            else{
                rect.set_width(rect.height())
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
            simulation.paint(&painter, to_screen, ui.visuals().dark_mode);
            painter.rect_stroke(rect, 1.0, Stroke::new(1.0, Color32::from_gray(16)));
            // Make sure we allocate what we used (everything)
            ui.expand_to_include_rect(painter.clip_rect());
            egui::warn_if_debug_build(ui);
        });

        if true {
            egui::Window::new("Left density/time").show(ctx, |ui| {
                Plot::new("data").include_y(50.0).include_x(0.0).auto_bounds_y().auto_bounds_x().show(ui, |plot_ui| plot_ui.line(Line::new(
                    points.iter().map(|&(x, p)| {
                        [x, p]}).collect::<PlotPoints>())));
            });
        }
    }
}
