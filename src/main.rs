mod pong;
mod lambda;
mod game;
mod parse_out;

use std::{
    env,
};

enum Backend {
    Native,
    Lambda(String),
}

fn usage() {
    eprintln!("usage: <program_name> <backend>");
    eprintln!("where <backend> is one of:");
    eprintln!("\t-n\tnative Rust backend");
    eprintln!("\t-l <filename>\tlambda calculus backend using source <filename>");
}

fn parse_args() -> Option<Backend> {
    let mut args = env::args();
    args.next(); // skip program name
    let backend = match args.next() {
        None => {
            eprintln!("error: no backend specified.");
            usage();
            None
        },
        Some(arg) => {
            if arg == "-n" {
                Some(Backend::Native)
            } else if arg == "-l" {
                match args.next() {
                    None => {
                        eprintln!("error: option '-l' requires a filename.");
                        usage();
                        None
                    },
                    Some(filename) => {
                        Some(Backend::Lambda(filename))
                    },
                }
            } else {
                eprintln!("unknown option '{}'", arg);
                usage();
                None
            }
        },
    };
    backend
}

fn main() {
    if let Some(backend) = parse_args() {
        match backend {
            Backend::Native => {
                let native_state = pong::State::new();
                let (canvas, event_pump) = game::game_init("native pong");
                game::game_loop(canvas, event_pump, native_state);
            },
            Backend::Lambda(filename) => {
                let lambda_state = match lambda::State::new(&filename) {
                    Ok(state) => state,
                    Err(e) => {
                        eprintln!("failed to create lambda state: '{}'", e);
                        return;
                    },
                };
                let (canvas, event_pump) = game::game_init("lambda pong");
                game::game_loop(canvas, event_pump, lambda_state);
            }
        };
    }
}
