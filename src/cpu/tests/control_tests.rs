use crate::cpu::*;

#[test]
fn test_halt()
{
    let mut cpu = Cpu::new();
    cpu.halt();
    assert!(cpu.halted);
}

#[test]
fn test_stop()
{
    let mut cpu = Cpu::new();
    cpu.stop();
    assert!(cpu.halted && cpu.stopped);
}

#[test]
fn test_aux_inc_pc()
{
    let mut cpu = Cpu::new();
    cpu.pc.current_instruction_width = 3;
    cpu.pc.should_increment = false;
    cpu.aux_inc_pc();
    assert_eq!(cpu.pc.reg, 0x0000);
    assert!(cpu.pc.current_instruction_width != 0);
    cpu.pc.should_increment = true;
    cpu.aux_inc_pc();
    assert_eq!(cpu.pc.reg, 0x0001);
    assert!(cpu.pc.current_instruction_width == 0);
}

#[test]
fn test_aux_read_pc()
{
    let mut cpu = Cpu::new();
    let mut ram = Ram::new();
    ram.write(0x5050, 0x12);
    cpu.pc.reg = 0x5050;
    let result = cpu.aux_read_pc(&mut ram);
    assert_eq!(result, 0x12);
}

#[test]
fn aux_read_immediate_data()
{
    let mut cpu = Cpu::new();
    let mut ram = Ram::new();
    ram.write(0x5050, 0x01);
    ram.write(0x5051, 0x02);
    cpu.pc.reg = 0x5050;
    let result = cpu.aux_read_immediate_data(&mut ram);
    assert_eq!(result, 0x02);
}

#[test]
fn test_reti()
{
    let mut proc = Cpu::new();
    let mut mem = Ram::new();
    mem.write(0xFFFD, 0x69); //LSH
    mem.write(0xFFFE, 0x42); //MSH
    proc.sp = 0xFFFD;
    Cpu::reti(&mut mem, &mut proc.sp, &mut proc.pc, &mut proc.ime);
    assert_eq!(proc.sp, 0xFFFF);
    assert_eq!(proc.pc.reg, 0x4269);
    assert_eq!(proc.ime, true);
}

#[test]
fn test_ei()
{
    let mut proc = Cpu::new();
    proc.ime = false;
    Cpu::ei(&mut proc.ime);
    assert_eq!(proc.ime, true);
}

#[test]
fn test_di()
{
    let mut proc = Cpu::new();
    proc.ime = true;
    Cpu::di(&mut proc.ime);
    assert_eq!(proc.ime, false);
}