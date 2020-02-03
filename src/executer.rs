mod commands;
mod instructions;
mod print;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io::stdin;

use instructions::Instruction;
use print::*;

/// A state representing the Y86 program
/// registers: a vector representing the registers
/// condition_code: u8 representing the current set condition codes
/// program_map: a vector holding the byte contents of the program
/// program_size: u64, the size of the program memory
/// program_counter: the program counter at all times, pointing to an address
/// in memory
pub struct State {
    registers: Vec<u64>,
    program_map: Vec<u8>,
    condition_code: u8,
    program_size: u64,
    program_counter: u64,
}

impl State {
    /// Creates a new state of the program from a machine code file
    /// file_name: string representing the file name of a Y86 Machine code
    /// file
    pub fn new(file_name: String) -> Result<Self, Box<dyn Error>> {
        let mut file = File::open(file_name)?;
        let program_size = file.metadata()?.len();
        let program_counter = 0;
        let mut program_map = Vec::new();
        file.read_to_end(&mut program_map)?;
        Ok(State {
            registers: vec![0; 16],
            program_map,
            program_size,
            condition_code: 0,
            program_counter,
        })
    }

    /// Retrieve the value of a register
    /// register_id: u8 representing the id of the register
    pub fn get_register(&self, register_id: u8) -> u64 {
        self.registers[register_id as usize]
    }

    /// sets the value of a register
    /// register_id: u8 representing the id of the register
    /// value: u64 representing the new value to put in the register
    pub fn set_register(&mut self, register_id: u8, value: u64) {
        self.registers[register_id as usize] = value;
    }

    /// Gets the current set condition codes
    pub fn get_condition_code(&self) -> u8 {
        self.condition_code
    }

    /// Sets the condition codes
    /// value: u8 representing the new value
    pub fn set_condition_code(&mut self, value: u8) {
        self.condition_code = value;
    }

    /// Gets the program size
    pub fn get_program_size(&self) -> u64 {
        self.program_size
    }

    /// Reads a memory address in little-endian
    /// address: u64 representing the address
    /// Returns a Result, fails if memory is out of bounds
    pub fn read_le(&self, address: u64) -> Result<u64, Box<dyn Error>> {
        let mut res: u64 = 0;
        for i in 0..8 {
            res = (res << 8) | self.program_map[(address + 7 - i) as usize] as u64;
        }
        Ok(res)
    }

    /// Writes to memory address in little-endian
    /// address: u64 representing the address
    /// value: u64 representing the value to insert into memory
    /// Returns a result, fails if memory is out of bounds
    pub fn write_le(&mut self, address: u64, value: u64) -> Result<(), Box<dyn Error>> {
        for i in 0..8 {
            let val = ((value >> 8 * i) & 0xFF) as u8;
            self.program_map[(address + i) as usize] = val;
        }
        Ok(())
    }

    /// Sets the value of the program counter
    /// new_pc: u64 representing the new pc to set
    pub fn set_pc(&mut self, new_pc: u64) {
        self.program_counter = new_pc;
    }

    /// Gets the current PC
    pub fn get_pc(&self) -> u64 {
        self.program_counter
    }


    /// Reads a single byte in memory
    /// address: u64 representing the address to the value to read
    pub fn read_byte(&self, address: u64) -> u8 {
        self.program_map[address as usize]
    }
}


/// Generic function to debug a Y86 program
/// file_name: String representing the name of a Y86 Machine code file
pub fn debug(file_name: String) -> Result<(), Box<dyn Error>> {
    let mut state = State::new(file_name.clone())?;
    while state.read_byte(state.get_pc()) == 0 {
        state.set_pc(state.get_pc() + 1);
    }
    println!(
        "## Opened {:}, starting PC 0x{:x}",
        file_name,
        state.get_pc()
    );

    loop {
        let mut instruction = Instruction::new(&state)?;
        print_instruction(&instruction);
        print!(">    ");
        std::io::stdout().flush()?;
        let mut buffer = String::new();
        match stdin().read_line(&mut buffer) {
            Ok(_) => (),
            Err(_) => {
                eprintln!("Could not parse input, please try again");
                continue;
            }
        }
        buffer = buffer.trim().to_string();
        if buffer.starts_with("quit") {
            break;
        }
        match commands::run(buffer, &mut instruction, &mut state) {
            Ok(_) => (),
            Err(e) => {
                eprintln!("{:}", e);
            }
        }
    }
    Ok(())
}
