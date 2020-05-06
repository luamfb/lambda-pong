use sdl2::{
    rect::Rect,
};

use std::{
    str,
    process::{Command, Stdio, Child},
    io::{Write, BufRead, BufReader},
    fs::File,
    thread,
    time::Duration,
    //iter::DoubleEndedIterator,
};

use crate::{
    parse_out,
    game::{GameState, UserInput},
};

// Name of the lambda calculus interpreter.
// We assume it can be found in PATH.
//
const LAMBDA_CALC_BIN_NAME: &str = "lambda_calc";

// All these symbols must be exported in lambda calculus source file used.
//
const SCALING_FACTOR_NAME: &str = "scalingFactor";
const X_OFFSET_NAME: &str = "xOffset";
const Y_OFFSET_NAME: &str = "yOffset";
const USER_INPUT_UP: &str = "up";
const USER_INPUT_DOWN: &str = "down";
const USER_INPUT_NONE: &str = "none";
const INITIAL_STATE: &str = "initState";
const GAME_OVER: &str = "gameOver";
const UPDATE_STATE: &str = "nextState";
const GET_RECTS: &str = "getScreenRects";

pub struct State {
    lambda_proc: Child,
    scaling_factor: i32,
    x_offset: i32,
    y_offset: i32,
    state: String,
}

impl State {
    pub fn new(filename: &str) -> Result<State, String> {
        let file = match File::open(filename) {
            Ok(f) => f,
            Err(e) => return Err(format!("failed to open file '{}': '{}'", filename, e)),
        };

        let lambda_proc = Command::new(LAMBDA_CALC_BIN_NAME)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .arg("-n")
            .spawn();
        let mut lambda_proc = match lambda_proc {
            Ok(p) => p,
            Err(e) => return Err(format!("failed to spawn lambda interpreter process: '{}'.
Make sure the 'lambda_calc' binary is installed in a directory included in your PATH.", e)),
        };

        let in_stream_unwrapped = match lambda_proc.stdin {
            None => return Err(format!("no input stream in lambda interpreter process")),
            Some(ref mut stream) => stream,
        };

        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = match line {
                Ok(s) => s,
                Err(e) => return Err(format!("failed to read line from file: {}", e)),
            };
            if let Err(e) = write!(in_stream_unwrapped, "{}\n", &line) {
                return Err(format!("failed to write to process's input stream: '{}'", e));
            };
        }
        let scaling_factor = get_child_output_line_for_input(&mut lambda_proc,
                                                             &SCALING_FACTOR_NAME)?;
        let scaling_factor = parse_out::clni_to_int(&scaling_factor)?;

        let x_offset = get_child_output_line_for_input(&mut lambda_proc,
                                                            &X_OFFSET_NAME)?;
        let x_offset = parse_out::clni_to_int(&x_offset)?;

        let y_offset = get_child_output_line_for_input(&mut lambda_proc,
                                                            &Y_OFFSET_NAME)?;
        let y_offset = parse_out::clni_to_int(&y_offset)?;

        let init_state = get_child_output_line_for_input(&mut lambda_proc,
                                                         &INITIAL_STATE)?;
        Ok(State {
            lambda_proc,
            scaling_factor,
            x_offset,
            y_offset,
            state: init_state,
        })
    }

    fn get_output(&mut self, input: &str) -> String {
        let output = match get_child_output_line_for_input(&mut self.lambda_proc,
                                                           input) {
            Ok(s) => s,
            Err(e) => panic!("failed to get lambda interpreter output: '{}'", e),
        };
        output
    }
}

impl GameState for State {
    fn game_over(&mut self) -> bool {
        let lambda_expr = format!("{} {}", GAME_OVER, &self.state);
        let answer_str = self.get_output(&lambda_expr);

        let answer = match parse_out::parse_church_bool(&answer_str) {
            Ok(ans) => ans,
            Err(e) => panic!("failed to parse output as a Church boolean: '{}'", e),
        };
        answer
    }

    fn update(&mut self, input: UserInput) {
        let user_input = match input {
            UserInput::Up => USER_INPUT_UP,
            UserInput::Down => USER_INPUT_DOWN,
            UserInput::Nothing => USER_INPUT_NONE,
        };
        let lambda_expr = format!("{} {} {}", UPDATE_STATE, &self.state, user_input);
        self.state = self.get_output(&lambda_expr);
    }

    fn get_rects(&mut self) -> Vec<Rect> {
        let lambda_expr = format!("{} {}", GET_RECTS, &self.state);
        let rects_str = self.get_output(&lambda_expr);

        let rects = parse_out::parse_rect_list(&rects_str,
                                               self.scaling_factor,
                                               self.x_offset,
                                               self.y_offset);
        let rects = match rects {
            Ok(r) => r,
            Err(e) => panic!("failed to parse list of rectangles: '{}'", e),
        };
        rects
    }
}

fn get_child_output_line_for_input(child: &mut Child,
                                   input: &str) -> Result<String, String> {
    let read_interval = Duration::from_millis(1);

    let child_stdin = match child.stdin {
        Some(ref mut stream) => stream,
        None => return Err(format!("no stdin stream in lambda interpreter")),
    };
    if let Err(e) = write!(child_stdin, "{}\n", input) {
        return Err(format!("failed to write to process's input stream: '{}'", e));
    }

    let mut output = String::new();
    let mut read_line_retval;
    loop {
        {
            let child_stdout = match child.stdout {
                Some(ref mut stream) => stream,
                None => return Err(format!("no stdout stream in lambda interpreter")),
            };
            let mut output_reader = BufReader::new(child_stdout);
            read_line_retval = output_reader.read_line(&mut output);
        }
        match read_line_retval {
            Err(e) => return Err(e.to_string()),
            Ok(0) => {
                // If we're here, it's possible that:
                // - the output is just not availibale yet, or
                // - there was a syntax error that made the child process die.
                //
                match child.try_wait() {
                    Err(e) => return Err(format!("failed to check if lambda interpreter terminated: '{}'", e)),
                    Ok(Some(_)) => return Err(format!("lambda interpreter already terminated; input was `{}`", input)),
                    Ok(None) => thread::sleep(read_interval), // wait before trying to read again
                };
            },
            Ok(_) => break,
        };
    };
    return Ok(output.replace("\n", ""));
}
