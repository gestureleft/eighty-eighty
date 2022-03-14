use crate::{instruction::Reg, Instruction};

#[derive(Debug)]
pub enum Error {
    OutOfMemory,
    BadMemoryAccess(u16),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ConditionCodes {
    // Zero Flag
    pub(crate) z: u8,
    // Negative Flag
    pub(crate) s: u8,
    // Parity Flag
    pub(crate) p: u8,
    // Carry Flag
    pub(crate) cy: u8,
    // Half Carry Flag
    pub(crate) ac: u8,
}

impl ConditionCodes {
    pub(crate) fn new() -> Self {
        Self {
            z: 0,
            s: 0,
            p: 0,
            cy: 0,
            ac: 0,
        }
    }
}

const MEMORY_SIZE: usize = 65_536;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cpu<BusWriteCallBack>
where
    BusWriteCallBack: FnMut(u8),
{
    pub(crate) a: u8,
    pub(crate) b: u8,
    pub(crate) c: u8,
    pub(crate) d: u8,
    pub(crate) e: u8,
    pub(crate) h: u8,
    pub(crate) l: u8,
    pub(crate) pc: u16,
    pub(crate) sp: u16,
    pub(crate) memory: [u8; MEMORY_SIZE],
    pub(crate) condition_codes: ConditionCodes,
    int_enable: u8,
    on_bus_write: BusWriteCallBack,
    bus: u8,
    halted: bool,
}

impl<T: FnMut(u8)> std::fmt::Display for Cpu<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "pc: {:#06x}, sp: {:#06x}, a: {:#04x}, h: {:#06x}, l: {:#06x}, b: {:#04x}",
            self.pc, self.sp, self.a, self.h, self.l, self.b
        )
    }
}

impl<T: FnMut(u8)> Cpu<T> {
    pub fn new(on_bus_write: T) -> Self {
        Self {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            pc: 0,
            sp: 0,
            memory: [0; MEMORY_SIZE],
            condition_codes: ConditionCodes::new(),
            int_enable: 1,
            on_bus_write,
            bus: 0,
            halted: false,
        }
    }

    pub fn memory(&self) -> &[u8] {
        &self.memory
    }

    pub fn a(&self) -> u8 {
        self.a
    }

    pub fn b(&self) -> u8 {
        self.b
    }

    pub fn c(&self) -> u8 {
        self.c
    }

    pub fn d(&self) -> u8 {
        self.d
    }

    pub fn e(&self) -> u8 {
        self.e
    }

    pub fn h(&self) -> u8 {
        self.h
    }

    pub fn l(&self) -> u8 {
        self.l
    }

    pub fn pc(&self) -> u16 {
        self.pc
    }

    pub fn sp(&self) -> u16 {
        self.sp
    }

    pub fn halted(&self) -> bool {
        self.halted
    }

    pub fn write_to_bus(&mut self, value: u8) {
        self.bus = value;
    }

    pub fn generate_interrupt(&mut self, value: u8) -> Result<(), Error> {
        if self.int_enable == 9 || self.halted {
            return Ok(());
        }

        let instruction = Instruction::RST { data: value };

        self.execute_instruction(instruction)?;
        self.pc += instruction.op_bytes() as u16;

        Ok(())
    }

    pub(crate) fn assign_value(&mut self, reg: Reg, val: u8) {
        match reg {
            Reg::A => self.a = val,
            Reg::B => self.b = val,
            Reg::C => self.c = val,
            Reg::D => self.d = val,
            Reg::E => self.e = val,
            Reg::SP => todo!(),
            Reg::H => self.h = val,
            Reg::L => self.l = val,
            Reg::Psw => todo!(),
            Reg::M => todo!(),
        }
    }

    pub(crate) fn get_register_val(&self, reg: Reg) -> u8 {
        match reg {
            Reg::A => self.a,
            Reg::B => self.b,
            Reg::C => self.c,
            Reg::D => self.d,
            Reg::E => self.e,
            Reg::SP => todo!(),
            Reg::H => self.h,
            Reg::L => self.l,
            Reg::Psw => todo!(),
            Reg::M => todo!(),
        }
    }

