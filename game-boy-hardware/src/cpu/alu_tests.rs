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
}