// pong.rs: native Rust implementation for Pong.
//
// This is here mainly for reference / comparison with the lambda calculus
// implementation.
//

use sdl2::rect::Rect;
use crate::game::{UserInput, GameState};

pub const WINDOW_WIDTH:i32 = 800;
pub const WINDOW_HEIGHT:i32 = 600;

// in pixels; applies to both player and CPU
pub const BAR_WIDTH:i32 = WINDOW_WIDTH / 200;
pub const BAR_HEIGHT:i32 = WINDOW_HEIGHT / 10;


const BALL_SIZE_I32:i32 = WINDOW_WIDTH / 200;
const STEP_SIZE:i32 = 2 * BALL_SIZE_I32;
const BALL_SIZE:u32 = BALL_SIZE_I32 as u32;

// player is to the right, CPU is to the left
const PLAYER_X_CENTER:i32 = 9 * WINDOW_WIDTH / 10;
const CPU_X_CENTER:i32 = WINDOW_WIDTH / 10;

// for convenience
pub const PLAYER_X_LEFT:i32 = PLAYER_X_CENTER - BAR_WIDTH / 2;
// pub const PLAYER_X_RIGHT:i32 = PLAYER_X_CENTER + BAR_WIDTH / 2; // never used
pub const CPU_X_LEFT:i32 = CPU_X_CENTER - BAR_WIDTH / 2;
pub const CPU_X_RIGHT:i32 = CPU_X_CENTER + BAR_WIDTH / 2;

const MIN_Y:i32 = STEP_SIZE;
const MAX_Y:i32 = WINDOW_HEIGHT - BAR_HEIGHT - STEP_SIZE;

// scores are shown as if in a seven-segment LED panel, according to the scheme:
//     A
//  F     B
//     G
//  E     C
//     D
//
pub const ACTIVE_LEDS_NUM: &[&[bool]] = &[
    // A      B      C      D      E      F      G
    &[true,  true,  true,  true,  true,  true,  false,], // 0
    &[false, true,  true,  false, false, false, false,], // 1
    &[true,  true,  false, true,  true,  false, true, ], // 2
    &[true,  true,  true,  true,  false, false, true, ], // 3
    &[false, true,  true,  false, false, true,  true, ], // 4
    &[true,  false, true,  true,  false, true,  true, ], // 5
    &[true,  false, true,  true,  true,  true,  true, ], // 6
    &[true,  true,  true,  false, false, false, false,], // 7
    &[true,  true,  true,  true,  true,  true,  true, ], // 8
    &[true,  true,  true,  true,  false, true,  true, ], // 9
];

pub const LED_SMALLER_DIM:u32 = CPU_X_CENTER as u32 / 10;
pub const LED_LARGER_DIM:u32 = 3 * LED_SMALLER_DIM;

// For simplicity, reflections always happen at a straight angle, so
// these are the only possible directions.
//
#[derive(Debug)]
enum Direction {
    NE,
    NW,
    SW,
    SE,
}

pub struct State {
    player_rect: Rect,
    cpu_rect: Rect,
    ball: Ball,
    player_score: usize,
    cpu_score: usize,
    player_led_coords: Vec<Rect>,
    cpu_led_coords: Vec<Rect>,
}

