#[cfg(test)]
mod jump_branch_tests
{
    use crate::cpu::*;

    #[test]
    fn test_jp_pc_16()
    {
        let mut cpu = Cpu::new();
        cpu.jp_pc_16(0x69, 0x42);
        assert_eq!(cpu.pc.reg, 0x6942);
        assert!(!cpu.pc.should_increment);
    }

    #[test]
    fn test_jp_flag_pc_16()
    {
        let mut cpu = Cpu::new();
        cpu.regs[REG_F] = 0;
        cpu.jp_flag_pc_16(Flag::FLAG_Z,0x69, 0x42);
        assert_eq!(cpu.pc.reg, 0);
        assert!(cpu.pc.should_increment);
        cpu.regs[REG_F] = 0b10000000;
        cpu.jp_flag_pc_16(Flag::FLAG_Z,0x69, 0x42);
        assert_eq!(cpu.pc.reg, 0x6942);
        assert!(!cpu.pc.should_increment);
    }

    fn test_jp_nflag_pc_16()
    {
        let mut cpu = Cpu::new();
        cpu.regs[REG_F] = 0b10000000;
        cpu.jp_nflag_pc_16(Flag::FLAG_Z,0x69, 0x42);
        assert_eq!(cpu.pc.reg, 0);
        assert!(cpu.pc.should_increment);
        cpu.regs[REG_F] = 0;
        cpu.jp_nflag_pc_16(Flag::FLAG_Z,0x69, 0x42);
        assert_eq!(cpu.pc.reg, 0x6942);
        assert!(!cpu.pc.should_increment);
    }

    #[test]
    fn test_jr_i8()
    {
        let mut cpu = Cpu::new();
        cpu.jr_i8(0x69);
        assert_eq!(cpu.pc.reg, 0x69);
        assert!(!cpu.pc.should_increment);
        cpu.jr_i8(-0x69);
        assert_eq!(cpu.pc.reg, 0);
        assert!(!cpu.pc.should_increment);
    }

    #[test]
    fn test_jr_flag_i8()
    {
        let mut cpu = Cpu::new();
        cpu.regs[REG_F] = 0;
        cpu.jr_flag_i8(Flag::FLAG_Z, 0x69);
        assert_eq!(cpu.pc.reg, 0);
        assert!(cpu.pc.should_increment);
        cpu.jr_flag_i8(Flag::FLAG_Z, -0x69);
        assert_eq!(cpu.pc.reg, 0);
        assert!(cpu.pc.should_increment);
        cpu.regs[REG_F] = 0b10000000;
        cpu.jr_flag_i8(Flag::FLAG_Z, 0x69);
        assert_eq!(cpu.pc.reg, 0x69);
        assert!(!cpu.pc.should_increment);
        cpu.jr_flag_i8(Flag::FLAG_Z, -0x69);
        assert_eq!(cpu.pc.reg, 0);
        assert!(!cpu.pc.should_increment);
    }

    #[test]
    fn test_jr_nflag_i8()
    {
        let mut cpu = Cpu::new();
        cpu.regs[REG_F] = 0b10000000;
        cpu.jr_nflag_i8(Flag::FLAG_Z, 0x69);
        assert_eq!(cpu.pc.reg, 0);
        assert!(cpu.pc.should_increment);
        cpu.jr_nflag_i8(Flag::FLAG_Z, -0x69);
        assert_eq!(cpu.pc.reg, 0);
        assert!(cpu.pc.should_increment);
        cpu.regs[REG_F] = 0;
        cpu.jr_nflag_i8(Flag::FLAG_Z, 0x69);
        assert_eq!(cpu.pc.reg, 0x69);
        assert!(!cpu.pc.should_increment);
        cpu.jr_nflag_i8(Flag::FLAG_Z, -0x69);
        assert_eq!(cpu.pc.reg, 0);
        assert!(!cpu.pc.should_increment);
    }

    #[test]
    fn test_call_16()
    {
        let mut cpu = Cpu::new();
        let mut ram = Ram::new();
        cpu.sp = 0x6969;
        cpu.pc.reg = 0x1234;
        cpu.call_16(&mut ram, 0x56, 0x78);
        assert_eq!(cpu.pc.reg, 0x5678);
        assert_eq!(cpu.sp, 0x6967);
        assert_eq!(ram.read(0x6969 - 1), 0x12);
        assert_eq!(ram.read(0x6969 - 2), 0x34);
    }

