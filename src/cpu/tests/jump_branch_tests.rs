#[cfg(test)]
mod jump_branch_tests
{
    use crate::cpu::*;

    #[test]
    fn test_jp_pc_16()
    {
        let mut proc = Cpu::new();
        Cpu::jp_pc_16(&mut proc.pc, 0x69, 0x42);
        assert_eq!(proc.pc.reg, 0x6942);
        assert!(!proc.pc.should_increment);
    }

    #[test]
    fn test_jp_flag_pc_16()
    {
        let mut proc = Cpu::new();
        proc.reg_f = CpuFlags::empty();
        Cpu::jp_flag_pc_16(&mut proc.pc, CpuFlags::FLAG_Z,0x69, 0x42, &mut proc.reg_f);
        assert_eq!(proc.pc.reg, 0);
        assert!(proc.pc.should_increment);
        proc.reg_f = CpuFlags::FLAG_Z;
        Cpu::jp_flag_pc_16(&mut proc.pc, CpuFlags::FLAG_Z,0x69, 0x42, &mut proc.reg_f);
        assert_eq!(proc.pc.reg, 0x6942);
        assert!(!proc.pc.should_increment);
    }

    fn test_jp_nflag_pc_16()
    {
        let mut proc = Cpu::new();
        proc.reg_f = CpuFlags::FLAG_Z;
        Cpu::jp_nflag_pc_16(&mut proc.pc, CpuFlags::FLAG_Z,0x69, 0x42, &mut proc.reg_f);
        assert_eq!(proc.pc.reg, 0);
        assert!(proc.pc.should_increment);
        proc.reg_f = CpuFlags::empty();
        Cpu::jp_nflag_pc_16(&mut proc.pc, CpuFlags::FLAG_Z,0x69, 0x42, &mut proc.reg_f);
        assert_eq!(proc.pc.reg, 0x6942);
        assert!(!proc.pc.should_increment);
    }

    #[test]
    fn test_jr_i8()
    {
        let mut proc = Cpu::new();
        Cpu::jr_i8(&mut proc.pc, 0x69);
        assert_eq!(proc.pc.reg, 0x69);
        assert!(!proc.pc.should_increment);
        Cpu::jr_i8(&mut proc.pc, -0x69);
        assert_eq!(proc.pc.reg, 0);
        assert!(!proc.pc.should_increment);
    }

    #[test]
    fn test_jr_flag_i8()
    {
        let mut proc = Cpu::new();
        proc.reg_f = CpuFlags::empty();
        Cpu::jr_flag_i8(&mut proc.pc, CpuFlags::FLAG_Z, 0x69, &mut proc.reg_f);
        assert_eq!(proc.pc.reg, 0);
        assert!(proc.pc.should_increment);
        Cpu::jr_flag_i8(&mut proc.pc, CpuFlags::FLAG_Z, -0x69, &mut proc.reg_f);
        assert_eq!(proc.pc.reg, 0);
        assert!(proc.pc.should_increment);
        proc.reg_f = CpuFlags::FLAG_Z;
        Cpu::jr_flag_i8(&mut proc.pc, CpuFlags::FLAG_Z, 0x69, &mut proc.reg_f);
        assert_eq!(proc.pc.reg, 0x69);
        assert!(!proc.pc.should_increment);
        Cpu::jr_flag_i8(&mut proc.pc, CpuFlags::FLAG_Z, -0x69, &mut proc.reg_f);
        assert_eq!(proc.pc.reg, 0);
        assert!(!proc.pc.should_increment);
    }

    #[test]
    fn test_jr_nflag_i8()
    {
        let mut proc = Cpu::new();
        proc.reg_f = CpuFlags::FLAG_Z;
        Cpu::jr_nflag_i8(&mut proc.pc, CpuFlags::FLAG_Z, 0x69, &mut proc.reg_f);
        assert_eq!(proc.pc.reg, 0);
        assert!(proc.pc.should_increment);
        Cpu::jr_nflag_i8(&mut proc.pc, CpuFlags::FLAG_Z, -0x69, &mut proc.reg_f);
        assert_eq!(proc.pc.reg, 0);
        assert!(proc.pc.should_increment);
        proc.reg_f = CpuFlags::empty();
        Cpu::jr_nflag_i8(&mut proc.pc, CpuFlags::FLAG_Z, 0x69, &mut proc.reg_f);
        assert_eq!(proc.pc.reg, 0x69);
        assert!(!proc.pc.should_increment);
        Cpu::jr_nflag_i8(&mut proc.pc, CpuFlags::FLAG_Z, -0x69, &mut proc.reg_f);
        assert_eq!(proc.pc.reg, 0);
        assert!(!proc.pc.should_increment);
    }

