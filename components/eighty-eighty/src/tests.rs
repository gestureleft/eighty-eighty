use crate::cpu::{self, Cpu};
use crate::instruction::Reg;
use crate::Instruction;

#[test]
fn parity() {
    use crate::cpu::Parity;
    assert!(0b00_u8.parity());
    assert!(!0b01_u8.parity());
    assert!(0b11_u8.parity());
    assert!(!0b10_u8.parity());

    assert!(0b101_u8.parity());
    assert!(!0b111_u8.parity());
    assert!(0b110_u8.parity());

    assert!(0b0000000000000000_u16.parity());
    assert!(!0b1000000000000000_u16.parity());
    assert!(!0b0000000000000001_u16.parity());
    assert!(0b0000100000000001_u16.parity());
    assert!(0b0000111100000000_u16.parity());
}

#[test]
fn processor_status_word() {
    let mut cpu = Cpu::new(|_| {});

    assert_eq!(cpu.processor_status_word(), 0b01000000);

    cpu.condition_codes.z = 1;

    assert_eq!(cpu.processor_status_word(), 0b01000010);

    cpu.condition_codes.s = 1;

    assert_eq!(cpu.processor_status_word(), 0b01000011);

    cpu.condition_codes.z = 0;

    assert_eq!(cpu.processor_status_word(), 0b01000001);

    cpu.condition_codes.s = 0;
    cpu.condition_codes.cy = 1;

    assert_eq!(cpu.processor_status_word(), 0b11000000);
}

#[test]
fn mvi_and_add() -> Result<(), cpu::Error> {
    let mut cpu = Cpu::new(|_| {});

    assert_eq!(cpu.a, 0);

    cpu.execute_instruction(Instruction::MVI {
        register: Reg::B,
        value: 20,
    })?;

    cpu.execute_instruction(Instruction::ADD {
        register: crate::instruction::Reg::B,
    })?;

    assert_eq!(cpu.a, 20);

    Ok(())
}

// [LXI] - Load Register Pair Immediate
#[test]
fn lxi() -> Result<(), cpu::Error> {
    let mut cpu = Cpu::new(|_| {});

    cpu.execute_instruction(Instruction::LXI {
        register: Reg::B,
        value: 0x0055,
    })?;

    assert_eq!(cpu.b, 0x00);
    assert_eq!(cpu.c, 0x55);

    cpu.execute_instruction(Instruction::LXI {
        register: Reg::B,
        value: 0x1200,
    })?;

    assert_eq!(cpu.b, 0x12);
    assert_eq!(cpu.c, 0x00);

    cpu.execute_instruction(Instruction::LXI {
        register: Reg::D,
        value: 0x1234,
    })?;

    assert_eq!(cpu.b, 0x12);
    assert_eq!(cpu.c, 0x00);
    assert_eq!(cpu.d, 0x12);
    assert_eq!(cpu.e, 0x34);

    Ok(())
}

// [STAX] - Store Accumulator Indirect
#[test]
fn stax() -> Result<(), cpu::Error> {
    let mut cpu = Cpu::new(|_| {});

    cpu.a = 0x34;

    cpu.execute_instruction(Instruction::LXI {
        register: Reg::B,
        value: 0xabcd,
    })?;

    cpu.execute_instruction(Instruction::STAX { register: Reg::B })?;

    assert_eq!(cpu.memory[0xabcd], 0x34);

    cpu.a = 0x12;

    cpu.execute_instruction(Instruction::LXI {
        register: Reg::D,
        value: 0x7831,
    })?;

    cpu.execute_instruction(Instruction::STAX { register: Reg::D })?;

    assert_eq!(cpu.memory[0xabcd], 0x34);
    assert_eq!(cpu.memory[0x7831], 0x12);

    Ok(())
}

