use sdl2::{
    EventPump,
    rect::Rect,
    video::Window,
    pixels::Color,
    event::Event,
    keyboard::Keycode,
    render::Canvas,
};

use crate::pong::{
    WINDOW_WIDTH, WINDOW_HEIGHT,
};

pub enum UserInput {
    Up,
    Down,
    Nothing,
}

pub fn game_init(window_name: &str) -> (Canvas<Window>, EventPump) {
    let sdl_context = sdl2::init()
        .expect("failed to initialize SDL");

    let video_sys = sdl_context.video()
        .expect("failed to initialize video subsystem");

    let window = video_sys.window(window_name, WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32)
        .position_centered()
        .build()
        .expect("failed to create window");

    let canvas = window.into_canvas()
        .present_vsync()
        .build()
        .expect("failed to create canvas / renderer");

    let event_pump = sdl_context.event_pump()
        .expect("failed to get event pump");

    (canvas, event_pump)
}

pub trait GameState {
    fn game_over(&mut self) -> bool;
    fn update(&mut self, input: UserInput);
    fn get_rects(&mut self) -> Vec<Rect>;
}

pub fn game_loop<S: GameState>(mut canvas: Canvas<Window>,
                               mut event_pump: EventPump,
                               mut state: S)
{
    let mut user_input;
    'game_loop: loop {
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        user_input = UserInput::Nothing;
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => break 'game_loop,

                Event::KeyDown { keycode, .. } => match keycode {
                    Some(Keycode::Q) => break 'game_loop,
                    Some(Keycode::Up) | Some(Keycode::K) => {
                        user_input = UserInput::Up;
                    },
                    Some(Keycode::Down) | Some(Keycode::J) => {
                        user_input = UserInput::Down;
                    },
                    _ => {},
                },
                _ => {},
            }
        }

        state.update(user_input);

        if state.game_over() {
            break 'game_loop;
        }

        canvas.set_draw_color(Color::RGB(0xff, 0xff, 0xff));
        for rect in state.get_rects() {
            canvas.fill_rect(Some(rect.clone())).expect("failed to draw rectangle");
        }

        canvas.present();
    }
}
