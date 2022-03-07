#[cfg(test)]
mod ld_tests
{
    use crate::cpu::{self, *};

    #[test]
    fn test_ld_r16_16()
    {
        let mut cpu = Cpu::new();
        cpu.ld_r16_16(cpu::REG_H, cpu::REG_L,
            33, 36);
        assert_eq!(cpu.regs[cpu::REG_H], 33);
        assert_eq!(cpu.regs[cpu::REG_L], 36);
    }

    #[test]
    fn test_ld_hl_sp_plus()
    {
        let mut cpu = Cpu::new();
        cpu.sp = 0x11FF;
        cpu.ld_hl_sp_plus(1);
        assert_eq!(cpu.regs[REG_H], 0x12);
        assert_eq!(cpu.regs[REG_L], 0x00);
        cpu.ld_hl_sp_plus(-1);
        assert_eq!(cpu.regs[REG_H], 0x11);
        assert_eq!(cpu.regs[REG_L], 0xFE);
        //CHECK CARRY FLAGS
    }

    #[test]
    fn test_ld_sp_16()
    {
        let mut cpu = Cpu::new();
        cpu.sp = 0;
        cpu.ld_sp_16(0x69, 0x42);
        assert_eq!(cpu.sp, 0x6942);
    }

    #[test]
    fn test_ld_r8_8()
    {
        let mut cpu = Cpu::new();
        cpu.regs[REG_D] = 0;
        cpu.ld_r8_8(REG_D, 0x69);
        assert_eq!(cpu.regs[REG_D], 0x69);
    }

    #[test]
    fn test_ld_r16a_8()
    {
        let mut cpu = Cpu::new();
        (cpu.regs[REG_H], cpu.regs[REG_L]) = (0x69, 0x42);
        let mut ram = Ram::new();
        cpu.ld_r16a_8(&mut ram, REG_H, REG_L, 0x88);
        assert_eq!(ram.read(0x6942), 0x88);
    }

    #[test]
    fn test_ld_16a_r8()
    {
        let mut cpu = Cpu::new();
        let mut ram = Ram::new();
        cpu.regs[REG_D] = 0x88;
        cpu.ld_16a_r8(&mut ram, 0x69, 0x42, REG_D);
        assert_eq!(ram.read(0x6942), 0x88);
    }

    #[test]
    fn test_ld_16a_sp()
    {
        let mut cpu = Cpu::new();
        let mut ram = Ram::new();
        cpu.sp = 0x1234;
        cpu.ld_16a_sp(&mut ram, 0x69, 0x42);
        assert_eq!(ram.read(0x6942), 0x34);
        assert_eq!(ram.read(0x6942 + 1), 0x12);
    }

    #[test]
    fn test_ld_r8_r8()
    {
        let mut cpu = Cpu::new();
        cpu.regs[REG_A] = 0;
        cpu.regs[REG_B] = 0x21;
        cpu.ld_r8_r8(REG_A, REG_B);
        assert_eq!(cpu.regs[REG_A], 0x21);
    }

    #[test]
    fn test_ld_sp_r16()
    {
        let mut cpu = Cpu::new();
        cpu.sp = 0;
        cpu.regs[REG_H] = 0x69;
        cpu.regs[REG_L] = 0x42;
        cpu.ld_sp_r16(REG_H, REG_L);
        assert_eq!(cpu.sp, 0x6942);
    }

    #[test]
    fn test_ld_r8_r16a()
    {
        let mut cpu = Cpu::new();
        (cpu.regs[REG_H], cpu.regs[REG_L]) = (0x12, 0x34);
        let mut ram = Ram::new();
        ram.write(0x1234, 0x69);
        cpu.ld_r8_r16a(&mut ram, REG_A, REG_H, REG_L);
        assert_eq!(cpu.regs[REG_A], 0x69);
    }

    #[test]
    fn test_ld_r8_16a()
    {
        let mut cpu = Cpu::new();
        let mut ram = Ram::new();
        ram.write(0x1234, 0x69);
        cpu.ld_r8_16a(&mut ram, REG_A, 0x12, 0x34);
        assert_eq!(cpu.regs[REG_A], 0x69);
    }

    #[test]
    fn test_ld_r16a_r8()
    {
        let mut cpu = Cpu::new();
        let mut ram = Ram::new();
        cpu.regs[REG_C] = 0x69;
        (cpu.regs[REG_H], cpu.regs[REG_L]) = (0x12, 0x34);
        cpu.ld_r16a_r8(&mut ram, REG_H, REG_L, REG_C);
        assert_eq!(ram.read(0x1234), 0x69);
    }
}