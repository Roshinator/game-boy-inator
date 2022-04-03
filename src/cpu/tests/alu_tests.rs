#[cfg(test)]
mod alu_tests
{
    use crate::cpu::{self, *};

    #[test]
    fn test_aux_inc_16()
    {
        let mut cpu = Cpu::new();
        let result = cpu.aux_inc_16(0x12, 0x34);
        assert_eq!(result, (0x35, 0x12));
        let result2 = cpu.aux_inc_16(0x00, 0xFF);
        assert_eq!(result2, (0x00, 0x01));
        let result3 = cpu.aux_inc_16(0xFF, 0xFF);
        assert_eq!(result3, (0x00, 0x00));
    }

    #[test]
    fn test_inc_r16()
    {
        let mut cpu = Cpu::new();
        let [lsh, msh] = u16::to_le_bytes(0xABCD);
        let flags_before = cpu.regs[REG_F];
        cpu.regs[REG_H] = msh;
        cpu.regs[REG_L] = lsh;
        cpu.inc_r16(REG_H, REG_L);
        let [lsh_after, msh_after] = u16::to_le_bytes(0xABCE);
        assert_eq!(msh_after, cpu.regs[REG_H]);
        assert_eq!(lsh_after, cpu.regs[REG_L]);
        assert_eq!(flags_before, cpu.regs[REG_F]);
    }

    #[test]
    fn test_inc_sp()
    {
        let mut cpu = Cpu::new();
        let flags_before = cpu.regs[REG_F];
        cpu.sp = 0xABCD;
        cpu.inc_sp();
        assert_eq!(cpu.sp, 0xABCE);
        assert_eq!(flags_before, cpu.regs[REG_F]);
    }

    #[test]
    fn test_inc_r8()
    {
        let mut cpu = Cpu::new();
        todo!();
        //INCS cant be performed, flags not modified
    }

    #[test]
    fn test_dec_r8()
    {
        let mut cpu = Cpu::new();
        todo!();
        //INCS cant be performed, flags not modified
    }

    #[test]
    fn test_dec_r16()
    {
        let mut cpu = Cpu::new();
        let [lsh, msh] = u16::to_le_bytes(0xABCD);
        let flags_before = cpu.regs[REG_F];
        cpu.regs[REG_H] = msh;
        cpu.regs[REG_L] = lsh;
        cpu.dec_r16(REG_H, REG_L);
        let [lsh_after, msh_after] = u16::to_le_bytes(0xABCC);
        assert_eq!(msh_after, cpu.regs[REG_H]);
        assert_eq!(lsh_after, cpu.regs[REG_L]);
        assert_eq!(flags_before, cpu.regs[REG_F]);
    }

    #[test]
    fn test_dec_sp()
    {
        let mut cpu = Cpu::new();
        let flags_before = cpu.regs[REG_F];
        cpu.sp = 0xABCD;
        cpu.dec_sp();
        assert_eq!(cpu.sp, 0xABCC);
        assert_eq!(flags_before, cpu.regs[REG_F]);
    }

    #[test]
    fn test_add_r8_r8()
    {
        let mut cpu = Cpu::new();
        //Test add
        cpu.regs[REG_A] = 0b00000111;
        cpu.regs[REG_B] = 1;
        cpu.add_r8_r8(REG_A, REG_B);
        assert_eq!(cpu.regs[REG_A], 8);
        //Test flag H (ZNHC0000)
        cpu.regs[REG_A] = 1;
        cpu.regs[REG_B] = 0b00001111;
        cpu.add_r8_r8(REG_A, REG_B);
        assert_eq!(cpu.regs[REG_A], 16);
        assert_eq!(cpu.regs[REG_F], 0b00100000);
        //Test flag Z
        cpu.regs[REG_A] = 0;
        cpu.regs[REG_B] = 0;
        cpu.add_r8_r8(REG_A, REG_B);
        assert_eq!(cpu.regs[REG_A], 0);
        assert_eq!(cpu.regs[REG_F], 0b10000000);
        //Test flag C
        cpu.regs[REG_A] = 0b11111111;
        cpu.regs[REG_B] = 2;
        cpu.add_r8_r8(REG_A, REG_B);
        assert_eq!(cpu.regs[REG_A], 1);
        assert_eq!(cpu.regs[REG_F], 0b00110000); //Half carry also occurs
    }

