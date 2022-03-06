//! TODO:
//! - Right tests to make sure that the register pair instructions are working
//!   (I'm not sure whether I've done the endianness correctly)

mod cpu;
mod instruction;

#[cfg(test)]
mod tests;

pub use cpu::Error;

pub use cpu::Cpu;

pub use instruction::Instruction;

pub fn disassemble(bin: Vec<u8>) {
    let mut position = 0;

    while position < bin.len() {
        let instr = Instruction::decode(&bin[position..]);
        if let Some(instr) = instr {
            println!("{:#06x} {}", position, instr);
            position += instr.op_bytes() as usize;
        } else {
            println!("{:#06x} NOP", position);
            position += 1;
        }
    }
}
