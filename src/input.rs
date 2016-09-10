use color::Color;
use image_ext;
use state::State;

pub type Keycode = u16;

#[derive(PartialEq)]
pub enum ExtendedChar {
    NonModified(Keycode),
    CtrlModified(Keycode),
    AltModified(Keycode),
}

/*
 * Veeery emacs inspired. Basically a emacs-like commando like
 * `M-x swap-color RET color1 color2`
 * would be translated in code as
 * &[AltModified(char), Exact("swap-color"), Color, Color]
 */

#[derive(PartialEq)]
pub enum Input {
    Char(ExtendedChar),
    Integer,
    Color,
    String,
    Exact(String)
}

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
}

#[derive(Copy, Clone)]
pub enum Command {
    ExportPng,
    Print,
    Quit,
}

pub const META_X: Input
    = Input::Char(ExtendedChar::AltModified('X' as Keycode)); 

pub fn get_commands() -> Vec<(Vec<Input>, Command)> {
    vec![(vec![Input::Char(ExtendedChar::CtrlModified('S' as Keycode))],
          Command::ExportPng),
         (vec![META_X,
               Input::Exact(String::from("export-png"))],
          Command::ExportPng),
         (vec![Input::Char(ExtendedChar::CtrlModified('Q' as Keycode))],
          Command::Quit),
         (vec![META_X,
               Input::Exact(String::from("quit"))],
          Command::Quit),
         (vec![META_X,
               Input::Exact(String::from("print")),
               Input::String],
          Command::Print)
    ]
}


pub enum InterpretErr {
    NoValidCommand,
    RequiresMoreInput
}

/*
 * Interpret the given input to see if there's a matching command
 * Returns matching command or an err if more input is required,
 * or wether there's no possible command for the input so far.
 */
pub fn interpret_input(input: &[Input],
                       commands: &[(Vec<Input>, Command)])
                                   -> Result<Command, InterpretErr> {
    let mut has_match = false;
    for &(ref inputstack, command) in commands {
        if input.len() <= inputstack.len() &&
            input == &inputstack[0..input.len()] {
            has_match = true;
            if input == inputstack.as_slice() {
                return Ok(command);
            }
        }
    }
    match has_match {
        true => Err(InterpretErr::RequiresMoreInput),
        false => Err(InterpretErr::NoValidCommand)
    }
}

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
                image_ext::save_png_image(&state.images[0],"tmp/test_out.png").unwrap();
                println!("Exported PNG image");
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
