use eframe::emath::RectTransform;
use egui::{Pos2, Color32, Stroke, Rect, Vec2};
use rand::Rng;
use rand_distr::StandardNormal;

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct Ball{
    coord: Vec2,
    speed: Vec2,
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
    fn in_bounds(&self, structure: &BoxStructure, coords: Vec2) -> bool{
        let inside_wall = coords.x > structure.width*0.48 && coords.x < structure.width*0.52;
        let accurate_y = true; //coords.y > structure.height*0.45 && coords.y < structure.height*0.55;
        return inside_wall && accurate_y;
    }

    fn refract_ball(&self, ball: &mut Ball){
        match self.filter_type {
            MaxwellType::Diode => {
                if ball.speed.x < 0.0{
                    ball.speed.x = - ball.speed.x;
                }
            },
            MaxwellType::Temperature { t } => {
                if ball.speed.x < 0.0{
                    if ball.speed.x.powf(2.0) < 1.0{
                        ball.speed.x = - ball.speed.x;
                    }
                }
                else{
                    if ball.speed.x.powf(2.0) > t{
                        ball.speed.x = - ball.speed.x;
                    }
                }
            },
            MaxwellType::Tennis => {
                if ball.speed.x * ball.speed.y >= 0.0 && ball.speed.x < ball.speed.y{
                    (ball.speed.x, ball.speed.y) = (ball.speed.y, ball.speed.x)
                }
                else if (ball.speed.x  < 0.0 &&  ball.speed.y > 0.0 && ball.speed.x.abs() > ball.speed.y ) 
                    || (ball.speed.x  > 0.0 &&  ball.speed.y < 0.0 && ball.speed.x < ball.speed.y.abs() ){
                    (ball.speed.x, ball.speed.y) = (-ball.speed.y, -ball.speed.x)
                } 
                else{
                    ball.speed.x = -ball.speed.x;
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

    fn in_bounds(&self, coords: Vec2) -> bool{
        let out_of_box = coords.x > self.width
            ||  coords.y > self.height
            ||  coords.x < 0.0
            ||  coords.y < 0.0;
        let in_wall = coords.x > self.width*0.48 && coords.x < self.width*0.52;
        return (out_of_box || in_wall) && (!self.maxwell.in_bounds(self, coords));
    }

    pub fn count_balls(&self, s: &Simulation) -> (usize, usize){
        let balls = &s.balls;
        let n_left = balls.iter().filter(|b| b.coord.x < self.width*0.5).count();
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
            let point = transform * b.coord.to_pos2();
            painter.circle(point, radius, Color32::from_gray(128), Stroke::new(1.0, Color32::from_gray(64)))
        }
    }
}

impl Ball
{
    fn step(&mut self, b: &BoxStructure, t: f32){ // works for any rectangle-based box
        let new_coord = self.coord + t*self.speed;

        match (self.inside_maxwell, b.maxwell.in_bounds(b, new_coord)){
            (true, true) => {
                self.coord = new_coord;
            },
            (false, false) => {
                self.wall_reflaction(b, new_coord);
            } 
            (true, false) => {
                if self.wall_reflaction(b, new_coord){
                    println!("Still inside, speed {}, {}", self.speed.x, self.speed.y);
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

   /*  fn ball_collision(&self, b2: &Ball, collision_r: f32){
        if b2.coord.x - self.coord.x < 0.1{
            if (b2.coord - self.coord).length() <= collision_r{
                let cm_x = (self.speed.x + b2.speed.x)/2.0;
                let cm_y = (self.speed.y + b2.speed.y)/2.0;
                self.speed.x = 2.0*cm_x-self.speed.x;
                self.speed.y = 2.0*cm_y-self.speed.y;
                b2.speed.x = 2.0*cm_x-b2.speed.x;
                b2.speed.y = 2.0*cm_y-b2.speed.y;
            }
        }
    }*/

    fn wall_reflaction(&mut self, b: &BoxStructure, new_coord: Vec2) -> bool{
        if b.in_bounds(Vec2{x: new_coord.x, y: self.coord.y}){ // problem with x
            self.speed.x = -self.speed.x;
        }
        else if b.in_bounds(new_coord){ // problem with y
            self.speed.y = -self.speed.y;
        }
        else{
            self.coord = new_coord;
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

            if !structure.in_bounds(Vec2{x, y}){
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
        Ball{coord: Vec2{x, y}, speed: Vec2{x: speed_x, y: speed_y}, inside_maxwell: false}
    }
}