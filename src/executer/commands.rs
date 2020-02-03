use super::instructions::{ICode, Instruction};
use super::print::{print_all_registers, print_memory_quad_value};
use super::State;
use lazy_static::lazy_static;
use std::collections::HashSet;
use std::error::Error;
use std::sync::{Arc, Mutex};

lazy_static! {
    static ref SET: Arc<Mutex<HashSet<u64>>> = Arc::new(Mutex::new(HashSet::new()));
}

#[derive(Debug, Clone)]
pub struct InvalidParameter;

impl std::fmt::Display for InvalidParameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid parameter")
    }
}

impl Error for InvalidParameter {
    fn description(&self) -> &str {
        "Invalid parameter"
    }

    fn cause(&self) -> Option<&dyn Error> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}

pub fn run(
    input: String,
    instr: &mut Instruction,
    state: &mut State,
) -> Result<(), Box<dyn Error>> {
    let space = input.find(" ");
    let command = match space {
        Some(index) => input.clone()[0..index].to_string(),
        None => input.clone(),
    };
    match command.as_str() {
        "step" => run_step(instr, state),
        "run" => run_run(instr, state),
        "next" => run_next(instr, state),
        "jump" => run_jump(input, instr, state),
        "break" => run_break(input, instr, state),
        "delete" => run_delete(input, instr, state),
        "registers" => run_registers(instr, state),
        "examine" => run_examine(input, instr, state),
        _ => Ok(eprintln!("Invalid command, please try again")),
    }
}

fn run_step(instr: &mut Instruction, state: &mut State) -> Result<(), Box<dyn Error>> {
    instr.execute(state)?;
    Ok(())
}
fn run_run(instr: &mut Instruction, state: &mut State) -> Result<(), Box<dyn Error>> {
    instr.execute(state)?;
    let mut curr = Instruction::new(&state)?;
    while !SET.lock().unwrap().contains(&curr.get_location()) && curr.get_icode() != ICode::IHALT {
        curr.execute(state)?;
        curr = Instruction::new(&state)?;
    }
    Ok(())
}
fn run_next(instr: &mut Instruction, state: &mut State) -> Result<(), Box<dyn Error>> {
    let val_p = instr.get_val_p();
    instr.execute(state)?;
    let mut curr = Instruction::new(&state)?;
    while !SET.lock().unwrap().contains(&curr.get_location())
        && curr.get_icode() != ICode::IHALT
        && state.get_pc() != val_p
    {
        curr.execute(state)?;
        curr = Instruction::new(&state)?;
    }
    Ok(())
}
fn run_jump(
    input: String,
    _instr: &mut Instruction,
    state: &mut State,
) -> Result<(), Box<dyn Error>> {
    let index = input.find(" ");

    let destination: u64 = match index {
        Some(i) => {
            let num = &input[i..].trim().trim_start_matches("0x");
            u64::from_str_radix(num, 16)?
        }
        None => {
            let boxed: Box<InvalidParameter> = InvalidParameter.into();
            Err(boxed)?
        }
    };
    state.set_pc(destination);
    Ok(())
}
fn run_break(
    input: String,
    _instr: &mut Instruction,
    _state: &mut State,
) -> Result<(), Box<dyn Error>> {
    let index = input.find(" ");

    let breakpoint: u64 = match index {
        Some(i) => {
            let num = &input[i..].trim().trim_start_matches("0x");
            u64::from_str_radix(num, 16)?
        }
        None => {
            let boxed: Box<InvalidParameter> = InvalidParameter.into();
            Err(boxed)?
        }
    };
    SET.lock().unwrap().insert(breakpoint);
    Ok(())
}
fn run_delete(
    input: String,
    _instr: &mut Instruction,
    _state: &mut State,
) -> Result<(), Box<dyn Error>> {
    let index = input.find(" ");

    let breakpoint: u64 = match index {
        Some(i) => {
            let num = &input[i..].trim().trim_start_matches("0x");
            u64::from_str_radix(num, 16)?
        }
        None => {
            let boxed: Box<InvalidParameter> = InvalidParameter.into();
            Err(boxed)?
        }
    };
    SET.lock().unwrap().remove(&breakpoint);
    Ok(())
}

fn run_registers(_instr: &mut Instruction, state: &mut State) -> Result<(), Box<dyn Error>> {
    print_all_registers(state);
    Ok(())
}

fn run_examine(
    input: String,
    _instr: &mut Instruction,
    state: &mut State,
) -> Result<(), Box<dyn Error>> {
    let index = input.find(" ");

    let address: u64 = match index {
        Some(i) => {
            let num = &input[i..].trim().trim_start_matches("0x");
            u64::from_str_radix(num, 16)?
        }
        None => {
            let boxed: Box<InvalidParameter> = InvalidParameter.into();
            Err(boxed)?
        }
    };
    print_memory_quad_value(state, address);
    Ok(())
}
