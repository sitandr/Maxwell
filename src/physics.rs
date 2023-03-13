use std::f32::consts::PI;

use eframe::emath::{RectTransform, Rot2};
use egui::{Pos2, Color32, Stroke, Rect, Vec2, Painter};
use rand::Rng;
use rand_distr::{StandardNormal};

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
    wall_left: f32,
    wall_right: f32,
    maxwell: Maxwell
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct Simulation{
    pub structure: BoxStructure,
    pub collision_radius: f32,
    pub collisions: bool,
    balls: Vec<Ball>
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, Copy, PartialEq)]
pub enum MaxwellType{
    Diode,
    Temperature{t: f32},
    Tennis,
    PhaseConserving {c: f32},
    Empty
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct Maxwell{
    filter_type: MaxwellType,
    top: f32,
    bottom: f32
}

impl Maxwell{
    fn new(filter_type: MaxwellType, height: f32) -> Self{
        Self{filter_type, top: (1.0 + height)/2.0, bottom: (1.0 - height)/2.0}
    }
    fn in_bounds(&self, structure: &BoxStructure, coords: Vec2, collision_radius: f32) -> bool{
        if self.top == self.bottom{
            return false
        }
        let inside_wall = coords.x > structure.wall_left - collision_radius && coords.x < structure.wall_right + collision_radius;
        let accurate_y = coords.y < self.top - collision_radius && coords.y > self.bottom + collision_radius;
        return inside_wall && accurate_y;
    }

    fn refract_ball(&self, ball: &mut Ball){
        match self.filter_type {
            MaxwellType::Empty => {},
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
             //   print!("{:?}", ball.speed);
                if ball.speed.x * ball.speed.y >= 0.0 && ball.speed.x < ball.speed.y{
                    (ball.speed.x, ball.speed.y) = (ball.speed.y, ball.speed.x)
                }
                else if (ball.speed.x  < 0.0 &&  ball.speed.y > 0.0 && ball.speed.x.abs() > ball.speed.y.abs() ) 
                    || (ball.speed.x  > 0.0 &&  ball.speed.y < 0.0 && ball.speed.x.abs() < ball.speed.y.abs() ){
                    (ball.speed.x, ball.speed.y) = (-ball.speed.y, -ball.speed.x)
                } 
                else{
                    ball.speed.x = -ball.speed.x;
                }
               // println!(" → {:?}", ball.speed)
            },
            MaxwellType::PhaseConserving { c } => {
                let speed = ball.speed.length();
                let angle = ball.speed.angle();
                let new_v = angle.sin() + c;
                if new_v.abs() <= 1.0{
                    let new_angle = if angle.abs() < PI/2.0{
                        new_v.asin()
                    }
                    else{
                        PI - new_v.asin()
                    };
                    
                    ball.speed = Vec2::new(speed*new_angle.cos(), speed*new_angle.sin())
                }
                else{
                    ball.speed.x = -ball.speed.x;
                }
                
            },
        }
        
    }

    fn coords(&self, structure: &BoxStructure) -> (Pos2, Pos2){
        (Pos2::new(structure.wall_left, self.bottom),
        Pos2::new(structure.wall_right, self.top))
    }
}

impl BoxStructure{
    pub fn new() -> Self{
        Self { width: 1.0, height: 1.0, wall_left: 0.48, wall_right: 0.52, maxwell: Maxwell::new(MaxwellType::Tennis, 0.0)}
    }

    fn in_bounds(&self, coords: Vec2, collision_radius: f32) -> bool{
        let out_of_box = coords.x > self.width - collision_radius
            ||  coords.y > self.height - collision_radius
            ||  coords.x < collision_radius
            ||  coords.y < collision_radius;
        let in_wall = coords.x > self.wall_left - collision_radius && coords.x < self.wall_right + collision_radius;
        return (out_of_box || in_wall) && (!self.maxwell.in_bounds(self, coords, collision_radius));
    }

    pub fn count_balls(&self, s: &Simulation) -> (usize, usize){
        let balls = &s.balls;
        let n_left = balls.iter().filter(|b| b.coord.x < self.width*0.5).count();
        (n_left, balls.len() - n_left)
    }

    fn coords(&self) -> (Pos2, Pos2){
        (Pos2::new(self.wall_left, 0.0),
        Pos2::new(self.wall_right, self.height))
    }
}

impl Simulation{
    pub fn new() -> Self{
        Simulation{structure: BoxStructure::new(), collision_radius: 0.1, balls: vec![], collisions: true}
    }

    pub fn step(&mut self, t: f32){
        if self.collisions {
            self.ball_collider(t);
        }
        for ball in &mut self.balls{
            ball.step(&self.structure, t, self.collision_radius);
        }
    }

