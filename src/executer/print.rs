use super::instructions::{ICode, Instruction, Register};
use super::State;
use lazy_static::lazy_static;
use num_traits::FromPrimitive;
use std::collections::HashMap;

lazy_static! {
    static ref MAP: HashMap<u8, &'static str> = vec![
        ((ICode::IHALT as u8) << 4, "halt"),
        ((ICode::INOP as u8) << 4, "nop"),
        ((ICode::IRRMVXX as u8) << 4 | 0, "rrmovq"),
        ((ICode::IRRMVXX as u8) << 4 | 1, "cmovle"),
        ((ICode::IRRMVXX as u8) << 4 | 2, "cmovl"),
        ((ICode::IRRMVXX as u8) << 4 | 3, "cmove"),
        ((ICode::IRRMVXX as u8) << 4 | 4, "cmovne"),
        ((ICode::IRRMVXX as u8) << 4 | 5, "cmovge"),
        ((ICode::IRRMVXX as u8) << 4 | 6, "cmovg"),
        ((ICode::IRMMOVQ as u8) << 4, "rmmovq"),
        ((ICode::IMRMOVQ as u8) << 4, "mrmovq"),
        ((ICode::IIRMOVQ as u8) << 4, "irmovq"),
        ((ICode::IOPQ as u8) << 4, "addq"),
        ((ICode::IOPQ as u8) << 4 | 1, "subq"),
        ((ICode::IOPQ as u8) << 4 | 2, "andq"),
        ((ICode::IOPQ as u8) << 4 | 3, "xorq"),
        ((ICode::IOPQ as u8) << 4 | 4, "mulq"),
        ((ICode::IOPQ as u8) << 4 | 5, "divq"),
        ((ICode::IOPQ as u8) << 4 | 6, "modq"),
        ((ICode::IJXX as u8) << 4, "jmp"),
        ((ICode::IJXX as u8) << 4 | 1, "jle"),
        ((ICode::IJXX as u8) << 4 | 2, "jl"),
        ((ICode::IJXX as u8) << 4 | 3, "je"),
        ((ICode::IJXX as u8) << 4 | 4, "jne"),
        ((ICode::IJXX as u8) << 4 | 5, "jge"),
        ((ICode::IJXX as u8) << 4 | 6, "jg"),
        ((ICode::ICALL as u8) << 4, "call"),
        ((ICode::IRET as u8) << 4, "ret"),
        ((ICode::IPUSHQ as u8) << 4, "pushq"),
        ((ICode::IPOPQ as u8) << 4, "popq")
    ]
    .into_iter()
    .collect();
}

pub fn print_register(register: Register) -> &'static str {
    match register {
        Register::RRAX => "%rax",
        Register::RRCX => "%rcx",
        Register::RRDX => "%rdx",
        Register::RRBX => "%rbx",
        Register::RRSP => "%rsp",
        Register::RRBP => "%rbp",
        Register::RRSI => "%rsi",
        Register::RRDI => "%rdi",
        Register::RR8 => "%r8",
        Register::RR9 => "%r9",
        Register::RR10 => "%r10",
        Register::RR11 => "%r11",
        Register::RR12 => "%r12",
        Register::RR13 => "%r13",
        Register::RR14 => "%r14",
        Register::RNONE => "WAT",
    }
}

pub fn print_instruction(instr: &Instruction) {
    let code = instr.get_icode();
    let ifun = instr.get_ifun();
    let icode_ifun = (code as u8) << 4 | ifun;
    let mut curr = std::format!("    {:}", MAP.get(&icode_ifun).unwrap()); // Remove unwrap
    match code {
        ICode::IIRMOVQ => {
            curr.push_str(&std::format!(
                " $0x{:x}, {:}",
                instr.get_val_c().unwrap(),
                print_register(instr.get_r_b().unwrap())
            ));
        }
        ICode::IPUSHQ | ICode::IPOPQ => curr.push_str(&std::format!(
            " {:}",
            print_register(instr.get_r_a().unwrap())
        )),
        ICode::IJXX | ICode::ICALL => {
            curr.push_str(&std::format!(" 0x{:x}", instr.get_val_c().unwrap()))
        }
        ICode::IRMMOVQ => curr.push_str(&std::format!(
            " {:}, 0x{:x}({:})",
            print_register(instr.get_r_a().unwrap()),
            instr.get_val_c().unwrap(),
            print_register(instr.get_r_b().unwrap())
        )),
        ICode::IMRMOVQ => curr.push_str(&std::format!(
            " 0x{:x}({:}), {:}",
            instr.get_val_c().unwrap(),
            print_register(instr.get_r_b().unwrap()),
            print_register(instr.get_r_a().unwrap())
        )),
        ICode::IRRMVXX | ICode::IOPQ => curr.push_str(&std::format!(
            " {:}, {:}",
            print_register(instr.get_r_a().unwrap()),
            print_register(instr.get_r_b().unwrap())
        )),
        _ => (),
    }
    curr.push_str(&std::format!("   #PC = 0x{:x}", instr.get_location()));
    println!("{:}", curr);
}

pub fn print_all_registers(state: &State) {
    (0..14)
        .into_iter()
        .for_each(|id| print_register_val(state, id));
}

pub fn print_memory_quad_value(state: &State, address: u64) {
    println!(
        "      #M_8[0x{:x}]  = 0x{:x}",
        address,
        state.read_le(address).unwrap()
    );
}

pub fn print_register_val(state: &State, val: u8) {
    println!(
        "       #R[{:}] = 0x{:x}",
        print_register(FromPrimitive::from_u8(val).unwrap()),
        state.get_register(val)
    );
}
