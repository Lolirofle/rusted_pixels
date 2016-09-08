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
    Exact(&'static str),
}

pub enum Arg {
    String(String),
    Integer(isize),
    Color(Color),
}

impl Arg {

}

#[derive(Copy, Clone)]
pub enum Command {
    ExportPng,
    Quit,
}

pub static COMMANDS: &'static [(&'static [Input], Command)] =
    &[(&[Input::Char(ExtendedChar::CtrlModified('S' as Keycode))],
       Command::ExportPng),
      (&[Input::Char(ExtendedChar::AltModified('X' as Keycode)),
         Input::Exact("export-png")],
       Command::ExportPng),
      (&[Input::Char(ExtendedChar::CtrlModified('Q' as Keycode))],
       Command::Quit)
    ];

pub enum InterpretErr {
    NoValidCommand,
    RequiresMoreInput
}


pub fn interpret_input(input: &[Input]) -> Result<Command, InterpretErr> {
    let mut has_match = false;
    for &(inputstack, command) in COMMANDS {
        if input == &inputstack[0..input.len()] {
            has_match = true;
            if input == inputstack {
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

pub fn execute_command(state: &mut State) -> CommandResult {
    pub fn clean_input_and_args(state: &mut State) {
        state.args = Vec::new();
        state.input = Vec::new();
    }
    
    match interpret_input(&state.input) {
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