    pub fn load_into_memory(&mut self, data: Vec<u8>) -> Result<(), Error> {
        let data_len = data.len();

        if data_len > self.memory.len() {
            return Err(Error::OutOfMemory);
        }

        for (&element, memory_ptr) in data.iter().zip(self.memory.iter_mut()) {
            *memory_ptr = element;
        }

        Ok(())
    }

    pub fn step(&mut self) -> Result<(), Error> {
        if self.halted {
            return Ok(());
        };
        if let Some(instruction) = self.fetch_instruction() {
            self.execute_instruction(instruction)?;
            self.pc += instruction.op_bytes() as u16;
        }
        Ok(())
    }

    pub(crate) fn fetch_instruction(&self) -> Option<Instruction> {
        Instruction::decode(&self.memory[self.pc.into()..])
    }

    pub(crate) fn execute_instruction(&mut self, instruction: Instruction) -> Result<(), Error> {
        match instruction {
            Instruction::NoOp => (),
            Instruction::LXI { register, value } => {
                self.set_register_pair(register, value);
            }
            Instruction::STAX { register } => {
                let val = self.a;
                let address = self.load_register_pair(register);
                self.write_to_memory_at(address, val)?;
            }
            Instruction::INX { register } => {
                let val = self.load_register_pair(register).wrapping_add(1);
                self.set_register_pair(register, val);
            }
            Instruction::INR { register } => {
                let res = if register == Reg::M {
                    let res = self.load_from_memory()?.wrapping_add(1);
                    self.write_to_memory(res);
                    res
                } else {
                    let res = self.get_register_val(register).wrapping_add(1);
                    self.assign_value(register, res);
                    res
                };
                // We don't update cy
                self.condition_codes.z = (res == 0).into();
                self.condition_codes.s = ((res & 0x80) != 0).into();
                self.update_parity(res);
            }
            Instruction::DCR { register } => {
                let res = if register == Reg::M {
                    let res = self.load_from_memory()?.wrapping_sub(1);
                    self.write_to_memory(res);
                    res
                } else {
                    let res = self.get_register_val(register).wrapping_sub(1);
                    self.assign_value(register, res);
                    res
                };
                // We don't update cy
                self.condition_codes.z = (res == 0).into();
                self.condition_codes.s = ((res & 0x80) != 0).into();
                self.update_parity(res);
            }
            Instruction::MVI { register, value } => {
                if register == Reg::M {
                    self.write_to_memory(value);
                } else {
                    self.assign_value(register, value);
                }
            }
            Instruction::RLC => {
                self.a = self.a.rotate_left(1);
                self.condition_codes.cy = self.a & 1;
            }
            Instruction::DAD { register } => {
                let result = self.load_register_pair(Reg::H) as u32
                    + self.load_register_pair(register) as u32;

                self.set_register_pair(Reg::H, (result & 0xffff) as u16);

                self.condition_codes.cy = if result > u16::MAX.into() { 1 } else { 0 };
            }
            Instruction::LDAX { register } => {
                // (A) <- ((rp))
                let address = self.load_register_pair(register);
                self.a = self.load_from_memory_at(address)?;
            }
            Instruction::DCX { register } => {
                let val = self.load_register_pair(register).wrapping_sub(1);
                self.set_register_pair(register, val);
            }
            Instruction::RRC => {
                self.condition_codes.cy = self.a & 1;
                self.a = self.a.rotate_right(1);
            }
            Instruction::RAL => todo!(),
            Instruction::RAR => todo!(),
            Instruction::SHLD { address } => todo!("{}", address),
            Instruction::CMA => todo!(),
            Instruction::DAA => todo!(),
            Instruction::LHLD { address } => todo!("{}", address),
            Instruction::STA { address } => todo!("{}", address),
            Instruction::STC => todo!(),
            Instruction::LDA { address } => todo!("{}", address),
            Instruction::CMC => todo!(),
            Instruction::MOV {
                source,
                destination,
            } => {
                if source == Reg::M {
                    let address = self.load_register_pair(Reg::H);
                    self.assign_value(destination, self.load_from_memory_at(address)?)
                } else if destination == Reg::M {
                    let address = self.load_register_pair(Reg::H);
                    self.write_to_memory_at(address, self.get_register_val(source))?;
                } else {
                    self.assign_value(destination, self.get_register_val(source));
                }
            }
            Instruction::HLT => {
                self.halted = true;
            }
            Instruction::ADD { register } => {
                let (res, overflow) = self
                    .load_from_memory_or_register(register)?
                    .overflowing_add(self.a);

                self.update_condition_codes(res, overflow);

                self.a = res;
            }
            Instruction::ADC { register } => {
                let (res, overflow) = self
                    .load_from_memory_or_register(register)?
                    .overflowing_add(self.a);
                let (res, overflow_two) = res.overflowing_add(self.condition_codes.cy);

                self.update_condition_codes(res, overflow || overflow_two);

                self.a = res;
            }
            Instruction::SUB { register } => {
                let (res, overflow) = self
                    .a
                    .overflowing_sub(self.load_from_memory_or_register(register)?);

                self.update_condition_codes(res, overflow);

                self.a = res;
            }
            Instruction::SBB { register } => {
                let (res, overflow) = self
                    .a
                    .overflowing_sub(self.load_from_memory_or_register(register)?);
                let (res, overflow_two) = res.overflowing_sub(self.condition_codes.cy);

                self.update_condition_codes(res, overflow || overflow_two);

                self.a = res;
            }
            Instruction::ANA { register } => todo!("{}", register),
            Instruction::XRA { register } => todo!("{}", register),
            Instruction::ORA { register } => todo!("{}", register),
            Instruction::CMP { register } => todo!("{}", register),
            Instruction::RNZ => todo!(),
            Instruction::POP { register } => {
                if register == Reg::Psw {
                    self.write_processor_status_word(self.load_from_memory_at(self.sp)?);
                    self.a = self.load_from_memory_at(self.sp.wrapping_add(1))?;
                } else {
                    let low = self.load_from_memory_at(self.sp)?;
                    let high = self.load_from_memory_at(self.sp + 1)?;
                    self.set_register_pair(register, ((high as u16) << 8) | low as u16);
                }
                self.sp = self.sp.wrapping_add(2);
            }
            Instruction::JNZ { address } => {
                if 0 == self.condition_codes.z {
                    self.pc = address - instruction.op_bytes() as u16;
                }
            }
            Instruction::JMP { address } => {
                self.pc = address - instruction.op_bytes() as u16;
            }
            Instruction::CNZ { address } => todo!("{}", address),
            Instruction::PUSH { register } => {
                if register == Reg::Psw {
                    self.write_to_memory_at(self.sp.wrapping_sub(1), self.a)?;
                    self.write_to_memory_at(self.sp.wrapping_sub(2), self.processor_status_word())?;
                } else {
                    let value = self.load_register_pair(register);
                    self.write_to_memory_at(self.sp.wrapping_sub(1), ((value >> 8) & 0xff) as u8)?;
                    self.write_to_memory_at(self.sp.wrapping_sub(2), (value & 0xff) as u8)?;
                }
                self.sp = self.sp.wrapping_sub(2);
            }
            Instruction::ADI { data } => {
                let (res, overflow) = self.a.overflowing_add(data);

                self.update_condition_codes(res, overflow);

                self.a = res;
            }
            Instruction::RST { data } => {
                self.write_to_memory_at(self.sp.wrapping_sub(1), ((self.pc >> 8) & 0xff) as u8)?;
                self.write_to_memory_at(self.sp.wrapping_sub(2), (self.pc & 0x00FF) as u8)?;
                self.sp = self.sp.wrapping_sub(2);
                self.pc = ((8 * data) as u16).wrapping_sub(instruction.op_bytes() as u16);
            }
            Instruction::RZ => todo!(),
            Instruction::RET => {
                self.pc = self.memory[self.sp as usize] as u16
                    | ((self.memory[(self.sp.wrapping_add(1)) as usize] as u16) << 8);
                self.sp = self.sp.wrapping_add(2);
            }
            Instruction::JZ { address } => {
                if 0 != self.condition_codes.z {
                    self.pc = address - instruction.op_bytes() as u16;
                }
            }
            Instruction::CZ { address } => todo!("{}", address),
            Instruction::CALL { address } => {
                let ret = self.pc + 2;
                self.write_to_memory_at(self.sp.wrapping_sub(1), ((ret >> 8) & 0xff) as u8)?;
                self.write_to_memory_at(self.sp.wrapping_sub(2), (ret & 0xff) as u8)?;
                self.sp = self.sp.wrapping_sub(2);
                self.pc = address.wrapping_sub(instruction.op_bytes().into()) as u16;
            }
            Instruction::ACI { data } => {
                let (res, overflow) = self.a.overflowing_add(data);
                let (res, overflow_two) = res.overflowing_add(self.condition_codes.cy);

                self.update_condition_codes(res, overflow || overflow_two);

                self.a = res;
            }
            Instruction::RNC => todo!(),
            Instruction::JNC { address } => {
                if 0 == self.condition_codes.cy {
                    self.pc = address - instruction.op_bytes() as u16;
                }
            }
            Instruction::OUT { data } => (self.on_bus_write)(data),
            Instruction::CNC { address } => todo!("{}", address),
            Instruction::SUI { data } => {
                let (res, overflow) = self.a.overflowing_sub(data);

                self.update_condition_codes(res, overflow);

                self.a = res;
            }
            Instruction::RC => todo!(),
            Instruction::JC { address } => {
                if 0 != self.condition_codes.cy {
                    self.pc = address - instruction.op_bytes() as u16;
                }
            }
            Instruction::IN { data } => todo!("{}", data),
            Instruction::CC { address } => todo!("{}", address),
            Instruction::SBI { data } => {
                let (res, overflow) = self.a.overflowing_sub(data);
                let (res, overflow_two) = res.overflowing_sub(self.condition_codes.cy);

                self.update_condition_codes(res, overflow || overflow_two);

                self.a = res;
            }
            Instruction::RPO => todo!(),
            Instruction::JPO { address } => todo!("{}", address),
            Instruction::XTHL => todo!(),
            Instruction::CPO { address } => todo!("{}", address),
            Instruction::ANI { data } => todo!("{}", data),
            Instruction::RPE => {
                if self.condition_codes.p == 1 {
                    self.execute_instruction(Instruction::RET)?;
                }
            }
            Instruction::PCHL => todo!(),
            Instruction::JPE { address } => todo!("{}", address),
            Instruction::XCHG => {
                let h = self.load_from_memory_or_register(Reg::H)?;
                let l = self.load_from_memory_or_register(Reg::L)?;
                let d = self.load_from_memory_or_register(Reg::D)?;
                let e = self.load_from_memory_or_register(Reg::E)?;
                self.h = d;
                self.d = h;
                self.l = e;
                self.e = l;
            }
            Instruction::CPE { address } => todo!("{}", address),
            Instruction::XRI { data } => todo!("{}", data),
            Instruction::RP => todo!(),
            Instruction::JP { address } => todo!("{}", address),
            Instruction::DI => todo!(),
            Instruction::CP { address } => todo!("{}", address),
            Instruction::ORI { data } => todo!("{}", data),
            Instruction::RM => todo!(),
            Instruction::SPHL => todo!(),
            Instruction::JM { address } => todo!("{}", address),
            Instruction::EI => todo!(),
            Instruction::CM { address } => todo!("{}", address),
            Instruction::CPI { data } => {
                let result = self.a.wrapping_sub(data);
                self.condition_codes.z = (result == 0) as u8;
                self.condition_codes.s = (0x80 == (result & 0x80)) as u8;
                self.update_parity(result);
                self.condition_codes.cy = (self.a < data) as u8;
            }
        }
        Ok(())
    }

