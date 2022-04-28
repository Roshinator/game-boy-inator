use crate::cpu::*;

#[test]
fn test_aux_inc_16()
{
    let mut proc = Cpu::new();
    let result = Cpu::aux_inc_16(0x12, 0x34);
    assert_eq!(result, (0x35, 0x12));
    let result2 = Cpu::aux_inc_16(0x00, 0xFF);
    assert_eq!(result2, (0x00, 0x01));
    let result3 = Cpu::aux_inc_16(0xFF, 0xFF);
    assert_eq!(result3, (0x00, 0x00));
}

#[test]
fn test_inc_r16()
{
    let mut proc = Cpu::new();
    let [lsh, msh] = u16::to_le_bytes(0xABCD);
    let flags_before = proc.reg_f;
    proc.reg_h = msh;
    proc.reg_l = lsh;
    Cpu::inc_r16(&mut proc.reg_h, &mut proc.reg_l);
    let [lsh_after, msh_after] = u16::to_le_bytes(0xABCE);
    assert_eq!(msh_after, proc.reg_h);
    assert_eq!(lsh_after, proc.reg_l);
    assert_eq!(flags_before, proc.reg_f);
}

#[test]
fn test_inc_sp()
{
    let mut proc = Cpu::new();
    let flags_before = proc.reg_f;
    proc.sp = 0xABCD;
    Cpu::inc_sp(&mut proc.sp);
    assert_eq!(proc.sp, 0xABCE);
    assert_eq!(flags_before, proc.reg_f);
}

#[test]
fn test_inc_r8()
{
    let mut proc = Cpu::new();
    //Test behavior
    proc.reg_f = CpuFlags::empty();
    proc.reg_a = 2;
    Cpu::inc_r8(&mut proc.reg_a, &mut proc.reg_f);
    assert_eq!(proc.reg_a, 3);
    assert_eq!(proc.reg_f, CpuFlags::empty());
    //Test H flag
    proc.reg_a = 0b00001111;
    Cpu::inc_r8(&mut proc.reg_a, &mut proc.reg_f);
    assert_eq!(proc.reg_a, 0b00010000);
    assert_eq!(proc.reg_f, CpuFlags::FLAG_H);
    //Test Z flag
    proc.reg_f = CpuFlags::empty();
    proc.reg_a = u8::MAX; //Trigger an overflow to reach 0 (guaranteed H flag too)
    Cpu::inc_r8(&mut proc.reg_a, &mut proc.reg_f);
    assert_eq!(proc.reg_a, 0);
    assert_eq!(proc.reg_f, CpuFlags::FLAG_Z | CpuFlags::FLAG_H);
}

#[test]
fn test_dec_r8()
{
    let mut proc = Cpu::new();
    //Test behavior
    proc.reg_f = CpuFlags::empty();
    proc.reg_a = 3;
    Cpu::dec_r8(&mut proc.reg_a, &mut proc.reg_f);
    assert_eq!(proc.reg_a, 2);
    assert_eq!(proc.reg_f, CpuFlags::FLAG_N);
    //Test H flag
    proc.reg_f = CpuFlags::empty();
    proc.reg_a = 0b00010000;
    Cpu::dec_r8(&mut proc.reg_a, &mut proc.reg_f);
    assert_eq!(proc.reg_a, 0b00001111);
    assert_eq!(proc.reg_f, CpuFlags::FLAG_N | CpuFlags::FLAG_H);
    //Test Z flag
    proc.reg_f = CpuFlags::empty();
    proc.reg_a = 1;
    Cpu::dec_r8(&mut proc.reg_a, &mut proc.reg_f);
    assert_eq!(proc.reg_a, 0);
    assert_eq!(proc.reg_f, CpuFlags::FLAG_N | CpuFlags::FLAG_Z);
}

#[test]
fn test_dec_r16()
{
    let mut proc = Cpu::new();
    let [lsh, msh] = u16::to_le_bytes(0xABCD);
    let flags_before = proc.reg_f;
    proc.reg_h = msh;
    proc.reg_l = lsh;
    Cpu::dec_r16(&mut proc.reg_h, &mut proc.reg_l);
    let [lsh_after, msh_after] = u16::to_le_bytes(0xABCC);
    assert_eq!(msh_after, proc.reg_h);
    assert_eq!(lsh_after, proc.reg_l);
    assert_eq!(flags_before, proc.reg_f);
}

#[test]
fn test_dec_sp()
{
    let mut proc = Cpu::new();
    let flags_before = proc.reg_f;
    proc.sp = 0xABCD;
    Cpu::dec_sp(&mut proc.sp);
    assert_eq!(proc.sp, 0xABCC);
    assert_eq!(flags_before, proc.reg_f);
}

