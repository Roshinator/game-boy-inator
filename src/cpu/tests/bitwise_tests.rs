use crate::cpu::*;

#[test]
fn test_rlc_r8()
{
    let mut proc = Cpu::new();
    proc.reg_a = 0b01000000;
    //Test basic rotate
    Cpu::rlc_r8(&mut proc.reg_a, &mut proc.reg_f);
    assert_eq!(proc.reg_a, 0b10000000);
    assert!(proc.reg_f.is_empty());
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
    assert!(proc.reg_f.is_empty());
    //Test wrapping
    proc.reg_a = 0b10000000;
    proc.reg_f = CpuFlags::empty();
    Cpu::rlca(&mut proc.reg_a, &mut proc.reg_f);
    assert_eq!(proc.reg_a, 0b00000001);
    assert_eq!(proc.reg_f, CpuFlags::FLAG_C);
    //Test zero
    proc.reg_a = 0b00000000;
    proc.reg_f = CpuFlags::empty();
    Cpu::rlca(&mut proc.reg_a, &mut proc.reg_f);
    assert_eq!(proc.reg_a, 0b00000000);
    assert!(proc.reg_f.is_empty());
}

#[test]
fn test_rrc_r8()
{
    let mut proc = Cpu::new();
    proc.reg_a = 0b10000000;
    //Test basic rotate
    Cpu::rrc_r8(&mut proc.reg_a, &mut proc.reg_f);
    assert_eq!(proc.reg_a, 0b01000000);
    assert!(proc.reg_f.is_empty());
    //Test wrapping
    proc.reg_a = 0b00000001;
    proc.reg_f = CpuFlags::empty();
    Cpu::rrc_r8(&mut proc.reg_a, &mut proc.reg_f);
    assert_eq!(proc.reg_a, 0b10000000);
    assert_eq!(proc.reg_f, CpuFlags::FLAG_C);
    //Test zero
    proc.reg_a = 0b00000000;
    proc.reg_f = CpuFlags::empty();
    Cpu::rrc_r8(&mut proc.reg_a, &mut proc.reg_f);
    assert_eq!(proc.reg_a, 0b00000000);
    assert_eq!(proc.reg_f, CpuFlags::FLAG_Z);
}

#[test]
fn test_rrca()
{
    let mut proc = Cpu::new();
    proc.reg_a = 0b10000000;
    //Test basic rotate
    Cpu::rrca(&mut proc.reg_a, &mut proc.reg_f);
    assert_eq!(proc.reg_a, 0b01000000);
    assert!(proc.reg_f.is_empty());
    //Test wrapping
    proc.reg_a = 0b00000001;
    proc.reg_f = CpuFlags::empty();
    Cpu::rrca(&mut proc.reg_a, &mut proc.reg_f);
    assert_eq!(proc.reg_a, 0b10000000);
    assert_eq!(proc.reg_f, CpuFlags::FLAG_C);
    //Test zero
    proc.reg_a = 0b00000000;
    proc.reg_f = CpuFlags::empty();
    Cpu::rrca(&mut proc.reg_a, &mut proc.reg_f);
    assert_eq!(proc.reg_a, 0b00000000);
    assert!(proc.reg_f.is_empty());
}

#[test]
fn test_rl_r8()
{
    let mut proc = Cpu::new();
    //Test Z and carry out
    proc.reg_a = 0b10000000;
    proc.reg_f = CpuFlags::empty();
    Cpu::rl_r8(&mut proc.reg_a, &mut proc.reg_f);
    assert_eq!(proc.reg_a, 0);
    assert_eq!(proc.reg_f, CpuFlags::FLAG_Z | CpuFlags::FLAG_C);
    //Test carry in
    proc.reg_a = 0b00000000;
    proc.reg_f = CpuFlags::FLAG_C;
    Cpu::rl_r8(&mut proc.reg_a, &mut proc.reg_f);
    assert_eq!(proc.reg_a, 0b00000001);
    assert!(proc.reg_f.is_empty());
}

#[test]
fn test_rla()
{
    let mut proc = Cpu::new();
    //Test Z and carry out
    proc.reg_a = 0b10000000;
    proc.reg_f = CpuFlags::empty();
    Cpu::rla(&mut proc.reg_a, &mut proc.reg_f);
    assert_eq!(proc.reg_a, 0);
    assert_eq!(proc.reg_f, CpuFlags::FLAG_C);
    //Test carry in
    proc.reg_a = 0b00000000;
    proc.reg_f = CpuFlags::FLAG_C;
    Cpu::rla(&mut proc.reg_a, &mut proc.reg_f);
    assert_eq!(proc.reg_a, 0b00000001);
    assert!(proc.reg_f.is_empty());
}

#[test]
fn test_rr_r8()
{
    let mut proc = Cpu::new();
    //Test Z and carry out
    proc.reg_a = 0b00000001;
    proc.reg_f = CpuFlags::empty();
    Cpu::rr_r8(&mut proc.reg_a, &mut proc.reg_f);
    assert_eq!(proc.reg_a, 0);
    assert_eq!(proc.reg_f, CpuFlags::FLAG_Z | CpuFlags::FLAG_C);
    //Test carry in
    proc.reg_a = 0b00000000;
    proc.reg_f = CpuFlags::FLAG_C;
    Cpu::rr_r8(&mut proc.reg_a, &mut proc.reg_f);
    assert_eq!(proc.reg_a, 0b10000000);
    assert!(proc.reg_f.is_empty());
}