    fn update_condition_codes(&mut self, value: u8, overflow: bool) {
        self.condition_codes.z = (value == 0).into();
        self.condition_codes.s = ((value & 0x80) != 0).into();
        self.update_parity(value);

        self.condition_codes.cy = if overflow { 1 } else { 0 };
    }

    pub(crate) fn processor_status_word(&self) -> u8 {
        self.condition_codes.s << 7
            | (self.condition_codes.z << 6)
            | (self.condition_codes.ac << 4)
            | (self.condition_codes.p << 2)
            | (1 << 1)
            | self.condition_codes.cy
    }

    fn write_processor_status_word(&mut self, processor_status_word: u8) {
        self.condition_codes.cy = processor_status_word & 0b1;
        self.condition_codes.p = (processor_status_word & 0b100) >> 2;
        self.condition_codes.ac = (processor_status_word & 0b10000) >> 4;
        self.condition_codes.z = (processor_status_word & 0b1000000) >> 6;
        self.condition_codes.s = (processor_status_word & 0b10000000) >> 7;
    }

    fn load_from_memory(&self) -> Result<u8, Error> {
        self.memory
            .iter()
            .nth((((self.h as u16) << 8) | self.l as u16).into())
            .copied()
            .ok_or_else(|| Error::BadMemoryAccess((((self.h as u16) << 8) as u8 | self.l).into()))
    }