#[test]
fn test_add_r8_r8()
{
    let mut proc = Cpu::new();
    //Test add
    proc.reg_a = 0b00000111;
    proc.reg_b = 1;
    Cpu::add_r8_r8(&mut proc.reg_a, &mut proc.reg_b, &mut proc.reg_f);
    assert_eq!(proc.reg_a, 8);
    //Test flag H (ZNHC0000)
    proc.reg_a = 1;
    proc.reg_b = 0b00001111;
    Cpu::add_r8_r8(&mut proc.reg_a, &mut proc.reg_b, &mut proc.reg_f);
    assert_eq!(proc.reg_a, 16);
    assert_eq!(proc.reg_f, CpuFlags::FLAG_H);
    //Test flag Z
    proc.reg_a = 0;
    proc.reg_b = 0;
    Cpu::add_r8_r8(&mut proc.reg_a, &mut proc.reg_b, &mut proc.reg_f);
    assert_eq!(proc.reg_a, 0);
    assert_eq!(proc.reg_f, CpuFlags::FLAG_Z);
    //Test flag C
    proc.reg_a = 0b11111111;
    proc.reg_b = 2;
    Cpu::add_r8_r8(&mut proc.reg_a, &mut proc.reg_b, &mut proc.reg_f);
    assert_eq!(proc.reg_a, 1);
    assert_eq!(proc.reg_f, CpuFlags::FLAG_H | CpuFlags::FLAG_C); //Half carry also occurs
}

#[test]
fn test_add_r8_8()
{
    let mut proc = Cpu::new();
    //Test add
    proc.reg_a = 0b00000111;
    Cpu::add_r8_8(&mut proc.reg_a, 1, &mut proc.reg_f);
    assert_eq!(proc.reg_a, 8);
    //Test flag H (ZNHC0000)
    proc.reg_a = 1;
    Cpu::add_r8_8(&mut proc.reg_a, 0b00001111, &mut proc.reg_f);
    assert_eq!(proc.reg_a, 16);
    assert_eq!(proc.reg_f, CpuFlags::FLAG_H);
    //Test flag Z
    proc.reg_a = 0;
    Cpu::add_r8_8(&mut proc.reg_a, 0, &mut proc.reg_f);
    assert_eq!(proc.reg_a, 0);
    assert_eq!(proc.reg_f, CpuFlags::FLAG_Z);
    //Test flag C
    proc.reg_a = 0b11111111;
    Cpu::add_r8_8(&mut proc.reg_a, 2, &mut proc.reg_f);
    assert_eq!(proc.reg_a, 1);
    assert_eq!(proc.reg_f, CpuFlags::FLAG_H | CpuFlags::FLAG_C); //Half carry also occurs
}

#[test]
fn test_add_r16_r16()
{
    let mut proc = Cpu::new();
    //Test add
    proc.reg_h = 0;
    proc.reg_l = 0b11111111;
    proc.reg_b = 0;
    proc.reg_c = 0b00000001;
    proc.reg_f = CpuFlags::empty();
    Cpu::add_r16_r16(&mut proc.reg_h, &mut proc.reg_l, &mut proc.reg_b, &mut proc.reg_c, &mut proc.reg_f);
    assert_eq!(proc.reg_h, 1);
    assert_eq!(proc.reg_l, 0);
    assert!(proc.reg_f.is_empty());
    //Test Carry and Zero (Zero should be unchanged)
    proc.reg_h = 0b11111111;
    proc.reg_l = 0b11111111;
    proc.reg_b = 0;
    proc.reg_c = 0b00000001;
    proc.reg_f = CpuFlags::empty();
    Cpu::add_r16_r16(&mut proc.reg_h, &mut proc.reg_l, &mut proc.reg_b, &mut proc.reg_c, &mut proc.reg_f);
    assert_eq!(proc.reg_h, 0);
    assert_eq!(proc.reg_l, 0);
    assert_eq!(proc.reg_f, CpuFlags::FLAG_H | CpuFlags::FLAG_C);
    //Test Half Carry
    proc.reg_h = 0b00001111;
    proc.reg_l = 0b11111111;
    proc.reg_b = 0;
    proc.reg_c = 0b00000001;
    proc.reg_f = CpuFlags::empty();
    Cpu::add_r16_r16(&mut proc.reg_h, &mut proc.reg_l, &mut proc.reg_b, &mut proc.reg_c, &mut proc.reg_f);
    assert_eq!(proc.reg_h, 0b00010000);
    assert_eq!(proc.reg_l, 0);
    assert_eq!(proc.reg_f, CpuFlags::FLAG_H);
}