    #[test]
    fn test_call_flag_16()
    {
        let mut cpu = Cpu::new();
        let mut ram = Ram::new();
        cpu.sp = 0x6969;
        cpu.pc.reg = 0x1234;
        cpu.regs[REG_F] = 0b00000000;
        cpu.call_flag_16(&mut ram, Flag::FLAG_Z, 0x56, 0x78);
        assert_eq!(cpu.pc.reg, 0x1234);
        assert_eq!(cpu.sp, 0x6969);
        assert_eq!(ram.read(0x6969 - 1), 0x0);
        assert_eq!(ram.read(0x6969 - 2), 0x0);
        cpu.regs[REG_F] = 0b10000000;
        cpu.call_flag_16(&mut ram, Flag::FLAG_Z, 0x56, 0x78);
        assert_eq!(cpu.pc.reg, 0x5678);
        assert_eq!(cpu.sp, 0x6967);
        assert_eq!(ram.read(0x6969 - 1), 0x12);
        assert_eq!(ram.read(0x6969 - 2), 0x34);
    }

    #[test]
    fn test_call_nflag_16()
    {
        let mut cpu = Cpu::new();
        let mut ram = Ram::new();
        cpu.sp = 0x6969;
        cpu.pc.reg = 0x1234;
        cpu.regs[REG_F] = 0b10000000;
        cpu.call_nflag_16(&mut ram, Flag::FLAG_Z, 0x56, 0x78);
        assert_eq!(cpu.pc.reg, 0x1234);
        assert_eq!(cpu.sp, 0x6969);
        assert_eq!(ram.read(0x6969 - 1), 0x0);
        assert_eq!(ram.read(0x6969 - 2), 0x0);
        cpu.regs[REG_F] = 0b00000000;
        cpu.call_nflag_16(&mut ram, Flag::FLAG_Z, 0x56, 0x78);
        assert_eq!(cpu.pc.reg, 0x5678);
        assert_eq!(cpu.sp, 0x6967);
        assert_eq!(ram.read(0x6969 - 1), 0x12);
        assert_eq!(ram.read(0x6969 - 2), 0x34);
    }

    #[test]
    fn test_ret()
    {
        let mut cpu = Cpu::new();
        let mut ram = Ram::new();
        ram.write(0x6968, 0x12);
        ram.write(0x6967, 0x34);
        cpu.sp = 0x6967;
        cpu.ret(&mut ram);
        assert_eq!(cpu.pc.reg, 0x1234);
        assert_eq!(cpu.sp, 0x6969);
    }

    #[test]
    fn test_ret_flag()
    {
        let mut cpu = Cpu::new();
        let mut ram = Ram::new();
        ram.write(0x6968, 0x12);
        ram.write(0x6967, 0x34);
        cpu.sp = 0x6967;
        cpu.regs[REG_F] = 0b00000000;
        cpu.ret_flag(&mut ram, Flag::FLAG_Z);
        assert_eq!(cpu.pc.reg, 0);
        assert_eq!(cpu.sp, 0x6967);
        cpu.regs[REG_F] = 0b10000000;
        cpu.ret_flag(&mut ram, Flag::FLAG_Z);
        assert_eq!(cpu.pc.reg, 0x1234);
        assert_eq!(cpu.sp, 0x6969);
    }

    #[test]
    fn test_ret_nflag()
    {
        let mut cpu = Cpu::new();
        let mut ram = Ram::new();
        ram.write(0x6968, 0x12);
        ram.write(0x6967, 0x34);
        cpu.sp = 0x6967;
        cpu.regs[REG_F] = 0b10000000;
        cpu.ret_nflag(&mut ram, Flag::FLAG_Z);
        assert_eq!(cpu.pc.reg, 0);
        assert_eq!(cpu.sp, 0x6967);
        cpu.regs[REG_F] = 0b00000000;
        cpu.ret_nflag(&mut ram, Flag::FLAG_Z);
        assert_eq!(cpu.pc.reg, 0x1234);
        assert_eq!(cpu.sp, 0x6969);
    }
}