extern crate sdl2;

use rand::Rng;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
use sdl2::video::Window;
use std::hash::Hash;
use std::ops::Add;
use std::collections::HashSet;
use std::time::Duration;

const GRID_X_SIZE: u32 = 40;
const GRID_Y_SIZE: u32 = 30;
const DOT_SIZE_IN_PX: u32 = 20;
const INITIAL_SPEED: u32 = 15;
const ACC_RATE: u32 = 1;
const TOP_SPEED: u32 = 5;

pub enum GameState {Playing, Paused, Over}

#[derive(PartialEq)]
pub enum PlayerDirection {Up, Down, Right, Left}

#[derive(Clone, Copy, PartialEq, Hash, Eq)]
pub struct Point(pub i32, pub i32);

impl Add<Point> for Point {
    type Output = Point;

    fn add(self, rhs: Point) -> Self::Output{
        Point(self.0 + rhs.0, self.1 + rhs.1)
    }
}

pub struct Renderer {canvas: WindowCanvas}

impl Renderer{
    pub fn new(window: Window) -> Result<Renderer,String> {
        let canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
        Ok(Renderer { canvas })
    }

    fn draw_dot(&mut self, point: &Point) -> Result<(), String> {
        let Point(x, y) = point;
        self.canvas.fill_rect(Rect::new(
            x * DOT_SIZE_IN_PX as i32,
            y * DOT_SIZE_IN_PX as i32,
            DOT_SIZE_IN_PX,
            DOT_SIZE_IN_PX,
        ))?;

        Ok(())
    }

    pub fn draw(&mut self, context: &GameContext) -> Result<(), String> {
        self.draw_background(context);
        self.draw_player(context)?;
        self.draw_food(context)?;
        self.canvas.present();

        Ok(())
    }

    fn draw_background(&mut self, context: &GameContext) {
        let color = match context.state {
            GameState::Playing => Color::RGB(0, 0, 0),
            GameState::Paused => Color::RGB(30, 30, 30),
            GameState::Over => Color::RGB(30, 0, 0),
        };
        self.canvas.set_draw_color(color);
        self.canvas.clear();
    }

    fn draw_player(&mut self, context: &GameContext) -> Result<(), String> {
        self.canvas.set_draw_color(Color::GREEN);
        for point in &context.player_position {
            self.draw_dot(point)?;
        }
        Ok(())
    }

    fn draw_food(&mut self, context: &GameContext) -> Result<(), String> {
        self.canvas.set_draw_color(Color::RED);
        self.draw_dot(&context.food)?;
        Ok(())
    }
}

pub struct GameContext {
    pub player_position: Vec<Point>,
    pub player_direction: PlayerDirection,
    pub score: u32,
    pub food: Point,
    pub state: GameState,
    pub space: Vec<Point>,
    pub speed: u32,
}

impl GameContext {

    pub fn getspace() -> Vec<Point>{
        let mut spacevec: Vec<Point> = Vec::with_capacity((GRID_X_SIZE*GRID_Y_SIZE).try_into().unwrap());
        for x in 0..GRID_X_SIZE {
            for y in 0..GRID_Y_SIZE {
                spacevec.push(Point(x as i32, y as i32))
            }
        }
        return spacevec
    }

    pub fn new() -> GameContext {
        GameContext {
            player_position: vec![Point(3,1), Point(2,1), Point(1,1)],
            player_direction: PlayerDirection::Right,
            score: 0,
            state: GameState::Paused,
            food: Point(3, 3),
            space: Self::getspace(),
            speed: INITIAL_SPEED, // Lower, faster
        }
    }

    pub fn next_tick(&mut self) {
        if let GameState::Paused = self.state {
            return;
        }
        let head_position = self.player_position.first().unwrap();
        let next_head_position = match self.player_direction{
            PlayerDirection::Up => *head_position + Point(0, -1),
            PlayerDirection::Down => *head_position + Point(0, 1),
            PlayerDirection::Left => *head_position + Point(-1, 0),
            PlayerDirection::Right => *head_position + Point(1, 0),
        };

        if next_head_position.0 >= GRID_X_SIZE.try_into().unwrap() || next_head_position.1 >= GRID_Y_SIZE.try_into().unwrap()
        || next_head_position.0 < 0 || next_head_position.1 < 0 || self.player_position.contains(&next_head_position) {
            self.state = GameState::Over;
            return
        }

        if next_head_position != self.food{
            self.player_position.pop();
        } else {
            self.food = self.getfreespace();
            self.score += 1;
            if !((self.speed-ACC_RATE) <= TOP_SPEED){
                self.speed -= ACC_RATE;
            }
        }
        self.player_position.reverse();
        self.player_position.push(next_head_position);
        self.player_position.reverse();
    }

    pub fn getfreespace(&mut self) -> Point{
        let temp:HashSet<Point> = self.space.iter().cloned().collect();
        let snakeset:HashSet<Point> = self.player_position.iter().cloned().collect();
        let mut freespaceset = temp.difference(&snakeset);
        let x = freespaceset.nth(rand::thread_rng().gen_range(0..freespaceset.clone().count())).unwrap();
        return *x;
    }

    pub fn restartgame(&mut self){
        self.player_position  = vec![Point(3,1), Point(2,1), Point(1,1)];
        self.player_direction = PlayerDirection::Right;
        self.food = Point(3, 3);
        self.space = Self::getspace();
        self.score = 0;
        self.speed = INITIAL_SPEED;
    }

    pub fn toggle_pause(&mut self){
        self.state = match self.state {
            GameState::Paused => GameState::Playing,
            GameState::Playing => GameState::Paused,
            GameState::Over => {
                self.restartgame();
                GameState::Paused
            }
        }
    }

    pub fn move_up(&mut self){
        if self.player_direction != PlayerDirection::Down{
            self.player_direction = PlayerDirection::Up;
        }
    }
    pub fn move_down(&mut self){
        if self.player_direction != PlayerDirection::Up{
            self.player_direction = PlayerDirection::Down;
        }
    }
    pub fn move_left(&mut self){
        if self.player_direction != PlayerDirection::Right{
            self.player_direction = PlayerDirection::Left;
        }
    }
    pub fn move_right(&mut self){
        if self.player_direction != PlayerDirection::Left{
            self.player_direction = PlayerDirection::Right;
        }
    }
}

pub fn main() -> Result<(), String>{
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("snaek", GRID_X_SIZE * DOT_SIZE_IN_PX, GRID_Y_SIZE * DOT_SIZE_IN_PX)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut event_pump = sdl_context.event_pump()?;

    let mut context = GameContext::new();
    let mut renderer = Renderer::new(window)?;

    let mut frame_counter = 0;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown {keycode: Some(keycode), .. } => {
                    match keycode {
                        Keycode::W => context.move_up(),
                        Keycode::S => context.move_down(),
                        Keycode::A => context.move_left(),
                        Keycode::D => context.move_right(),
                        Keycode::Escape => context.toggle_pause(),
                        _ => {}
                    }
                },
                _ => {}
            }
        }
    
    ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));

    frame_counter += 1;
    if frame_counter % context.speed == 0 {
        context.next_tick();
        frame_counter = 0;
    }

    renderer.draw(&context)?;
    }
    Ok(())
}