    fn load_from_memory_at(&self, address: u16) -> Result<u8, Error> {
        self.memory
            .iter()
            .nth(address as usize)
            .copied()
            .ok_or(Error::BadMemoryAccess(address))
    }

    fn write_to_memory(&mut self, val: u8) {
        let dest = self
            .memory
            .iter_mut()
            .nth((((self.h as u16) << 8) | self.l as u16).into())
            .unwrap();

        *dest = val;
    }

    /// Write the given value to the given address
    fn write_to_memory_at(&mut self, address: u16, val: u8) -> Result<(), Error> {
        let dest = self
            .memory
            .iter_mut()
            .nth(address as usize)
            .ok_or(Error::BadMemoryAccess(address))?;
        *dest = val;
        Ok(())
    }

    fn load_from_memory_or_register(&self, memory_or_register: Reg) -> Result<u8, Error> {
        if memory_or_register == Reg::M {
            self.load_from_memory()
        } else {
            Ok(self.get_register_val(memory_or_register))
        }
    }

    fn update_parity<Prim: Parity>(&mut self, val: Prim) {
        self.condition_codes.p = if val.parity() { 1 } else { 0 };
    }

    // TODO - Make this return a `Result` so we don't have to panic
    fn load_register_pair(&self, reg: Reg) -> u16 {
        match reg {
            Reg::A => panic!("Invalid Arg: Called `Cpu.load_register_pair` with Reg::A"),
            Reg::B => ((self.b as u16) << 8) + self.c as u16,
            Reg::C => panic!("Invalid Arg: Called `Cpu.load_register_pair` with Reg::C"),
            Reg::D => ((self.d as u16) << 8) + self.e as u16,
            Reg::E => panic!("Invalid Arg: Called `Cpu.load_register_pair` with Reg::E"),
            Reg::SP => self.sp,
            Reg::H => ((self.h as u16) << 8) + self.l as u16,
            Reg::L => panic!("Invalid Arg: Called `Cpu.load_register_pair` with Reg::L"),
            Reg::Psw => panic!("Invalid Arg: Called `Cpu.load_register_pair` with Reg::Psw"),
            Reg::M => panic!("Invalid Arg: Called `Cpu.load_register_pair` with Reg::M"),
        }
    }

    fn set_register_pair(&mut self, reg: Reg, val: u16) {
        match reg {
            Reg::A => todo!(),
            Reg::B => {
                self.b = (val >> 8) as u8;
                self.c = (val & 0xff) as u8;
            }
            Reg::C => todo!(),
            Reg::D => {
                self.d = (val >> 8) as u8;
                self.e = (val & 0xff) as u8;
            }
            Reg::E => todo!(),
            Reg::SP => self.sp = val,
            Reg::H => {
                self.h = (val >> 8) as u8;
                self.l = (val & 0xff) as u8;
            }
            Reg::L => todo!(),
            Reg::Psw => todo!(),
            Reg::M => todo!(),
        }
    }
}

pub(crate) trait Parity {
    fn parity(self) -> bool;
}

fn parity_impl(input: u16, size: u32) -> bool {
    let mut high_bits = 0;
    for i in 0..size {
        let val = input.rotate_right(i);
        if val & 0b1 == 1 {
            high_bits += 1;
        }
    }
    high_bits % 2 == 0
}

impl Parity for u8 {
    fn parity(self) -> bool {
        parity_impl(self.into(), 8)
    }
}

impl Parity for u16 {
    fn parity(self) -> bool {
        parity_impl(self, 16)
    }
}