// [INX] - Increment Register Pair
#[test]
fn inx() -> Result<(), cpu::Error> {
    let mut cpu = Cpu::new(|_| {});

    cpu.condition_codes.p = 12;
    cpu.condition_codes.z = 13;
    cpu.condition_codes.s = 14;
    cpu.condition_codes.cy = 16;
    cpu.condition_codes.ac = 17;
    cpu.condition_codes.pad = 18;

    // Test for proper wrapping
    cpu.execute_instruction(Instruction::LXI {
        register: Reg::B,
        value: 0xffff,
    })?;

    cpu.execute_instruction(Instruction::INX { register: Reg::B })?;

    assert_eq!(cpu.b, 0x00);
    assert_eq!(cpu.c, 0x00);

    assert_eq!(cpu.condition_codes.p, 12);
    assert_eq!(cpu.condition_codes.z, 13);
    assert_eq!(cpu.condition_codes.s, 14);
    assert_eq!(cpu.condition_codes.cy, 16);
    assert_eq!(cpu.condition_codes.ac, 17);
    assert_eq!(cpu.condition_codes.pad, 18);

    cpu.execute_instruction(Instruction::LXI {
        register: Reg::B,
        value: 0xfffe,
    })?;

    cpu.execute_instruction(Instruction::INX { register: Reg::B })?;

    assert_eq!(cpu.b, 0xff);
    assert_eq!(cpu.c, 0xff);

    assert_eq!(cpu.condition_codes.p, 12);
    assert_eq!(cpu.condition_codes.z, 13);
    assert_eq!(cpu.condition_codes.s, 14);
    assert_eq!(cpu.condition_codes.cy, 16);
    assert_eq!(cpu.condition_codes.ac, 17);
    assert_eq!(cpu.condition_codes.pad, 18);

    Ok(())
}

// [INR] - Increment Register
#[test]
fn inr_register() -> Result<(), cpu::Error> {
    let mut cpu = Cpu::new(|_| {});

    cpu.execute_instruction(Instruction::INR { register: Reg::A })?;

    assert_eq!(cpu.a, 0x01);
    assert_eq!(cpu.condition_codes.cy, 0);

    cpu.execute_instruction(Instruction::MVI {
        register: Reg::A,
        value: 0xff,
    })?;

    assert_eq!(cpu.a, 0xff);
    assert_eq!(cpu.condition_codes.cy, 0);

    cpu.execute_instruction(Instruction::INR { register: Reg::A })?;

    assert_eq!(cpu.a, 0x00);
    assert_eq!(cpu.condition_codes.cy, 0);

    Ok(())
}

// [INR] - Increment Memory
#[test]
fn inr_memory() -> Result<(), cpu::Error> {
    let mut cpu = Cpu::new(|_| {});

    cpu.execute_instruction(Instruction::INR { register: Reg::M })?;

    assert_eq!(cpu.memory[0], 1);

    cpu.execute_instruction(Instruction::LXI {
        register: Reg::H,
        value: 0x1234,
    })?;

    assert_eq!(cpu.h, 0x12);
    assert_eq!(cpu.l, 0x34);

    cpu.execute_instruction(Instruction::INR { register: Reg::M })?;

    assert_eq!(cpu.memory[0x1234], 1);

    Ok(())
}

// [DCR] - Decrement Register
#[test]
fn dcr_register() -> Result<(), cpu::Error> {
    let mut cpu = Cpu::new(|_| {});

    cpu.execute_instruction(Instruction::DCR { register: Reg::B })?;

    assert_eq!(cpu.b, 0xff);
    assert_eq!(cpu.condition_codes.cy, 0);
    assert_eq!(cpu.condition_codes.s, 1);
    assert_eq!(cpu.condition_codes.p, 1);

    cpu.execute_instruction(Instruction::DCR { register: Reg::B })?;

    assert_eq!(cpu.b, 0xfe);
    assert_eq!(cpu.condition_codes.cy, 0);
    assert_eq!(cpu.condition_codes.s, 1);
    assert_eq!(cpu.condition_codes.p, 0);

    Ok(())
}

