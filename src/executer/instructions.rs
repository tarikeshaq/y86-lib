use super::State;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::error::Error;

const CC_ZERO_MASK: u8 = 0x1;
const CC_SIGN_MASK: u8 = 0x2;

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

pub struct Instruction {
    icode: ICode,
    ifun: u8,
    r_a: Option<Register>,
    r_b: Option<Register>,
    val_c: Option<u64>,
    location: u64,
    val_p: u64,
}

#[derive(Debug, Clone)]
pub struct InvalidICode;

impl std::fmt::Display for InvalidICode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid icode")
    }
}

impl Error for InvalidICode {
    fn description(&self) -> &str {
        "Invalid icode"
    }

    fn cause(&self) -> Option<&dyn Error> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}

impl Instruction {
    pub fn new(state: &State) -> Result<Self, Box<dyn Error>> {
        let icode_ifun = state.read_byte(state.get_pc());
        let icode = (icode_ifun >> 4) & 0x0F;
        match icode {
            code if code == ICode::IHALT as u8 => Self::from_halt(state),
            code if code == ICode::INOP as u8 => Self::from_nop(state),
            code if code == ICode::IRRMVXX as u8 => Self::from_rrmovxx(state),
            code if code == ICode::IMRMOVQ as u8 => Self::from_mrmovq(state),
            code if code == ICode::IRMMOVQ as u8 => Self::from_rmmovq(state),
            code if code == ICode::IIRMOVQ as u8 => Self::from_irmovq(state),
            code if code == ICode::IJXX as u8 => Self::from_jmp(state),
            code if code == ICode::ICALL as u8 => Self::from_call(state),
            code if code == ICode::IRET as u8 => Self::from_ret(state),
            code if code == ICode::IPOPQ as u8 => Self::from_pop(state),
            code if code == ICode::IPUSHQ as u8 => Self::from_push(state),
            code if code == ICode::IOPQ as u8 => Self::from_opq(state),
            _ => Err(InvalidICode.into()),
        }
    }

    pub fn get_location(&self) -> u64 {
        self.location
    }

    pub fn get_val_p(&self) -> u64 {
        self.val_p
    }

    pub fn execute(&self, state: &mut State) -> Result<(), Box<dyn Error>> {
        match self.icode {
            ICode::IHALT => self.execute_halt(state),
            ICode::INOP => self.execute_nop(state),
            ICode::IRRMVXX => self.execute_rrmovxx(state),
            ICode::IMRMOVQ => self.execute_mrmovq(state),
            ICode::IRMMOVQ => self.execute_rmmovq(state),
            ICode::IIRMOVQ => self.execute_irmovq(state),
            ICode::IJXX => self.execute_jump(state),
            ICode::ICALL => self.execute_call(state),
            ICode::IRET => self.execute_ret(state),
            ICode::IPOPQ => self.execute_pop(state),
            ICode::IPUSHQ => self.execute_push(state),
            ICode::IOPQ => self.execute_opq(state),
            ICode::IINVALID => self.execute_invalid(state),
            ICode::ITOOSHORT => self.execute_too_short(state),
        }
    }

    pub fn get_icode(&self) -> ICode {
        self.icode
    }

    pub fn get_ifun(&self) -> u8 {
        self.ifun
    }

    pub fn get_val_c(&self) -> Option<u64> {
        self.val_c
    }

    pub fn get_r_a(&self) -> Option<Register> {
        self.r_a
    }

    pub fn get_r_b(&self) -> Option<Register> {
        self.r_b
    }

    fn get_icode_ifun(state: &State) -> (u8, u8) {
        let icode_ifun = state.read_byte(state.get_pc());
        let icode = icode_ifun >> 4 & 0x0F;
        let ifun = icode_ifun & 0x0F;
        (icode, ifun)
    }

    fn get_registers(state: &State) -> (u8, u8) {
        let ra_rb = state.read_byte(state.get_pc() + 1);
        let ra = ra_rb >> 4 & 0x0F;
        let rb = ra_rb & 0x0F;
        (ra, rb)
    }

