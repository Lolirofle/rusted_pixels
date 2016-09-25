use color::Color;
use image_ext;
use state::State;
use util;

pub use gdk::enums::key as keys;
pub use gdk::enums::key::Key as Keycode;
pub use gdk::ModifierType as Mod;
pub use gdk::{
    CONTROL_MASK as LCTRLMOD,
    MOD1_MASK    as LALTMOD,
};

/*
 * Veeery emacs inspired. Basically a emacs-like commando like
 * `M-x swap-color RET color1 color2`
 * would be translated in code as
 * &[AltModified(char), Exact("swap-color"), Color, Color]
 */

#[derive(Clone,Debug,PartialEq)]
pub enum Input {
    Char(Keycode,Mod),
    Integer,
    Color,
    String,
    Exact(String)
}

#[derive(Clone,Debug,PartialEq)]
pub enum Arg {
    String(String),
    Integer(isize),
    Color(Color),
}

impl Arg {
    pub fn coerce_string(self) -> String {
        if let Arg::String(string) = self {
            return string;
        }
        panic!("Commands misconfigured. Expected `String` on stack.");
    }

    pub fn coerce_color(self) -> Color {
        if let Arg::Color(color) = self {
            return color;
        }
        panic!("Commands misconfigured. Expected `Color` on stack.");
    }
}

#[derive(Copy,Clone,Debug,Eq,PartialEq)]
pub enum Command {
    ExportPng,
    Print,
    Quit,
    SetColor,
    ZoomMult2,
    ZoomDiv2,
}

pub type Commands = Vec<(Vec<Input>,Command)>;

pub const META_X: Input = Input::Char(keys::x,LALTMOD);

#[allow(non_snake_case)]
pub fn get_commands() -> Commands {
    let NOMOD = Mod::empty();

    vec![//TODO: Consider alternative structuring: This looks like a trie and could e.g. be implemented using HashMap<Vec<Input>,RequiresMoreInput|Command> giving approximately O(1) lookup
        (
            vec![Input::Char(keys::s,LCTRLMOD)],
            Command::ExportPng
        ),(
            vec![META_X , Input::Exact(String::from("export-png")) , Input::String],
            Command::ExportPng
        ),(
            vec![Input::Char(keys::q,LCTRLMOD)],
            Command::Quit
        ),(
            vec![META_X , Input::Exact(String::from("quit"))],
            Command::Quit
        ),(
            vec![META_X , Input::Exact(String::from("print")) , Input::String],
            Command::Print
        ),(
            vec![META_X , Input::Exact(String::from("set-color")) , Input::Color],
            Command::SetColor
        ),(
            vec![Input::Char(keys::plus,NOMOD)],
            Command::ZoomMult2
        ),(
            vec![Input::Char(keys::KP_Add,NOMOD)],
            Command::ZoomMult2
        ),(
            vec![Input::Char(keys::minus,NOMOD)],
            Command::ZoomDiv2
        ),(
            vec![Input::Char(keys::KP_Subtract,NOMOD)],
            Command::ZoomDiv2
        )
    ]
}

#[derive(Copy,Clone,Debug,Eq,PartialEq)]
pub enum InterpretErr {
    NoValidCommand,
    RequiresMoreInput
}

/*
 * Interpret the given input to see if there's a matching command
 * Returns matching command or an err if more input is required,
 * or whether there's no possible command for the input so far.
 */
pub fn interpret_input(input: &[Input],
                       commands: &[(Vec<Input>, Command)])
                                   -> Result<Command, InterpretErr> {
    let mut has_match = false;
    for &(ref inputstack, command) in commands {//TODO: This is the trie-like lookup. See notes in get_commands
        if input.len() <= inputstack.len() &&
            input == &inputstack[0..input.len()] {
            has_match = true;
            if input == inputstack.as_slice() {
                return Ok(command);
            }
        }
    }
    Err(match has_match{
        true  => InterpretErr::RequiresMoreInput,
        false => InterpretErr::NoValidCommand
    })
}

#[derive(Copy,Clone,Debug,Eq,PartialEq)]
pub enum CommandResult {
    Quit,
    RequiresMoreInput,
    NoValidCommand,
    Success,
}

/*
 * Check wether the current states input has a valid
 * command, and if so, executes the given command.
 * If no command is possible from the given input,
 * clear the input buffer. Otherwise, do nothing
 * and await more user input
 */
pub fn execute_command(state: &mut State,
                       commands: &[(Vec<Input>, Command)])
-> CommandResult {
    pub fn clean_input_and_args(state: &mut State) {
        state.args = Vec::new();
        state.input = Vec::new();
    }

    match interpret_input(&state.input, commands) {
        Ok(command) => match command {
            Command::ExportPng => {
                let out = state.args.pop()
                    .unwrap_or(Arg::String(String::from("tmp/test_out.png")))
                    .coerce_string();
                image_ext::save_png_image(&state.images[0],out).unwrap();
                println!("exported png");
                clean_input_and_args(state);
                CommandResult::Success
            },
            Command::Quit => {
                println!("quit succesfully");
                CommandResult::Quit
            },
            Command::Print => {
                println!("{}", state.args.pop().unwrap().coerce_string());
                clean_input_and_args(state);
                CommandResult::Success
            },
            Command::SetColor => {
                let color = state.args.pop().unwrap().coerce_color();
                state.current_color = color;
                clean_input_and_args(state);
                println!("set color");
                CommandResult::Success
            },
            Command::ZoomMult2 => {
                state.zoom*=2.0;
                clean_input_and_args(state);
                CommandResult::Success
            },
            Command::ZoomDiv2 => {
                state.zoom/=2.0;
                clean_input_and_args(state);//TODO: Every command cleans? Why not move to end of scope?
                CommandResult::Success
            },
        },
        Err(InterpretErr::NoValidCommand) => {
            clean_input_and_args(state);
            CommandResult::NoValidCommand
        },
        Err(InterpretErr::RequiresMoreInput) => {
            CommandResult::RequiresMoreInput
        }
    }
}

pub fn keycode_to_char(keycode: Keycode) -> Option<char> {
    if (keycode>='A' as Keycode && keycode<='Z'  as Keycode)
    || (keycode>='a' as Keycode && keycode<='z'  as Keycode)
    || keycode=='\'' as Keycode
    || keycode=='-'  as Keycode
    || keycode==' '  as Keycode
    {
        Some((keycode as u8) as char)
    }else{
        None
    }
}

/*
 * Parses the input, returning.
 * If the input is an argument, also return it.
 */
pub fn parse_input(input: &str) -> (Input, Option<Arg>) {
    if let Ok(integer) = input.parse::<isize>() {
        (Input::Integer, Some(Arg::Integer(integer)))
    }
    else if input.len() > 1
        && input.starts_with('\'')
        && input.as_bytes()[input.len() - 1] == b'\'' {
            // remove quotation marks
            let parsed_string = input[1..(input.len() - 1)].to_string();
            (Input::String, Some(Arg::String(parsed_string)))
        }
    else if let Some(color) = util::parse_color(input) {
        (Input::Color, Some(Arg::Color(color)))
    }
    else {
        (Input::Exact(input.to_string()), None)
    }
}

