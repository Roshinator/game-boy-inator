use crate::cpu::*;

#[test]
fn test_rlc_r8()
{
    let mut cpu = Cpu::new();
    cpu.regs[REG_A] = 0b01000000;
    //Test basic rotate
    cpu.rlc_r8(REG_A);
    assert_eq!(cpu.regs[REG_A], 0b10000000);
    assert_eq!(cpu.regs[REG_F], CpuFlags::empty().bits);
    //Test wrapping
    cpu.rlc_r8(REG_A);
    assert_eq!(cpu.regs[REG_A], 0b00000001);
    assert_eq!(cpu.regs[REG_F], CpuFlags::FLAG_C.bits);
    //Test zero
    cpu.regs[REG_A] = 0b00000000;
    cpu.rlc_r8(REG_A);
    assert_eq!(cpu.regs[REG_A], 0b00000000);
    assert_eq!(cpu.regs[REG_F], CpuFlags::FLAG_Z.bits);
}

#[test]
fn test_rlca()
{
    let mut cpu = Cpu::new();
    cpu.regs[REG_A] = 0b01000000;
    //Test basic rotate
    cpu.rlca();
    assert_eq!(cpu.regs[REG_A], 0b10000000);
    assert_eq!(cpu.regs[REG_F], CpuFlags::empty().bits);
    //Test wrapping
    cpu.rlca();
    assert_eq!(cpu.regs[REG_A], 0b00000001);
    assert_eq!(cpu.regs[REG_F], CpuFlags::FLAG_C.bits);
    //Test zero
    cpu.regs[REG_A] = 0b00000000;
    cpu.rlca();
    assert_eq!(cpu.regs[REG_A], 0b00000000);
    assert_eq!(cpu.regs[REG_F], CpuFlags::empty().bits);
}