    fn cond(ifun: u8, cond_code: u8) -> bool {
        match ifun {
            0 => true,
            1 if (cond_code & CC_ZERO_MASK != 0 || cond_code & CC_SIGN_MASK != 0) => true,
            2 if (cond_code & CC_SIGN_MASK != 0) => true,
            3 if (cond_code & CC_ZERO_MASK != 0) => true,
            4 if (cond_code & CC_ZERO_MASK == 0) => true,
            5 if (cond_code & CC_SIGN_MASK == 0) => true,
            6 if (cond_code & CC_SIGN_MASK == 0 && cond_code & CC_ZERO_MASK == 0) => true,
            _ => false,
        }
    }

    pub fn from_halt(state: &State) -> Result<Self, Box<dyn Error>> {
        let (icode, ifun) = Self::get_icode_ifun(state);
        let val_p = state.get_pc() + 1;
        Ok(Instruction {
            icode: FromPrimitive::from_u8(icode).unwrap(),
            ifun,
            r_a: None,
            r_b: None,
            val_c: None,
            val_p,
            location: state.get_pc(),
        })
    }
    pub fn from_nop(state: &State) -> Result<Self, Box<dyn Error>> {
        let (icode, ifun) = Self::get_icode_ifun(state);
        let val_p = state.get_pc() + 1;
        Ok(Instruction {
            icode: FromPrimitive::from_u8(icode).unwrap(),
            ifun,
            r_a: None,
            r_b: None,
            val_c: None,
            val_p,
            location: state.get_pc(),
        })
    }
    pub fn from_rrmovxx(state: &State) -> Result<Self, Box<dyn Error>> {
        let (icode, ifun) = Self::get_icode_ifun(state);
        let (r_a, r_b) = Self::get_registers(state);
        let val_p = state.get_pc() + 2;
        Ok(Instruction {
            icode: FromPrimitive::from_u8(icode).unwrap(),
            ifun,
            r_a: Some(FromPrimitive::from_u8(r_a).unwrap()),
            r_b: Some(FromPrimitive::from_u8(r_b).unwrap()),
            val_c: None,
            val_p,
            location: state.get_pc(),
        })
    }
    pub fn from_rmmovq(state: &State) -> Result<Self, Box<dyn Error>> {
        let (icode, ifun) = Self::get_icode_ifun(state);
        let (r_a, r_b) = Self::get_registers(state);
        let val_c = state.read_le(state.get_pc() + 2)?;
        let val_p = state.get_pc() + 10;
        Ok(Instruction {
            icode: FromPrimitive::from_u8(icode).unwrap(),
            ifun,
            r_a: Some(FromPrimitive::from_u8(r_a).unwrap()),
            r_b: Some(FromPrimitive::from_u8(r_b).unwrap()),
            val_c: Some(val_c),
            val_p,
            location: state.get_pc(),
        })
    }
    pub fn from_mrmovq(state: &State) -> Result<Self, Box<dyn Error>> {
        let (icode, ifun) = Self::get_icode_ifun(state);
        let (r_a, r_b) = Self::get_registers(state);
        let val_c = state.read_le(state.get_pc() + 2)?;
        let val_p = state.get_pc() + 10;
        Ok(Instruction {
            icode: FromPrimitive::from_u8(icode).unwrap(),
            ifun,
            r_a: Some(FromPrimitive::from_u8(r_a).unwrap()),
            r_b: Some(FromPrimitive::from_u8(r_b).unwrap()),
            val_c: Some(val_c),
            val_p,
            location: state.get_pc(),
        })
    }
    pub fn from_irmovq(state: &State) -> Result<Self, Box<dyn Error>> {
        let (icode, ifun) = Self::get_icode_ifun(state);
        let val_c = state.read_le(state.get_pc() + 2)?;
        let registers = state.read_byte(state.get_pc() + 1);
        let r_a = registers >> 4 & 0x0F;
        let r_b = registers & 0x0F;
        let val_p = state.get_pc() + 10;
        Ok(Instruction {
            icode: FromPrimitive::from_u8(icode).unwrap(),
            ifun,
            r_a: Some(FromPrimitive::from_u8(r_a).unwrap()),
            r_b: Some(FromPrimitive::from_u8(r_b).unwrap()),
            val_c: Some(val_c),
            val_p,
            location: state.get_pc(),
        })
    }
    pub fn from_jmp(state: &State) -> Result<Self, Box<dyn Error>> {
        let (icode, ifun) = Self::get_icode_ifun(state);
        let val_c = state.read_le(state.get_pc() + 1)?;
        let val_p = state.get_pc() + 9;
        Ok(Instruction {
            icode: FromPrimitive::from_u8(icode).unwrap(),
            ifun,
            r_a: None,
            r_b: None,
            val_c: Some(val_c),
            val_p,
            location: state.get_pc(),
        })
    }
    pub fn from_call(state: &State) -> Result<Self, Box<dyn Error>> {
        let (icode, ifun) = Self::get_icode_ifun(state);
        let val_c = state.read_le(state.get_pc() + 1)?;
        let val_p = state.get_pc() + 9;
        Ok(Instruction {
            icode: FromPrimitive::from_u8(icode).unwrap(),
            ifun,
            r_a: None,
            r_b: None,
            val_c: Some(val_c),
            val_p,
            location: state.get_pc(),
        })
    }
    pub fn from_ret(state: &State) -> Result<Self, Box<dyn Error>> {
        let (icode, ifun) = Self::get_icode_ifun(state);
        let val_p = state.get_pc() + 1;
        Ok(Instruction {
            icode: FromPrimitive::from_u8(icode).unwrap(),
            ifun,
            r_a: None,
            r_b: None,
            val_c: None,
            val_p,
            location: state.get_pc(),
        })
    }
    pub fn from_pop(state: &State) -> Result<Self, Box<dyn Error>> {
        let (icode, ifun) = Self::get_icode_ifun(state);
        let (r_a, r_b) = Self::get_registers(state);
        let val_p = state.get_pc() + 2;
        Ok(Instruction {
            icode: FromPrimitive::from_u8(icode).unwrap(),
            ifun,
            r_a: Some(FromPrimitive::from_u8(r_a).unwrap()),
            r_b: Some(FromPrimitive::from_u8(r_b).unwrap()),
            val_c: None,
            val_p,
            location: state.get_pc(),
        })
    }
    pub fn from_push(state: &State) -> Result<Self, Box<dyn Error>> {
        let (icode, ifun) = Self::get_icode_ifun(state);
        let (r_a, r_b) = Self::get_registers(state);
        let val_p = state.get_pc() + 2;
        Ok(Instruction {
            icode: FromPrimitive::from_u8(icode).unwrap(),
            ifun,
            r_a: Some(FromPrimitive::from_u8(r_a).unwrap()),
            r_b: Some(FromPrimitive::from_u8(r_b).unwrap()),
            val_c: None,
            val_p,
            location: state.get_pc(),
        })
    }
    pub fn from_opq(state: &State) -> Result<Self, Box<dyn Error>> {
        let (icode, ifun) = Self::get_icode_ifun(state);
        let (r_a, r_b) = Self::get_registers(state);
        let val_p = state.get_pc() + 2;
        Ok(Instruction {
            icode: FromPrimitive::from_u8(icode).unwrap(),
            ifun,
            r_a: Some(FromPrimitive::from_u8(r_a).unwrap()),
            r_b: Some(FromPrimitive::from_u8(r_b).unwrap()),
            val_c: None,
            val_p,
            location: state.get_pc(),
        })
    }