    #[test]
    fn test_add_r8_8()
    {
        let mut cpu = Cpu::new();
        //Test add
        cpu.regs[REG_A] = 0b00000111;
        cpu.add_r8_8(REG_A, 1);
        assert_eq!(cpu.regs[REG_A], 8);
        //Test flag H (ZNHC0000)
        cpu.regs[REG_A] = 1;
        cpu.add_r8_8(REG_A, 0b00001111);
        assert_eq!(cpu.regs[REG_A], 16);
        assert_eq!(cpu.regs[REG_F], 0b00100000);
        //Test flag Z
        cpu.regs[REG_A] = 0;
        cpu.add_r8_8(REG_A, 0);
        assert_eq!(cpu.regs[REG_A], 0);
        assert_eq!(cpu.regs[REG_F], 0b10000000);
        //Test flag C
        cpu.regs[REG_A] = 0b11111111;
        cpu.add_r8_8(REG_A, 2);
        assert_eq!(cpu.regs[REG_A], 1);
        assert_eq!(cpu.regs[REG_F], 0b00110000); //Half carry also occurs
    }

    #[test]
    fn test_add_r16_r16()
    {
        let mut cpu = Cpu::new();
        //Test add
        cpu.regs[REG_H] = 0;
        cpu.regs[REG_L] = 0b11111111;
        cpu.regs[REG_B] = 0;
        cpu.regs[REG_C] = 0b00000001;
        cpu.regs[REG_F] = 0;
        cpu.add_r16_r16(REG_H, REG_L, REG_B, REG_C);
        assert_eq!(cpu.regs[REG_H], 1);
        assert_eq!(cpu.regs[REG_L], 0);
        assert_eq!(cpu.regs[REG_F], 0b00000000);
        //Test Carry and Zero (Zero should be unchanged)
        cpu.regs[REG_H] = 0b11111111;
        cpu.regs[REG_L] = 0b11111111;
        cpu.regs[REG_B] = 0;
        cpu.regs[REG_C] = 0b00000001;
        cpu.regs[REG_F] = 0;
        cpu.add_r16_r16(REG_H, REG_L, REG_B, REG_C);
        assert_eq!(cpu.regs[REG_H], 0);
        assert_eq!(cpu.regs[REG_L], 0);
        assert_eq!(cpu.regs[REG_F], 0b00110000);
        //Test Half Carry
        cpu.regs[REG_H] = 0b00001111;
        cpu.regs[REG_L] = 0b11111111;
        cpu.regs[REG_B] = 0;
        cpu.regs[REG_C] = 0b00000001;
        cpu.regs[REG_F] = 0;
        cpu.add_r16_r16(REG_H, REG_L, REG_B, REG_C);
        assert_eq!(cpu.regs[REG_H], 0b00010000);
        assert_eq!(cpu.regs[REG_L], 0);
        assert_eq!(cpu.regs[REG_F], 0b00100000);
    }

    #[test]
    fn test_and_r8_r8()
    {
        let mut cpu = Cpu::new();
        cpu.regs[REG_A] = 0b10101011;
        cpu.regs[REG_B] = 0b01010101;
        cpu.and_r8_r8(REG_A, REG_B);
        assert_eq!(cpu.regs[REG_A], 1);
        assert_eq!(cpu.regs[REG_F], 0b00100000);
        cpu.regs[REG_B] = 0b01010100;
        cpu.and_r8_r8(REG_A, REG_B);
        assert_eq!(cpu.regs[REG_F], 0b10100000);
    }

