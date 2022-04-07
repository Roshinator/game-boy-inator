#[cfg(test)]
mod ld_tests
{
    use crate::cpu::{self, *};

    #[test]
    fn test_ld_r16_16()
    {
        let mut proc = Cpu::new();
        Cpu::ld_r16_16(&mut proc.reg_h, &mut proc.reg_l,
            33, 36);
        assert_eq!(proc.reg_h, 33);
        assert_eq!(proc.reg_l, 36);
    }

    #[test]
    fn test_ld_hl_sp_plus()
    {
        let mut proc = Cpu::new();
        proc.sp = 0x11FF;
        Cpu::ld_hl_sp_plus(&mut proc.sp, &mut proc.reg_h, &mut proc.reg_l, 1);
        assert_eq!(proc.reg_h, 0x12);
        assert_eq!(proc.reg_l, 0x00);
        Cpu::ld_hl_sp_plus(&mut proc.sp, &mut proc.reg_h, &mut proc.reg_l,-1);
        assert_eq!(proc.reg_h, 0x11);
        assert_eq!(proc.reg_l, 0xFE);
        //CHECK CARRY FLAGS
    }

    #[test]
    fn test_ld_sp_16()
    {
        let mut proc = Cpu::new();
        proc.sp = 0;
        Cpu::ld_sp_16(&mut proc.sp, 0x69, 0x42);
        assert_eq!(proc.sp, 0x6942);
    }

    #[test]
    fn test_ld_r8_8()
    {
        let mut proc = Cpu::new();
        proc.reg_d = 0;
        Cpu::ld_r8_8(&mut proc.reg_d, 0x69);
        assert_eq!(proc.reg_d, 0x69);
    }

    #[test]
    fn test_ld_16a_sp()
    {
        let mut proc = Cpu::new();
        let mut mem = Ram::new();
        proc.sp = 0x1234;
        Cpu::ld_16a_sp(&mut proc.sp, &mut mem, 0x69, 0x42);
        assert_eq!(mem.read(0x6942), 0x34);
        assert_eq!(mem.read(0x6942 + 1), 0x12);
    }

    #[test]
    fn test_ld_r8_r8()
    {
        let mut proc = Cpu::new();
        proc.reg_a = 0;
        proc.reg_b = 0x21;
        Cpu::ld_r8_r8(&mut proc.reg_a, &mut proc.reg_b);
        assert_eq!(proc.reg_a, 0x21);
    }

    #[test]
    fn test_ld_r8_r8_s()
    {
        let mut proc = Cpu::new();
        proc.reg_a = 0x21;
        Cpu::ld_r8_r8_s(&mut proc.reg_a);
        assert_eq!(proc.reg_a, 0x21);
    }

    #[test]
    fn test_ld_sp_r16()
    {
        let mut proc = Cpu::new();
        proc.sp = 0;
        proc.reg_h = 0x69;
        proc.reg_l = 0x42;
        Cpu::ld_sp_r16(&mut proc.sp, &mut proc.reg_h, &mut proc.reg_l);
        assert_eq!(proc.sp, 0x6942);
    }
}