// [DCR] - Decrement Memory
#[test]
fn dcr_memory() -> Result<(), cpu::Error> {
    let mut cpu = Cpu::new(|_| {});

    cpu.load_into_memory(vec![0x35])?;

    assert_eq!(cpu.memory[0], 0x35);
    assert_eq!(cpu.pc, 0);

    cpu.step()?;

    assert_eq!(cpu.memory[0], 0x34);
    assert_eq!(cpu.pc, 1);

    Ok(())
}

// [MVI] - Move Immediate Register
#[test]
fn mvi_reg() -> Result<(), cpu::Error> {
    let mut cpu = Cpu::new(|_| {});

    cpu.execute_instruction(Instruction::MVI {
        register: Reg::B,
        value: 0x12,
    })?;

    assert_eq!(cpu.b, 0x12);

    Ok(())
}

// [MVI] - Move Immediate Memory
#[test]
fn mvi_mem() -> Result<(), cpu::Error> {
    let mut cpu = Cpu::new(|_| {});

    cpu.execute_instruction(Instruction::LXI {
        register: Reg::H,
        value: 0x1234,
    })?;

    cpu.execute_instruction(Instruction::MVI {
        register: Reg::M,
        value: 0x12,
    })?;

    assert_eq!(cpu.memory[0x1234], 0x12);

    Ok(())
}

// [RLC] - Rotate Left
#[test]
fn rlc() -> Result<(), cpu::Error> {
    let mut cpu = Cpu::new(|_| {});

    cpu.execute_instruction(Instruction::MVI {
        register: Reg::A,
        value: 0xff,
    })?;

    assert_eq!(cpu.a, 0xff);
    assert_eq!(cpu.condition_codes.cy, 0);

    cpu.execute_instruction(Instruction::RLC)?;

    assert_eq!(cpu.a, 0xff);
    assert_eq!(cpu.condition_codes.cy, 1);

    cpu.execute_instruction(Instruction::MVI {
        register: Reg::A,
        value: 2,
    })?;

    assert_eq!(cpu.a, 2);
    assert_eq!(cpu.condition_codes.cy, 1);

    cpu.execute_instruction(Instruction::RLC)?;

    assert_eq!(cpu.a, 4);
    assert_eq!(cpu.condition_codes.cy, 0);

    Ok(())
}

// [DAD] - Add Register Pair to H and L
#[test]
fn dad() -> Result<(), cpu::Error> {
    let mut cpu = Cpu::new(|_| {});

    cpu.execute_instruction(Instruction::LXI {
        register: Reg::B,
        value: 0x1234,
    })?;
    cpu.execute_instruction(Instruction::LXI {
        register: Reg::H,
        value: 0x4321,
    })?;
    cpu.execute_instruction(Instruction::DAD { register: Reg::B })?;

    assert_eq!(cpu.h, 0x55);
    assert_eq!(cpu.l, 0x55);

    let mut cpu = Cpu::new(|_| {});

    cpu.execute_instruction(Instruction::LXI {
        register: Reg::B,
        value: 0x1200,
    })?;
    cpu.execute_instruction(Instruction::LXI {
        register: Reg::H,
        value: 0x0034,
    })?;
    cpu.condition_codes.cy = 1;
    cpu.execute_instruction(Instruction::DAD { register: Reg::B })?;

    assert_eq!(cpu.h, 0x12);
    assert_eq!(cpu.l, 0x34);
    assert_eq!(cpu.condition_codes.cy, 0);

    let mut cpu = Cpu::new(|_| {});

    cpu.execute_instruction(Instruction::LXI {
        register: Reg::B,
        value: 1,
    })?;

    assert_eq!(cpu.c, 0x01);

    cpu.execute_instruction(Instruction::LXI {
        register: Reg::H,
        value: 0xffff,
    })?;

    assert_eq!(cpu.h, 0xff);
    assert_eq!(cpu.l, 0xff);

    cpu.execute_instruction(Instruction::DAD { register: Reg::B })?;

    assert_eq!(cpu.h, 0x00);
    assert_eq!(cpu.l, 0x00);
    assert_eq!(cpu.condition_codes.cy, 1);

    Ok(())
}