    #[test]
    fn test_and_r8_8()
    {
        let mut cpu = Cpu::new();
        cpu.regs[REG_A] = 0b10101011;
        cpu.and_r8_8(REG_A, 0b01010101);
        assert_eq!(cpu.regs[REG_A], 1);
        assert_eq!(cpu.regs[REG_F], 0b00100000);
        cpu.and_r8_8(REG_A, 0b01010100);
        assert_eq!(cpu.regs[REG_F], 0b10100000);
    }

    #[test]
    fn test_xor_r8_r8()
    {
        let mut cpu = Cpu::new();
        cpu.regs[REG_A] = 0b10101011;
        cpu.regs[REG_B] = 0b01010101;
        cpu.xor_r8_r8(REG_A, REG_B);
        assert_eq!(cpu.regs[REG_A], 0b11111110);
        assert_eq!(cpu.regs[REG_F], 0b00000000);
        cpu.regs[REG_B] = 0b11111110;
        cpu.xor_r8_r8(REG_A, REG_B);
        assert_eq!(cpu.regs[REG_F], 0b10000000);
    }

    #[test]
    fn test_xor_r8_8()
    {
        let mut cpu = Cpu::new();
        cpu.regs[REG_A] = 0b10101011;
        cpu.xor_r8_8(REG_A, 0b01010101);
        assert_eq!(cpu.regs[REG_A], 0b11111110);
        assert_eq!(cpu.regs[REG_F], 0b00000000);
        cpu.xor_r8_8(REG_A, 0b11111110);
        assert_eq!(cpu.regs[REG_F], 0b10000000);
    }

    #[test]
    fn test_or_r8_r8()
    {
        let mut cpu = Cpu::new();
        cpu.regs[REG_A] = 0b10101011;
        cpu.regs[REG_B] = 0b01010101;
        cpu.or_r8_r8(REG_A, REG_B);
        assert_eq!(cpu.regs[REG_A], 0b11111111);
        assert_eq!(cpu.regs[REG_F], 0b00000000);
        cpu.regs[REG_A] = 0b00000000;
        cpu.regs[REG_B] = 0b00000000;
        cpu.or_r8_r8(REG_A, REG_B);
        assert_eq!(cpu.regs[REG_F], 0b10000000);
    }

    #[test]
    fn test_or_r8_8()
    {
        let mut cpu = Cpu::new();
        cpu.regs[REG_A] = 0b10101011;
        cpu.or_r8_8(REG_A, 0b01010101);
        assert_eq!(cpu.regs[REG_A], 0b11111111);
        assert_eq!(cpu.regs[REG_F], 0b00000000);
        cpu.regs[REG_A] = 0b00000000;
        cpu.or_r8_8(REG_A, 0b00000000);
        assert_eq!(cpu.regs[REG_F], 0b10000000);
    }

    #[test]
    fn test_cpl()
    {
        let mut cpu = Cpu::new();
        cpu.regs[REG_A] = 0b10101010;
        cpu.cpl();
        assert_eq!(cpu.regs[REG_A], 0b01010101);
        assert!(cpu.aux_read_flag(CpuFlags::FLAG_H));
        assert!(cpu.aux_read_flag(CpuFlags::FLAG_N));
    }

    #[test]
    fn test_ccf()
    {
        let mut cpu = Cpu::new();
        cpu.regs[REG_F] = CpuFlags::FLAG_C.bits;
        cpu.ccf();
        assert!(!cpu.aux_read_flag(CpuFlags::FLAG_C));
    }

    #[test]
    fn test_scf()
    {
        let mut cpu = Cpu::new();
        cpu.regs[REG_F] = CpuFlags::empty().bits;
        cpu.scf();
        assert!(cpu.aux_read_flag(CpuFlags::FLAG_C));
    }
}