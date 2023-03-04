use eframe::emath::RectTransform;
use egui::{Pos2, Color32, Stroke, Rect};
use rand::Rng;
use rand_distr::StandardNormal;

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct Ball{
    x: f32,
    y: f32,
    speed_x: f32,
    speed_y: f32,
    inside_maxwell: bool
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct BoxStructure{
    width: f32,
    height: f32,
    maxwell: Maxwell
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct Simulation{
    pub structure: BoxStructure,
    balls: Vec<Ball>
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub enum MaxwellType{
    Diode,
    Temperature{t: f32},
    Tennis
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct Maxwell{
    filter_type: MaxwellType
}

impl Maxwell{
    fn in_bounds(&self, structure: &BoxStructure, x: f32, y: f32) -> bool{
        let inside_wall = x > structure.width*0.48 && x < structure.width*0.52;
        let accurate_y = true; //y > structure.height*0.45 && y < structure.height*0.55;
        return inside_wall && accurate_y;
    }

    fn refract_ball(&self, ball: &mut Ball){
        match self.filter_type {
            MaxwellType::Diode => {
                if ball.speed_x < 0.0{
                    ball.speed_x = - ball.speed_x;
                }
            },
            MaxwellType::Temperature { t } => {
                if ball.speed_x < 0.0{
                    if ball.speed_x.powf(2.0) < 1.0{
                        ball.speed_x = - ball.speed_x;
                    }
                }
                else{
                    if ball.speed_x.powf(2.0) > t{
                        ball.speed_x = - ball.speed_x;
                    }
                }
            },
            MaxwellType::Tennis => {
                if ball.speed_x * ball.speed_y >= 0.0 && ball.speed_x < ball.speed_y{
                    (ball.speed_x, ball.speed_y) = (ball.speed_y, ball.speed_x)
                }
                else if (ball.speed_x  < 0.0 &&  ball.speed_y > 0.0 && ball.speed_x.abs() > ball.speed_y ) 
                    || (ball.speed_x  > 0.0 &&  ball.speed_y < 0.0 && ball.speed_x < ball.speed_y.abs() ){
                    (ball.speed_x, ball.speed_y) = (-ball.speed_y, -ball.speed_x)
                } 
                else{
                    ball.speed_x = -ball.speed_x;
                }
            },
        }
        
    }

    fn coords(&self, structure: &BoxStructure) -> (Pos2, Pos2){
        (Pos2::new(structure.width*0.48, structure.height*0.45),
        Pos2::new(structure.width*0.52, structure.height*0.55))
    }
}

impl BoxStructure{
    pub fn new(filter_type: MaxwellType) -> Self{
        Self { width: 1.0, height: 1.0, maxwell: Maxwell {filter_type}}
    }

    fn in_bounds(&self, x: f32, y: f32) -> bool{
        let out_of_box = x > self.width
            ||  y > self.height
            ||  x < 0.0
            ||  y < 0.0;
        let in_wall = x > self.width*0.48 && x < self.width*0.52;
        return (out_of_box || in_wall) && (!self.maxwell.in_bounds(self, x, y));
    }

    pub fn count_balls(&self, s: &Simulation) -> (usize, usize){
        let balls = &s.balls;
        let n_left = balls.iter().filter(|b| b.x < self.width*0.5).count();
        (n_left, balls.len() - n_left)
    }
}

impl Simulation{
    pub fn new(structure: BoxStructure) -> Self{
        Simulation{structure, balls: vec![]}
    }
    pub fn step(&mut self, t: f32){
        for ball in &mut self.balls{
            ball.step(&self.structure, t);
        }
    }

    pub fn random_initiation(&mut self, balls_n: u16, temperature: f32){
        assert!(temperature >= 0.0);
        self.balls = Vec::with_capacity(balls_n.into());
        let mut rng = rand::thread_rng();

        for _ in 0..balls_n{
            self.balls.push(Ball::random_initiation(&self.structure, temperature, &mut rng))
        }
    }

    pub fn paint(&self, painter: &egui::Painter, transform: RectTransform, radius: f32) {
        let (p1, p2) = self.structure.maxwell.coords(&self.structure);
        painter.rect(Rect::from_two_pos(transform*p1, transform*p2), 1.0, Color32::from_gray(16), Stroke::new(1.0, Color32::from_gray(64)));
        for b in &self.balls{
            let point = transform * Pos2{x: b.x, y: b.y};
            painter.circle(point, radius, Color32::from_gray(128), Stroke::new(1.0, Color32::from_gray(64)))
        }
    }
}

impl Ball
{
    fn step(&mut self, b: &BoxStructure, t: f32){ // works for any rectangle-based box
        let new_x = self.x + self.speed_x*t;
        let new_y = self.y + self.speed_y*t;

        match (self.inside_maxwell, b.maxwell.in_bounds(b, new_x, new_y)){
            (true, true) => {
                self.x = new_x;
                self.y = new_y;
            },
            (false, false) => {
                self.wall_reflaction(b, new_x, new_y);
            } 
            (true, false) => {
                if self.wall_reflaction(b, new_x, new_y){
                    println!("Still inside, speed {}, {}", self.speed_x, self.speed_y);
                    self.inside_maxwell = true;
                }
                else{
                    self.inside_maxwell = false;
                }
            },
            (false, true) => {
                self.inside_maxwell = true;
                b.maxwell.refract_ball(self);
                return;
            }
        }
        
    }

    fn wall_reflaction(&mut self, b: &BoxStructure, new_x: f32, new_y: f32) -> bool{
        if b.in_bounds(new_x, self.y){ // problem with x
            self.speed_x = -self.speed_x;
        }
        else if b.in_bounds(new_x, new_y){ // problem with y
            self.speed_y = -self.speed_y;
        }
        else{
            self.x = new_x;
            self.y = new_y;
            return false;
        }
        true
    }

    fn random_initiation<T: Rng>(structure: &BoxStructure, temperature: f32, rng: &mut T) -> Self{
        let mut attempts = 0;
        let mut x;
        let mut y;
        loop {
            x = rng.gen::<f32>() * structure.width;
            y = rng.gen::<f32>() * structure.height;

            if !structure.in_bounds(x, y){
                break;
            }
            attempts += 1;
            if attempts > 10{
                panic!("Impossible to place balls to box")
            }
        }
        
        let speed = rng.sample::<f32, StandardNormal>(StandardNormal) * temperature.sqrt();
        let angle: f32 = rng.gen();
        let speed_x = speed * angle.cos();
        let speed_y = speed * angle.sin();
        Ball{x, y, speed_x, speed_y, inside_maxwell: false}
    }
}