// [LDAX] - Load Accumulator Indirect
#[test]
fn ldax() -> Result<(), cpu::Error> {
    let mut cpu = Cpu::new(|_| {});

    cpu.execute_instruction(Instruction::LXI {
        register: Reg::B,
        value: 0x1234,
    })?;

    cpu.memory[0x1234] = 0x11;

    cpu.execute_instruction(Instruction::LDAX { register: Reg::B })?;

    assert_eq!(cpu.a, 0x11);

    Ok(())
}

// [DCX] - Decrement Register Pair
#[test]
fn dcx() -> Result<(), cpu::Error> {
    let mut cpu = Cpu::new(|_| {});

    cpu.execute_instruction(Instruction::LXI {
        register: Reg::B,
        value: 0x1234,
    })?;

    assert_eq!(cpu.b, 0x12);
    assert_eq!(cpu.c, 0x34);

    cpu.execute_instruction(Instruction::DCX { register: Reg::B })?;

    assert_eq!(cpu.b, 0x12);
    assert_eq!(cpu.c, 0x33);

    cpu.execute_instruction(Instruction::LXI {
        register: Reg::D,
        value: 0x0000,
    })?;

    cpu.execute_instruction(Instruction::DCX { register: Reg::D })?;

    assert_eq!(cpu.d, 0xff);
    assert_eq!(cpu.d, 0xff);
    assert_eq!(cpu.condition_codes.cy, 0);

    Ok(())
}

// [PUSH] - Push
#[test]
fn push() -> Result<(), cpu::Error> {
    let mut cpu = Cpu::new(|_| {});

    cpu.execute_instruction(Instruction::LXI {
        register: Reg::B,
        value: 0x1234,
    })?;
    cpu.execute_instruction(Instruction::LXI {
        register: Reg::SP,
        value: 0x0012,
    })?;
    cpu.execute_instruction(Instruction::PUSH { register: Reg::B })?;

    assert_eq!(cpu.memory[0x10], 0x34);
    assert_eq!(cpu.memory[0x11], 0x12);
    assert_eq!(cpu.sp, 0x10);

    Ok(())
}

// [XCHG] - Exchange H and L with D and E
#[test]
fn xchg() -> Result<(), cpu::Error> {
    let mut cpu = Cpu::new(|_| {});

    cpu.execute_instruction(Instruction::LXI {
        register: Reg::H,
        value: 0x1234,
    })?;

    cpu.execute_instruction(Instruction::LXI {
        register: Reg::D,
        value: 0x1111,
    })?;

    cpu.execute_instruction(Instruction::XCHG)?;

    assert_eq!(cpu.d, 0x12);
    assert_eq!(cpu.e, 0x34);
    assert_eq!(cpu.h, 0x11);
    assert_eq!(cpu.l, 0x11);

    Ok(())
}

// [POP] - Pop
#[test]
fn pop() -> Result<(), cpu::Error> {
    let mut cpu = Cpu::new(|_| {});

    cpu.execute_instruction(Instruction::LXI {
        register: Reg::B,
        value: 0x1234,
    })?;
    cpu.execute_instruction(Instruction::LXI {
        register: Reg::SP,
        value: 0x12,
    })?;
    cpu.execute_instruction(Instruction::PUSH { register: Reg::B })?;
    cpu.execute_instruction(Instruction::POP { register: Reg::D })?;

    assert_eq!(cpu.d, 0x12);
    assert_eq!(cpu.e, 0x34);
    assert_eq!(cpu.sp, 0x12);

    Ok(())
}

// [OUT] - Output
#[test]
fn out() -> Result<(), cpu::Error> {
    let mut i = 0;

    let mut cpu = Cpu::new(|v| i = v);

    cpu.execute_instruction(Instruction::OUT { data: 0x12 })?;

    assert_eq!(i, 0x12);

    Ok(())
}