    pub fn ball_collider(&mut self, t: f32){
        for i in 0..self.balls.len(){
            for j in 0..i{
                let ball = &self.balls[i];
                let other_ball = &self.balls[j];
                if (ball.coord.x - other_ball.coord.x).abs() <= 10.0*self.collision_radius{
                    let new_coord_one = ball.coord + ball.speed * t;
                    let new_coord_other = other_ball.coord + other_ball.speed * t;
                    let delta = new_coord_one - new_coord_other;
                    //let delta = ball.coord - other_ball.coord;
                    if delta.length() <= 2.0*self.collision_radius{
                        let cm = (other_ball.speed + ball.speed)/2.0;

                        let angle = delta.angle();
                        let new_ball_speed = cm - Rot2::from_angle(angle) * (other_ball.speed - cm);
                        let new_other_speed = cm - Rot2::from_angle(angle) * (ball.speed - cm);
                        self.balls[i].speed = new_ball_speed;
                        self.balls[j].speed = new_other_speed;
                    }
                }
            }
        }
    }

    pub fn random_initiation(&mut self, balls_n: u16, temperature: f32, radius: f32, filter_height: f32, filter_type: MaxwellType, collisions: bool, wall_width: f32){
        assert!(temperature >= 0.0);
        self.balls = Vec::with_capacity(balls_n.into());
        self.collision_radius = radius;
        self.structure.maxwell = Maxwell::new(filter_type, filter_height);
        self.structure.wall_left = 0.5 - wall_width/2.0;
        self.structure.wall_right = 0.5 + wall_width/2.0;
        self.collisions = collisions;
        let mut rng = rand::thread_rng();

        for _ in 0..balls_n{
            self.balls.push(Ball::random_initiation(&self.structure, temperature, &mut rng, radius))
        }
        /*self.balls.push(Ball{ coord: Vec2 { x: 0.3, y: 0.3 }, speed: Vec2 { x: 0.1, y: 1.0 }, inside_maxwell: false });
        self.collision_radius = 0.05;
        self.structure.maxwell = Maxwell::new(MaxwellType::Tennis, 0.8)*/

    }

    pub fn paint(&self, painter: &Painter, transform: RectTransform) {
        
        let real_radius = transform.scale().x * self.collision_radius;
        let (p1, p2) = self.structure.coords();
        painter.rect(Rect::from_two_pos(transform*p1, transform*p2), 1.0, Color32::from_gray(48), Stroke::new(1.0, Color32::from_gray(64)));
        if self.structure.maxwell.top != self.structure.maxwell.bottom{
            let (p1, p2) = self.structure.maxwell.coords(&self.structure);
            painter.rect(Rect::from_two_pos(transform*p1, transform*p2), 1.0, Color32::from_gray(16), Stroke::new(1.0, Color32::from_gray(16)));
        }
        for b in &self.balls{
            let point = transform * b.coord.to_pos2();
            painter.circle(point, real_radius, Color32::from_gray(128), Stroke::new(1.0, Color32::from_gray(64)))
        }
    }
}

impl Ball
{
    fn step(&mut self, b: &BoxStructure, t: f32, collision_radius: f32){ // works for any rectangle-based box
        let new_coord = self.coord + t*self.speed;

        match (self.inside_maxwell, b.maxwell.in_bounds(b, new_coord, collision_radius)){
            (true, true) => {
                self.coord = new_coord;
            },
            (false, false) => {
                self.wall_reflaction(b, new_coord, collision_radius);
            } 
            (true, false) => {
                if self.wall_reflaction(b, new_coord, collision_radius){
                    self.inside_maxwell = true;
                }
                else{
                    self.inside_maxwell = false;
                }
            },
            (false, true) => {
                self.inside_maxwell = true;
                self.coord = new_coord;
                b.maxwell.refract_ball(self);
                return;
            }
        }
        
    }

    fn wall_reflaction(&mut self, b: &BoxStructure, new_coord: Vec2, collision_radius: f32) -> bool{
        if b.in_bounds(Vec2{x: new_coord.x, y: self.coord.y}, collision_radius){ // problem with x
            self.speed.x = -self.speed.x;
        }
        else if b.in_bounds(new_coord, collision_radius){ // problem with y
            self.speed.y = -self.speed.y;
        }
        else{
            self.coord = new_coord;
            return false;
        }
        true
    }

    fn random_initiation<T: Rng>(structure: &BoxStructure, temperature: f32, rng: &mut T, collision_radius: f32) -> Self{
        let mut attempts = 0;
        let mut x;
        let mut y;
        loop {
            x = rng.gen::<f32>() * structure.width;
            y = rng.gen::<f32>() * structure.height;

            if !structure.in_bounds(Vec2{x, y}, collision_radius){
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