    pub fn execute_halt(&self, _state: &mut State) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    pub fn execute_nop(&self, state: &mut State) -> Result<(), Box<dyn Error>> {
        state.set_pc(self.val_p);
        Ok(())
    }
    pub fn execute_rrmovxx(&self, state: &mut State) -> Result<(), Box<dyn Error>> {
        let cond_code = state.get_condition_code();
        if Self::cond(self.ifun, cond_code) {
            state.set_register(
                self.get_r_b().unwrap() as u8,
                state.get_register(self.get_r_a().unwrap() as u8),
            );
        }
        state.set_pc(self.val_p);
        Ok(())
    }
    pub fn execute_mrmovq(&self, state: &mut State) -> Result<(), Box<dyn Error>> {
        let address = self.val_c.unwrap() + state.get_register(self.get_r_b().unwrap() as u8);
        let value = state.read_le(address)?;
        state.set_register(self.get_r_a().unwrap() as u8, value);
        state.set_pc(self.val_p);
        Ok(())
    }
    pub fn execute_rmmovq(&self, state: &mut State) -> Result<(), Box<dyn Error>> {
        let address = self.val_c.unwrap() + state.get_register(self.get_r_b().unwrap() as u8);
        state.write_le(address, state.get_register(self.get_r_a().unwrap() as u8))?;
        state.set_pc(self.val_p);
        Ok(())
    }
    pub fn execute_irmovq(&self, state: &mut State) -> Result<(), Box<dyn Error>> {
        state.set_register(self.get_r_b().unwrap() as u8, self.val_c.unwrap());
        state.set_pc(self.val_p);
        Ok(())
    }
    pub fn execute_jump(&self, state: &mut State) -> Result<(), Box<dyn Error>> {
        if Self::cond(self.ifun, state.get_condition_code()) {
            state.set_pc(self.val_c.unwrap());
            Ok(())
        } else {
            state.set_pc(self.val_p);
            Ok(())
        }
    }
    pub fn execute_call(&self, state: &mut State) -> Result<(), Box<dyn Error>> {
        let address = state.get_register(4) - 8;
        state.write_le(address, self.val_p)?;
        state.set_register(4, address);
        state.set_pc(self.val_c.unwrap());
        Ok(())
    }
    pub fn execute_ret(&self, state: &mut State) -> Result<(), Box<dyn Error>> {
        let address = state.get_register(4);
        let value = state.read_le(address)?;
        state.set_register(4, address + 8);
        state.set_pc(value);
        Ok(())
    }
    pub fn execute_pop(&self, state: &mut State) -> Result<(), Box<dyn Error>> {
        let address = state.get_register(4);
        let value = state.read_le(address)?;
        state.set_register(4, address + 8);
        state.set_register(self.get_r_a().unwrap() as u8, value);
        state.set_pc(self.val_p);
        Ok(())
    }
    pub fn execute_push(&self, state: &mut State) -> Result<(), Box<dyn Error>> {
        let address = state.get_register(4) - 8;
        state.write_le(address, state.get_register(self.get_r_a().unwrap() as u8))?;
        state.set_register(4, address);
        state.set_pc(self.val_p);
        Ok(())
    }
    pub fn execute_opq(&self, state: &mut State) -> Result<(), Box<dyn Error>> {
        let ra_val = state.get_register(self.get_r_a().unwrap() as u8) as i64;
        let rb_val = state.get_register(self.get_r_b().unwrap() as u8) as i64;
        let res: i64 = match self.ifun {
            0 => ra_val + rb_val,
            1 => rb_val - ra_val,
            2 => rb_val & ra_val,
            3 => rb_val ^ ra_val,
            4 => rb_val * ra_val,
            5 => rb_val / ra_val,
            6 => rb_val % ra_val,
            _ => 0,
        };
        if res == 0 {
            state.set_condition_code(CC_ZERO_MASK);
        } else if res < 0 {
            state.set_condition_code(CC_SIGN_MASK);
        } else {
            state.set_condition_code(0);
        }
        state.set_register(self.get_r_b().unwrap() as u8, res as u64);
        state.set_pc(self.get_val_p());
        Ok(())
    }
    pub fn execute_invalid(&self, _state: &mut State) -> Result<(), Box<dyn Error>> {
        unimplemented!("")
    }
    pub fn execute_too_short(&self, _state: &mut State) -> Result<(), Box<dyn Error>> {
        unimplemented!("")
    }
}
