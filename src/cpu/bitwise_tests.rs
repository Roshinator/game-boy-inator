#[cfg(test)]
mod bitwise_tests
{
    use crate::cpu::{self, *};

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
    fn test_rlc_r16a()
    {
        let mut cpu = Cpu::new();
        let mut ram = Ram::new();
        (cpu.regs[REG_H], cpu.regs[REG_L]) = (0x50, 0x00);
        //Test rotate
        ram.write(0x5000, 0b01000000);
        cpu.rlc_r16a(&mut ram, REG_H, REG_L);
        assert_eq!(ram.read_rp(cpu.regs[REG_H], cpu.regs[REG_L]), 0b10000000);
        assert_eq!(cpu.regs[REG_F], CpuFlags::empty().bits);
        //Test wrapping
        cpu.rlc_r16a(&mut ram, REG_H, REG_L);
        assert_eq!(ram.read_rp(cpu.regs[REG_H], cpu.regs[REG_L]), 0b00000001);
        assert_eq!(cpu.regs[REG_F], CpuFlags::FLAG_C.bits);
        //Test zero
        ram.write(0x5000, 0b00000000);
        cpu.rlc_r16a(&mut ram, REG_H, REG_L);
        assert_eq!(ram.read_rp(cpu.regs[REG_H], cpu.regs[REG_L]), 0b00000000);
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
}