#[test]
fn test_and_r8_r8()
{
    let mut proc = Cpu::new();
    proc.reg_a = 0b10101011;
    proc.reg_b = 0b01010101;
    Cpu::and_r8_r8(&mut proc.reg_a, &mut proc.reg_b, &mut proc.reg_f);
    assert_eq!(proc.reg_a, 1);
    assert_eq!(proc.reg_f, CpuFlags::FLAG_H);
    proc.reg_b = 0b01010100;
    Cpu::and_r8_r8(&mut proc.reg_a, &mut proc.reg_b, &mut proc.reg_f);
    assert_eq!(proc.reg_f, CpuFlags::FLAG_Z | CpuFlags::FLAG_H);
}

#[test]
fn test_and_r8_8()
{
    let mut proc = Cpu::new();
    proc.reg_a = 0b10101011;
    Cpu::and_r8_8(&mut proc.reg_a, 0b01010101, &mut proc.reg_f);
    assert_eq!(proc.reg_a, 1);
    assert_eq!(proc.reg_f, CpuFlags::FLAG_H);
    Cpu::and_r8_8(&mut proc.reg_a, 0b01010100, &mut proc.reg_f);
    assert_eq!(proc.reg_f, CpuFlags::FLAG_Z | CpuFlags::FLAG_H);
}

#[test]
fn test_xor_r8_r8()
{
    let mut proc = Cpu::new();
    proc.reg_a = 0b10101011;
    proc.reg_b = 0b01010101;
    Cpu::xor_r8_r8(&mut proc.reg_a, &mut proc.reg_b, &mut proc.reg_f);
    assert_eq!(proc.reg_a, 0b11111110);
    assert!(proc.reg_f.is_empty());
    proc.reg_b = 0b11111110;
    Cpu::xor_r8_r8(&mut proc.reg_a, &mut proc.reg_b, &mut proc.reg_f);
    assert_eq!(proc.reg_f, CpuFlags::FLAG_Z);
}

#[test]
fn test_xor_r8_8()
{
    let mut proc = Cpu::new();
    proc.reg_a = 0b10101011;
    Cpu::xor_r8_8(&mut proc.reg_a, 0b01010101, &mut proc.reg_f);
    assert_eq!(proc.reg_a, 0b11111110);
    assert!(proc.reg_f.is_empty());
    Cpu::xor_r8_8(&mut proc.reg_a, 0b11111110, &mut proc.reg_f);
    assert_eq!(proc.reg_f, CpuFlags::FLAG_Z);
}

#[test]
fn test_or_r8_r8()
{
    let mut proc = Cpu::new();
    proc.reg_a = 0b10101011;
    proc.reg_b = 0b01010101;
    Cpu::or_r8_r8(&mut proc.reg_a, &mut proc.reg_b, &mut proc.reg_f);
    assert_eq!(proc.reg_a, 0b11111111);
    assert!(proc.reg_f.is_empty());
    proc.reg_a = 0b00000000;
    proc.reg_b = 0b00000000;
    Cpu::or_r8_r8(&mut proc.reg_a, &mut proc.reg_b, &mut proc.reg_f);
    assert_eq!(proc.reg_f, CpuFlags::FLAG_Z);
}

#[test]
fn test_or_r8_8()
{
    let mut proc = Cpu::new();
    proc.reg_a = 0b10101011;
    Cpu::or_r8_8(&mut proc.reg_a, 0b01010101, &mut proc.reg_f);
    assert_eq!(proc.reg_a, 0b11111111);
    assert!(proc.reg_f.is_empty());
    proc.reg_a = 0b00000000;
    Cpu::or_r8_8(&mut proc.reg_a, 0b00000000, &mut proc.reg_f);
    assert_eq!(proc.reg_f, CpuFlags::FLAG_Z);
}

#[test]
fn test_cpl()
{
    let mut proc = Cpu::new();
    proc.reg_a = 0b10101010;
    Cpu::cpl(&mut proc.reg_a, &mut proc.reg_f);
    assert_eq!(proc.reg_a, 0b01010101);
    assert!(proc.reg_f.contains(CpuFlags::FLAG_H));
    assert!(proc.reg_f.contains(CpuFlags::FLAG_N));
}

#[test]
fn test_ccf()
{
    let mut proc = Cpu::new();
    proc.reg_f = CpuFlags::FLAG_C;
    Cpu::ccf(&mut proc.reg_f);
    assert!(!proc.reg_f.contains(CpuFlags::FLAG_C));
}

#[test]
fn test_scf()
{
    let mut proc = Cpu::new();
    proc.reg_f = CpuFlags::empty();
    Cpu::scf(&mut proc.reg_f);
    assert!(proc.reg_f.contains(CpuFlags::FLAG_C));
}