    #[test]
    fn test_call_16()
    {
        let mut proc = Cpu::new();
        let mut mem = Ram::new();
        proc.sp = 0x6969;
        proc.pc.reg = 0x1234;
        Cpu::call_16(&mut mem, 0x56, 0x78, &mut proc.pc, &mut proc.sp);
        assert_eq!(proc.pc.reg, 0x5678);
        assert_eq!(proc.sp, 0x6967);
        assert_eq!(mem.read(0x6969 - 1), 0x12);
        assert_eq!(mem.read(0x6969 - 2), 0x34);
    }

    #[test]
    fn test_call_flag_16()
    {
        let mut proc = Cpu::new();
        let mut mem = Ram::new();
        proc.sp = 0x6969;
        proc.pc.reg = 0x1234;
        proc.reg_f = CpuFlags::empty();
        Cpu::call_flag_16(&mut mem, CpuFlags::FLAG_Z, 0x56, 0x78, &mut proc.pc, &mut proc.sp, &mut proc.reg_f);
        assert_eq!(proc.pc.reg, 0x1234);
        assert_eq!(proc.sp, 0x6969);
        assert_eq!(mem.read(0x6969 - 1), 0x0);
        assert_eq!(mem.read(0x6969 - 2), 0x0);
        proc.reg_f = CpuFlags::FLAG_Z;
        Cpu::call_flag_16(&mut mem, CpuFlags::FLAG_Z, 0x56, 0x78, &mut proc.pc, &mut proc.sp, &mut proc.reg_f);
        assert_eq!(proc.pc.reg, 0x5678);
        assert_eq!(proc.sp, 0x6967);
        assert_eq!(mem.read(0x6969 - 1), 0x12);
        assert_eq!(mem.read(0x6969 - 2), 0x34);
    }

    #[test]
    fn test_call_nflag_16()
    {
        let mut proc = Cpu::new();
        let mut mem = Ram::new();
        proc.sp = 0x6969;
        proc.pc.reg = 0x1234;
        proc.reg_f = CpuFlags::FLAG_Z;
        Cpu::call_nflag_16(&mut mem, CpuFlags::FLAG_Z, 0x56, 0x78, &mut proc.pc, &mut proc.sp, &mut proc.reg_f);
        assert_eq!(proc.pc.reg, 0x1234);
        assert_eq!(proc.sp, 0x6969);
        assert_eq!(mem.read(0x6969 - 1), 0x0);
        assert_eq!(mem.read(0x6969 - 2), 0x0);
        proc.reg_f = CpuFlags::empty();
        Cpu::call_nflag_16(&mut mem, CpuFlags::FLAG_Z, 0x56, 0x78, &mut proc.pc, &mut proc.sp, &mut proc.reg_f);
        assert_eq!(proc.pc.reg, 0x5678);
        assert_eq!(proc.sp, 0x6967);
        assert_eq!(mem.read(0x6969 - 1), 0x12);
        assert_eq!(mem.read(0x6969 - 2), 0x34);
    }

    #[test]
    fn test_ret()
    {
        let mut proc = Cpu::new();
        let mut mem = Ram::new();
        mem.write(0x6968, 0x12);
        mem.write(0x6967, 0x34);
        proc.sp = 0x6967;
        Cpu::ret(&mut mem, &mut proc.pc, &mut proc.sp);
        assert_eq!(proc.pc.reg, 0x1234);
        assert_eq!(proc.sp, 0x6969);
    }

    #[test]
    fn test_ret_flag()
    {
        let mut proc = Cpu::new();
        let mut mem = Ram::new();
        mem.write(0x6968, 0x12);
        mem.write(0x6967, 0x34);
        proc.sp = 0x6967;
        proc.reg_f = CpuFlags::empty();
        Cpu::ret_flag(&mut mem, &mut proc.pc, &mut proc.sp, CpuFlags::FLAG_Z, &mut proc.reg_f);
        assert_eq!(proc.pc.reg, 0);
        assert_eq!(proc.sp, 0x6967);
        proc.reg_f = CpuFlags::FLAG_Z;
        Cpu::ret_flag(&mut mem, &mut proc.pc, &mut proc.sp, CpuFlags::FLAG_Z, &mut proc.reg_f);
        assert_eq!(proc.pc.reg, 0x1234);
        assert_eq!(proc.sp, 0x6969);
    }

    #[test]
    fn test_ret_nflag()
    {
        let mut proc = Cpu::new();
        let mut mem = Ram::new();
        mem.write(0x6968, 0x12);
        mem.write(0x6967, 0x34);
        proc.sp = 0x6967;
        proc.reg_f = CpuFlags::FLAG_Z;
        Cpu::ret_nflag(&mut mem, &mut proc.pc, &mut proc.sp, CpuFlags::FLAG_Z, &mut proc.reg_f);
        assert_eq!(proc.pc.reg, 0);
        assert_eq!(proc.sp, 0x6967);
        proc.reg_f = CpuFlags::empty();
        Cpu::ret_nflag(&mut mem, &mut proc.pc, &mut proc.sp, CpuFlags::FLAG_Z, &mut proc.reg_f);
        assert_eq!(proc.pc.reg, 0x1234);
        assert_eq!(proc.sp, 0x6969);
    }
}