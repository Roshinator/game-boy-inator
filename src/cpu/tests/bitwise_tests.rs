use crate::cpu::*;

#[test]
fn test_rlc_r8()
{
    let mut proc = Cpu::new();
    proc.reg_a = 0b01000000;
    //Test basic rotate
    Cpu::rlc_r8(&mut proc.reg_a, &mut proc.reg_f);
    assert_eq!(proc.reg_a, 0b10000000);
    assert_eq!(proc.reg_f, CpuFlags::empty());
    //Test wrapping
    Cpu::rlc_r8(&mut proc.reg_a, &mut proc.reg_f);
    assert_eq!(proc.reg_a, 0b00000001);
    assert_eq!(proc.reg_f, CpuFlags::FLAG_C);
    //Test zero
    proc.reg_a = 0b00000000;
    Cpu::rlc_r8(&mut proc.reg_a, &mut proc.reg_f);
    assert_eq!(proc.reg_a, 0b00000000);
    assert_eq!(proc.reg_f, CpuFlags::FLAG_Z);
}

#[test]
fn test_rlca()
{
    let mut proc = Cpu::new();
    proc.reg_a = 0b01000000;
    //Test basic rotate
    Cpu::rlca(&mut proc.reg_a, &mut proc.reg_f);
    assert_eq!(proc.reg_a, 0b10000000);
    assert_eq!(proc.reg_f, CpuFlags::empty());
    //Test wrapping
    Cpu::rlca(&mut proc.reg_a, &mut proc.reg_f);
    assert_eq!(proc.reg_a, 0b00000001);
    assert_eq!(proc.reg_f, CpuFlags::FLAG_C);
    //Test zero
    proc.reg_a = 0b00000000;
    Cpu::rlca(&mut proc.reg_a, &mut proc.reg_f);
    assert_eq!(proc.reg_a, 0b00000000);
    assert_eq!(proc.reg_f, CpuFlags::empty());
}