impl State {
    pub fn new() -> State {
        let player_rect = Rect::new(PLAYER_X_LEFT,
                                    WINDOW_HEIGHT / 2 - BAR_HEIGHT / 2,
                                    BAR_WIDTH as u32,
                                    BAR_HEIGHT as u32);
        let cpu_rect = Rect::new(CPU_X_LEFT,
                                 WINDOW_HEIGHT / 2 - BAR_HEIGHT / 2,
                                 BAR_WIDTH as u32,
                                 BAR_HEIGHT as u32);
        // These coordinates are relative to the display's upper-left corner,
        // meaning all rectangles must be transposed to the appropriate (x, y)
        // position for both player and CPU scores.
        //
        let led_relative_coords: &[Rect] = &[
            Rect::new(0,
                      0,
                      LED_LARGER_DIM,
                      LED_SMALLER_DIM), // A
            Rect::new(LED_LARGER_DIM as i32 - LED_SMALLER_DIM as i32,
                      0,
                      LED_SMALLER_DIM,
                      LED_LARGER_DIM), // B
            Rect::new(LED_LARGER_DIM as i32 - LED_SMALLER_DIM as i32,
                      LED_LARGER_DIM as i32,
                      LED_SMALLER_DIM,
                      LED_LARGER_DIM), // C
            Rect::new(0,
                      2 * LED_LARGER_DIM as i32,
                      LED_LARGER_DIM,
                      LED_SMALLER_DIM), // D
            Rect::new(0,
                      LED_LARGER_DIM as i32,
                      LED_SMALLER_DIM,
                      LED_LARGER_DIM), // E
            Rect::new(0,
                      0,
                      LED_SMALLER_DIM,
                      LED_LARGER_DIM), // F
            Rect::new(0,
                      LED_LARGER_DIM as i32,
                      LED_LARGER_DIM,
                      LED_SMALLER_DIM), // G
        ];

        let player_led_coords = led_relative_coords.iter().map(|r| {
            let mut new_r = r.clone();
            new_r.set_x(r.x() + WINDOW_WIDTH - LED_LARGER_DIM as i32);
            new_r
        }).collect::<Vec<Rect>>();

        let cpu_led_coords = led_relative_coords.to_vec();

        State {
            player_rect,
            cpu_rect,
            ball: Ball::new(),
            player_score: 0,
            cpu_score: 0,
            player_led_coords,
            cpu_led_coords,
        }
    }

    fn update_cpu_pos(&mut self) {
        let y = self.cpu_rect.y();
        let cpu_center_y = self.cpu_rect.center().y();
        let ball_center_y = self.ball.rect.center().y();

        // A very simple AI that simply adjusts its center to the ball's center.
        if cpu_center_y > ball_center_y && y > MIN_Y {
            self.cpu_rect.set_y(y - STEP_SIZE);
        } else if cpu_center_y < ball_center_y && y <= MAX_Y {
            self.cpu_rect.set_y(y + STEP_SIZE);
        }
        // else, do nothing.
    }

    fn move_player_up(&mut self) {
        let y = self.player_rect.y();
        if y > MIN_Y {
            self.player_rect.set_y(y - STEP_SIZE);
        }
    }

    fn move_player_down(&mut self) {
        let y = self.player_rect.y();
        if y <= MAX_Y {
            self.player_rect.set_y(y + STEP_SIZE);
        }
    }
}

impl GameState for State {
    fn game_over(&mut self) -> bool {
        return self.player_score >= 10 || self.cpu_score >= 10;
    }

    fn update(&mut self, input: UserInput) {
        match input {
            UserInput::Up => self.move_player_up(),
            UserInput::Down => self.move_player_down(),
            _ => {},
        }

        self.ball.update_pos(&self.player_rect,
                             &self.cpu_rect,
                             &mut self.player_score,
                             &mut self.cpu_score);
        self.update_cpu_pos();
    }

    fn get_rects(&mut self) -> Vec<Rect> {
        let mut rects = vec![self.cpu_rect.clone(), self.player_rect.clone(), self.ball.rect.clone(), ];
        append_active_led_rects(&mut rects, self.player_score, &self.player_led_coords);
        append_active_led_rects(&mut rects, self.cpu_score, &self.cpu_led_coords);

        rects
    }
}

fn append_active_led_rects(rects: &mut Vec<Rect>,
                           score: usize,
                           led_coords: &Vec<Rect>) {
    for (i, active_led) in ACTIVE_LEDS_NUM[score].iter().enumerate() {
        if *active_led {
            rects.push(led_coords[i].clone());
        }
    }
}

