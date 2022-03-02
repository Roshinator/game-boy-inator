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
    #[ignore = "INCS cant be performed, flags not modified"]
    fn test_inc_r16()
    {
        let mut cpu = Cpu::new();
        todo!();
    }

    #[test]
    #[ignore = "INCS cant be performed, flags not modified"]
    fn test_inc_sp()
    {
        let mut cpu = Cpu::new();
        todo!();
        //INCS cant be performed, flags not modified
    }

    #[test]
    #[ignore = "INCS cant be performed, flags not modified"]
    fn test_inc_r8()
    {
        let mut cpu = Cpu::new();
        todo!();
        //INCS cant be performed, flags not modified
    }

    #[test]
    #[ignore = "INCS cant be performed, flags not modified"]
    fn test_inc_r16a()
    {
        let mut cpu = Cpu::new();
        todo!();
        //INCS cant be performed, flags not modified
    }

    #[test]
    #[ignore = "INCS cant be performed, flags not modified"]
    fn test_dec_r8()
    {
        let mut cpu = Cpu::new();
        todo!();
        //INCS cant be performed, flags not modified
    }

    #[test]
    #[ignore = "INCS cant be performed, flags not modified"]
    fn test_dec_r16a()
    {
        let mut cpu = Cpu::new();
        todo!();
        //INCS cant be performed, flags not modified
    }

    #[test]
    #[ignore = "INCS cant be performed, flags not modified"]
    fn test_dec_r16()
    {
        let mut cpu = Cpu::new();
        todo!();
        //INCS cant be performed, flags not modified
    }

    #[test]
    #[ignore = "INCS cant be performed, flags not modified"]
    fn test_dec_sp()
    {
        let mut cpu = Cpu::new();
        todo!();
        //INCS cant be performed, flags not modified
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
    fn test_and_r8_r16a()
    {
        let mut cpu = Cpu::new();
        let mut ram = Ram::new();
        cpu.regs[REG_A] = 0b10101011;
        cpu.regs[REG_B] = 0x69;
        cpu.regs[REG_C] = 0x42;
        ram.write_rp(0x69, 0x42, 0b01010101);
        cpu.and_r8_r16a(&mut ram, REG_A, REG_B, REG_C);
        assert_eq!(cpu.regs[REG_A], 1);
        assert_eq!(cpu.regs[REG_F], 0b00100000);
        ram.write_rp(0x69, 0x42, 0b01010100);
        cpu.and_r8_r16a(&mut ram, REG_A, REG_B, REG_C);
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
    fn test_xor_r8_r16a()
    {
        let mut cpu = Cpu::new();
        let mut ram = Ram::new();
        cpu.regs[REG_A] = 0b10101011;
        cpu.regs[REG_B] = 0x69;
        cpu.regs[REG_C] = 0x42;
        ram.write_rp(0x69, 0x42, 0b01010101);
        cpu.xor_r8_r16a(&mut ram, REG_A, REG_B, REG_C);
        assert_eq!(cpu.regs[REG_A], 0b11111110);
        assert_eq!(cpu.regs[REG_F], 0b00000000);
        ram.write_rp(0x69, 0x42, 0b11111110);
        cpu.xor_r8_r16a(&mut ram, REG_A, REG_B, REG_C);
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
    fn test_or_r8_r16a()
    {
        let mut cpu = Cpu::new();
        let mut ram = Ram::new();
        cpu.regs[REG_A] = 0b10101011;
        cpu.regs[REG_B] = 0x69;
        cpu.regs[REG_C] = 0x42;
        ram.write_rp(0x69, 0x42, 0b01010101);
        cpu.or_r8_r16a(&mut ram, REG_A, REG_B, REG_C);
        assert_eq!(cpu.regs[REG_A], 0b11111111);
        assert_eq!(cpu.regs[REG_F], 0b00000000);
        cpu.regs[REG_A] = 0b00000000;
        ram.write_rp(0x69, 0x42, 0b00000000);
        cpu.or_r8_r16a(&mut ram, REG_A, REG_B, REG_C);
        assert_eq!(cpu.regs[REG_F], 0b10000000);
    }
}