#[test]
fn test_rra()
{
    let mut proc = Cpu::new();
    //Test Z and carry out
    proc.reg_a = 0b00000001;
    proc.reg_f = CpuFlags::empty();
    Cpu::rra(&mut proc.reg_a, &mut proc.reg_f);
    assert_eq!(proc.reg_a, 0);
    assert_eq!(proc.reg_f, CpuFlags::FLAG_C);
    //Test carry in
    proc.reg_a = 0b00000000;
    proc.reg_f = CpuFlags::FLAG_C;
    Cpu::rra(&mut proc.reg_a, &mut proc.reg_f);
    assert_eq!(proc.reg_a, 0b10000000);
    assert!(proc.reg_f.is_empty());
}

#[test]
fn test_sla_r8()
{
    let mut proc = Cpu::new();
    //Test Z and carry out
    proc.reg_a = 0b10000000;
    proc.reg_f = CpuFlags::empty();
    Cpu::sla_r8(&mut proc.reg_a, &mut proc.reg_f);
    assert_eq!(proc.reg_a, 0);
    assert_eq!(proc.reg_f, CpuFlags::FLAG_Z | CpuFlags::FLAG_C);
    //Test carry in
    proc.reg_a = 0b00000000;
    proc.reg_f = CpuFlags::FLAG_C;
    Cpu::sla_r8(&mut proc.reg_a, &mut proc.reg_f);
    assert_eq!(proc.reg_a, 0b00000000);
    assert_eq!(proc.reg_f, CpuFlags::FLAG_Z);
}

#[test]
fn test_sra_r8()
{
    let mut proc = Cpu::new();
    //Test Z and carry out
    proc.reg_a = 0b10000001;
    proc.reg_f = CpuFlags::empty();
    Cpu::sra_r8(&mut proc.reg_a, &mut proc.reg_f);
    assert_eq!(proc.reg_a, 0b11000000);
    assert_eq!(proc.reg_f, CpuFlags::FLAG_C);
    //Test carry in
    proc.reg_a = 0b00000000;
    proc.reg_f = CpuFlags::FLAG_C;
    Cpu::sra_r8(&mut proc.reg_a, &mut proc.reg_f);
    assert_eq!(proc.reg_a, 0b00000000);
    assert_eq!(proc.reg_f, CpuFlags::FLAG_Z);
}

#[test]
fn test_srl_r8()
{
    let mut proc = Cpu::new();
    //Test Z and carry out
    proc.reg_a = 0b10000001;
    proc.reg_f = CpuFlags::empty();
    Cpu::srl_r8(&mut proc.reg_a, &mut proc.reg_f);
    assert_eq!(proc.reg_a, 0b01000000);
    assert_eq!(proc.reg_f, CpuFlags::FLAG_C);
    //Test carry in
    proc.reg_a = 0b00000000;
    proc.reg_f = CpuFlags::FLAG_C;
    Cpu::srl_r8(&mut proc.reg_a, &mut proc.reg_f);
    assert_eq!(proc.reg_a, 0b00000000);
    assert_eq!(proc.reg_f, CpuFlags::FLAG_Z);
}

#[test]
fn test_swap_r8()
{
    let mut proc = Cpu::new();
    proc.reg_a = 0xAB;
    //Flags not reset because all should be set to 0 by function except Z which is dynamic
    Cpu::swap_r8(&mut proc.reg_a, &mut proc.reg_f);
    assert_eq!(proc.reg_a, 0xBA);
    assert!(proc.reg_f.is_empty());
    //Test Z flag
    proc.reg_a = 0x00;
    //Flags not reset because all should be set to 0 by function except Z which is dynamic
    Cpu::swap_r8(&mut proc.reg_a, &mut proc.reg_f);
    assert_eq!(proc.reg_a, 0x00);
    assert_eq!(proc.reg_f, CpuFlags::FLAG_Z);
}

#[test]
fn test_bit_r8()
{
    let mut proc = Cpu::new();
    //Test Z is false + Cy unchanged
    proc.reg_a = 0b00000001;
    proc.reg_f = CpuFlags::FLAG_C; //Test if C is unchanged
    Cpu::bit_r8(0, &mut proc.reg_a, &mut proc.reg_f);
    assert_eq!(proc.reg_f, CpuFlags::FLAG_C | CpuFlags::FLAG_H);
    //Test Z is true
    proc.reg_a = 0b11111110;
    proc.reg_f = CpuFlags::empty();
    Cpu::bit_r8(0, &mut proc.reg_a, &mut proc.reg_f);
    assert_eq!(proc.reg_f, CpuFlags::FLAG_Z | CpuFlags::FLAG_H);
}

#[test]
fn test_res_r8()
{
    let mut proc = Cpu::new();
    proc.reg_a = 0b11111111;
    Cpu::res_r8(4, &mut proc.reg_a);
    assert_eq!(proc.reg_a, 0b11101111);
}

#[test]
fn test_set_r8()
{
    let mut proc = Cpu::new();
    proc.reg_a = 0b00000000;
    Cpu::set_r8(4, &mut proc.reg_a);
    assert_eq!(proc.reg_a, 0b00010000);
}