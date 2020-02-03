use lazy_static::lazy_static;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{self, Display, Formatter};

lazy_static! {
    static ref INSTRUCTION_CODE: HashMap<&'static str, u8> = vec![
        ("halt", (ICode::IHALT as u8) << 4),
        ("nop", (ICode::INOP as u8) << 4),
        ("rrmovq", (ICode::IRRMVXX as u8) << 4),
        ("cmovle", (ICode::IRRMVXX as u8) << 4 | 1),
        ("cmovl", (ICode::IRRMVXX as u8) << 4 | 2),
        ("cmove", (ICode::IRRMVXX as u8) << 4 | 3),
        ("cmovne", (ICode::IRRMVXX as u8) << 4 | 4),
        ("cmovge", (ICode::IRRMVXX as u8) << 4 | 5),
        ("cmovg", (ICode::IRRMVXX as u8) << 4 | 6),
        ("rmmovq", (ICode::IRMMOVQ as u8) << 4),
        ("mrmovq", (ICode::IMRMOVQ as u8) << 4),
        ("irmovq", (ICode::IIRMOVQ as u8) << 4),
        ("addq", (ICode::IOPQ as u8) << 4),
        ("subq", (ICode::IOPQ as u8) << 4 | 1),
        ("andq", (ICode::IOPQ as u8) << 4 | 2),
        ("xorq", (ICode::IOPQ as u8) << 4 | 3),
        ("mulq", (ICode::IOPQ as u8) << 4 | 4),
        ("divq", (ICode::IOPQ as u8) << 4 | 5),
        ("modq", (ICode::IOPQ as u8) << 4 | 6),
        ("jmp", (ICode::IJXX as u8) << 4),
        ("jle", (ICode::IJXX as u8) << 4 | 1),
        ("jl", (ICode::IJXX as u8) << 4 | 2),
        ("je", (ICode::IJXX as u8) << 4 | 3),
        ("jne", (ICode::IJXX as u8) << 4 | 4),
        ("jge", (ICode::IJXX as u8) << 4 | 5),
        ("jg", (ICode::IJXX as u8) << 4 | 6),
        ("call", (ICode::ICALL as u8) << 4),
        ("ret", (ICode::IRET as u8) << 4),
        ("pushq", (ICode::IPUSHQ as u8) << 4),
        ("popq", (ICode::IPOPQ as u8) << 4)
    ]
    .into_iter()
    .collect();
    static ref REGISTERS: HashMap<&'static str, u8> = vec![
        ("%rax", Register::RRAX as u8),
        ("%rcx", Register::RRCX as u8),
        ("%rdx", Register::RRDX as u8),
        ("%rbx", Register::RRBX as u8),
        ("%rsp", Register::RRSP as u8),
        ("%rbp", Register::RRBP as u8),
        ("%rsi", Register::RRSI as u8),
        ("%rdi", Register::RRDI as u8),
        ("%r8", Register::RR8 as u8),
        ("%r9", Register::RR9 as u8),
        ("%r10", Register::RR10 as u8),
        ("%r11", Register::RR11 as u8),
        ("%r12", Register::RR12 as u8),
        ("%r13", Register::RR13 as u8),
        ("%r14", Register::RR14 as u8),
    ]
    .into_iter()
    .collect();
}

pub fn parse(line: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    if line.contains(".quad") {
        parse_quad(line)
    } else {
        let mut split_line = line.split(' ');
        let instr = Parser::new(&split_line.next().unwrap().to_string())?;
        instr.parse(line)
    }
}

pub fn get_icode_from_string(string: &str) -> Result<ICode, Box<dyn Error>> {
    let b: u8 = match INSTRUCTION_CODE.get(string) {
        Some(&val) => val,
        None => return Err(Box::new(InvalidInstructionError)),
    };
    get_icode_from_byte(b)
}

pub fn parse_quad(line: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut split = line.split(".quad");
    split.next();
    let val = split.next().unwrap();
    let parsed = get_immediate(val.trim())?;
    let mut res = vec![];
    push_le(&mut res, parsed);
    Ok(res)
}

#[derive(Copy, Clone, FromPrimitive, PartialEq)]
pub enum ICode {
    IHALT = 0x0,
    INOP = 0x1,
    IRRMVXX = 0x2,
    IIRMOVQ = 0x3,
    IRMMOVQ = 0x4,
    IMRMOVQ = 0x5,
    IOPQ = 0x6,
    IJXX = 0x7,
    ICALL = 0x8,
    IRET = 0x9,
    IPUSHQ = 0xA,
    IPOPQ = 0xB,
    IINVALID = 0x10,
    ITOOSHORT = 0x11,
}
#[derive(Copy, Clone, FromPrimitive)]
pub enum Register {
    RRAX = 0x0,
    RRCX = 0x1,
    RRDX = 0x2,
    RRBX = 0x3,
    RRSP = 0x4,
    RRBP = 0x5,
    RRSI = 0x6,
    RRDI = 0x7,
    RR8 = 0x8,
    RR9 = 0x9,
    RR10 = 0xA,
    RR11 = 0xB,
    RR12 = 0xC,
    RR13 = 0xD,
    RR14 = 0xE,
    RNONE = 0xF,
}

pub struct Parser {
    instruction_type: u8,
}

#[derive(Debug)]
struct InvalidInstructionError;

impl std::error::Error for InvalidInstructionError {}

impl Display for InvalidInstructionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid instruction")
    }
}

#[derive(Debug)]
struct InvalidRegisterError;

impl std::error::Error for InvalidRegisterError {}

impl Display for InvalidRegisterError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid Register")
    }
}