// [ADD] - Add Register
#[test]
fn add_register() -> Result<(), cpu::Error> {
    let mut cpu = Cpu::new(|_| {});

    cpu.execute_instruction(Instruction::MVI {
        register: Reg::B,
        value: 0xFF,
    })?;

    cpu.execute_instruction(Instruction::MVI {
        register: Reg::A,
        value: 0x01,
    })?;

    cpu.execute_instruction(Instruction::ADD { register: Reg::B })?;

    assert_eq!(cpu.a, 0);
    assert_eq!(cpu.b, 0xff);
    assert_eq!(cpu.condition_codes.s, 0);
    assert_eq!(cpu.condition_codes.z, 1);
    assert_eq!(cpu.condition_codes.p, 1);
    assert_eq!(cpu.condition_codes.cy, 1);

    Ok(())
}

// [ADD] - Add Memory
#[test]
fn add_memory() -> Result<(), cpu::Error> {
    let mut cpu = Cpu::new(|_| {});

    cpu.execute_instruction(Instruction::LXI {
        register: Reg::H,
        value: 0x3456,
    })?;
    cpu.execute_instruction(Instruction::MVI {
        register: Reg::M,
        value: 0xff,
    })?;

    cpu.execute_instruction(Instruction::MVI {
        register: Reg::A,
        value: 1,
    })?;
    cpu.execute_instruction(Instruction::ADD { register: Reg::M })?;

    assert_eq!(cpu.a, 0);
    assert_eq!(cpu.memory[0x3456], 0xff);
    assert_eq!(cpu.condition_codes.s, 0);
    assert_eq!(cpu.condition_codes.z, 1);
    assert_eq!(cpu.condition_codes.p, 1);
    assert_eq!(cpu.condition_codes.cy, 1);

    Ok(())
}

// [JZ] - Jump Zero
#[test]
fn jz() -> Result<(), cpu::Error> {
    let mut cpu = Cpu::new(|_| {});

    cpu.execute_instruction(Instruction::ADD { register: Reg::B })?;

    let jump_instruction = Instruction::JZ { address: 0x12 };
    cpu.execute_instruction(jump_instruction)?;

    assert_eq!(cpu.pc, 0x12 - jump_instruction.op_bytes() as u16);

    Ok(())
}

// [JC] - Jump Carry
#[test]
fn jc() -> Result<(), cpu::Error> {
    let mut cpu = Cpu::new(|_| {});

    cpu.execute_instruction(Instruction::MVI {
        register: Reg::B,
        value: 0xff,
    })?;
    cpu.execute_instruction(Instruction::MVI {
        register: Reg::A,
        value: 1,
    })?;
    cpu.execute_instruction(Instruction::ADD { register: Reg::B })?;
    let jump_instruction = Instruction::JC { address: 0x82 };
    cpu.execute_instruction(jump_instruction)?;

    assert_eq!(cpu.pc, 0x82 - jump_instruction.op_bytes() as u16);

    cpu.pc = 1;
    cpu.execute_instruction(Instruction::ADD { register: Reg::B })?;
    cpu.execute_instruction(jump_instruction)?;
    assert_eq!(cpu.pc, 1);

    Ok(())
}

// [JNC] - Jump Not Carry
#[test]
fn jnc() -> Result<(), cpu::Error> {
    let mut cpu = Cpu::new(|_| {});

    cpu.execute_instruction(Instruction::MVI {
        register: Reg::B,
        value: 0xff,
    })?;
    cpu.execute_instruction(Instruction::MVI {
        register: Reg::A,
        value: 1,
    })?;
    cpu.execute_instruction(Instruction::ADD { register: Reg::B })?;
    let jump_instruction = Instruction::JNC { address: 0x82 };
    cpu.execute_instruction(jump_instruction)?;

    assert_eq!(cpu.pc, 0);

    cpu.execute_instruction(Instruction::ADD { register: Reg::B })?;
    cpu.execute_instruction(jump_instruction)?;
    assert_eq!(cpu.pc, 0x82 - jump_instruction.op_bytes() as u16);

    Ok(())
}