struct Ball {
    dir: Direction,
    accel: i32,
    rect: Rect,
}

impl Ball {
    pub fn new() -> Ball {
        Ball {
            dir: Direction::SE,
            accel: 1,
            rect: Rect::new(WINDOW_WIDTH / 2, WINDOW_HEIGHT / 2,
                            BALL_SIZE, BALL_SIZE),
        }
    }

    fn reset(&mut self) {
        self.dir = Direction::SE;
        self.accel = 1;
        self.rect.set_x(WINDOW_WIDTH / 2);
        self.rect.set_y(WINDOW_HEIGHT / 2);
    }

    pub fn update_pos(&mut self,
                      player_rect: &Rect, cpu_rect: &Rect,
                      player_score: &mut usize, cpu_score: &mut usize) {
        let x = self.rect.x();
        let y = self.rect.y();
        let (mut new_x, mut new_y) = self.get_new_pos(x, y);

        // check if someone scored
        if new_x < 0 {
            *player_score += 1;
            self.reset();
            return;
        } else if new_x > WINDOW_WIDTH {
            *cpu_score += 1;
            self.reset();
            return;
        }

        if self.reflect_upper_or_lower_bound(new_y)
            || self.reflect_hit_bar(new_x, new_y, player_rect, cpu_rect)
        {
            // Rust doesn't allow doing this directly
            let (temp1, temp2) = self.get_new_pos(x, y);
            new_x = temp1;
            new_y = temp2;
        }

        self.rect.set_x(new_x);
        self.rect.set_y(new_y);
    }

    // does not check for bouncing, etc
    fn get_new_pos(&self, x: i32, y: i32) -> (i32, i32) {
        let step = self.accel;
        let pos = match self.dir {
            // note: in SDL2, the y axis points south.
            Direction::NE => (x + step, y - step),
            Direction::NW => (x - step, y - step),
            Direction::SE => (x + step, y + step),
            Direction::SW => (x - step, y + step),
        };
        pos
    }

    // make ball bounce horizontally and accelerate it when hitting a bar
    fn reflect_hit_bar(&mut self, new_x: i32, new_y: i32,
                       player_rect: &Rect, cpu_rect: &Rect) -> bool {
        if new_x + self.rect.width() as i32 >= PLAYER_X_LEFT
            && new_y >= player_rect.top() && new_y <= player_rect.bottom()
        {
            self.dir = match self.dir {
                Direction::NE => Direction::NW,
                Direction::SE => Direction::SW,
                Direction::NW => Direction::SW, // when the ball hits under the paddle
                Direction::SW => Direction::NW, // "    "   "    "    over  "
            };
            self.accel += 1;
            true
        } else if new_x < CPU_X_RIGHT
            && new_y >= cpu_rect.top() && new_y <= cpu_rect.bottom()
        {
            self.dir = match self.dir {
                Direction::NW => Direction::NE,
                Direction::SW => Direction::SE,
                Direction::NE => Direction::SE, // when the ball hits under the paddle
                Direction::SE => Direction::NE, // "    "   "    "    over  "
            };
            self.accel += 1;
            true
        } else {
            false
        }
    }

    // make ball bounce vertically when hitting the upper / lower corner
    fn reflect_upper_or_lower_bound(&mut self, new_y: i32) -> bool {
        if new_y + self.rect.height() as i32 > WINDOW_HEIGHT {
            self.dir = match self.dir {
                Direction::SE => Direction::NE,
                Direction::SW => Direction::NW,
                _ => panic!(format!("impossible state 1: direction = {:?}", self.dir)),
            };
            true
        } else if new_y < 0 {
            self.dir = match self.dir {
                Direction::NE => Direction::SE,
                Direction::NW => Direction::SW,
                _ => panic!(format!("impossible state 2: direction = {:?}", self.dir)),
            };
            true
        } else {
            false
        }
    }
}