pub fn get_icode_from_byte(b: u8) -> Result<ICode, Box<dyn std::error::Error>> {
    match FromPrimitive::from_u8(b >> 4) {
        Some(val) => Ok(val),
        None => Err(Box::new(InvalidInstructionError)),
    }
}

impl Parser {
    pub fn new(instr: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let instruction_type = match INSTRUCTION_CODE.get(&instr[..]) {
            Some(&val) => val,
            None => return Err(Box::new(InvalidInstructionError)),
        };
        Ok(Parser { instruction_type })
    }

    pub fn parse(&self, line: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut res = vec![self.instruction_type];
        match get_icode_from_byte(self.instruction_type)? {
            ICode::IIRMOVQ => parse_irmovq(line, &mut res)?,
            ICode::IRRMVXX | ICode::IOPQ => parse_rr_opq(line, &mut res)?,
            ICode::IMRMOVQ => parse_mrmovq(line, &mut res)?,
            ICode::IRMMOVQ => parse_rmmovq(line, &mut res)?,
            ICode::IJXX | ICode::ICALL => parse_jxx_call(line, &mut res)?,
            ICode::IRET | ICode::IHALT | ICode::INOP => {}
            ICode::IPUSHQ | ICode::IPOPQ => parse_push_pop(line, &mut res)?,
            _ => return Err(Box::new(InvalidInstructionError)),
        };
        Ok(res)
    }
}

fn form_byte(first: u8, second: u8) -> u8 {
    ((first << 4) & 0xF0) | (second & 0x0F)
}

fn get_immediate(value: &str) -> Result<u64, Box<dyn std::error::Error>> {
    crate::number_parser::parse_num(value)
}

fn get_register(value: &str) -> Result<u8, Box<dyn std::error::Error>> {
    match REGISTERS.get(value.trim()) {
        Some(&val) => Ok(val),
        None => Err(Box::new(InvalidRegisterError)),
    }
}

fn push_le(vec: &mut Vec<u8>, val: u64) {
    for i in 0..8 {
        vec.push((val >> (i * 8)) as u8);
    }
}

fn parse_irmovq(line: &str, res: &mut Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
    let mut split = line.split(',');
    let instr_val = split.next().unwrap();
    let mut instr_val = instr_val.split(' ');
    instr_val.next();
    let mut first = instr_val.next();
    while first.is_some() && first.unwrap() == "" {
        first = instr_val.next();
    }
    let val_c = get_immediate(first.unwrap().trim())?;
    let reg = get_register(split.next().unwrap().trim())?;
    let b: u8 = form_byte(0x0F, reg);
    res.push(b);
    push_le(res, val_c);
    Ok(())
}

fn parse_rr_opq(line: &str, res: &mut Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
    let mut split = line.split(',');
    let instr_reg = split.next().unwrap().trim();
    let mut reg_split = instr_reg.split(' ');
    reg_split.next();
    let mut first = reg_split.next();
    while first.is_some() && first.unwrap() == "" {
        first = reg_split.next();
    }
    let reg_a = get_register(first.unwrap().trim())?;
    let reg_b = get_register(split.next().unwrap().trim())?;
    res.push(form_byte(reg_a, reg_b));
    Ok(())
}
fn parse_mrmovq(line: &str, res: &mut Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
    let mut split = line.split(',');
    let first = split.next().unwrap().trim();
    let mut imm_reg_split = first.split(' ');
    imm_reg_split.next();
    let mut first = imm_reg_split.next();
    while first.is_some() && first.unwrap() == "" {
        first = imm_reg_split.next();
    }
    let mem_brackets = first.unwrap().trim();
    let mut num_reg_b = mem_brackets.split('(');
    let val_c = get_immediate(num_reg_b.next().unwrap().trim())?;
    let mut reg_only = num_reg_b.next().unwrap().split(')');
    let reg_b = get_register(reg_only.next().unwrap().trim())?;
    let reg_a = get_register(split.next().unwrap().trim())?;
    res.push(form_byte(reg_a, reg_b));
    push_le(res, val_c);
    Ok(())
}
fn parse_rmmovq(line: &str, res: &mut Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
    let mut split = line.split(',');
    let first = split.next().unwrap().trim();
    let mut instr_reg_a = first.split(' ');
    instr_reg_a.next();
    let mut first = instr_reg_a.next();
    while first.is_some() && first.unwrap() == "" {
        first = instr_reg_a.next();
    }
    let reg_a = get_register(first.unwrap().trim())?;
    let mem_brackets = split.next().unwrap().trim();
    let mut num_reg_b = mem_brackets.split('(');
    let val_c = get_immediate(num_reg_b.next().unwrap().trim().trim())?;
    let mut reg_only = num_reg_b.next().unwrap().trim().split(')');
    let reg_b = get_register(reg_only.next().unwrap().trim())?;
    res.push(form_byte(reg_a, reg_b));
    push_le(res, val_c);
    Ok(())
}
fn parse_jxx_call(line: &str, res: &mut Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
    let mut split = line.trim().split(' ');
    split.next();
    let mut first = split.next();
    while first.is_some() && first.unwrap() == "" {
        first = split.next();
    }
    let val_c = get_immediate(first.unwrap().trim())?;
    push_le(res, val_c);
    Ok(())
}

fn parse_push_pop(line: &str, res: &mut Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
    let mut split = line.trim().split(' ');
    split.next();
    let mut first = split.next();
    while first.is_some() && first.unwrap() == "" {
        first = split.next();
    }
    let reg_a = get_register(first.unwrap().trim())?;
    res.push(form_byte(reg_a, 0x0F));
    Ok(())
}
