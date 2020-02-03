/// Y86 Assembler, can be used to generate the machine code
/// associated with a Y86 file
pub mod assembler;

/// More accurately a debugger
/// Can execute Y86 machine code, can also step through
/// instructions and set breakpoints
pub mod executer;

///Simple number parser, can parse hex and decimal values
pub mod number_parser;