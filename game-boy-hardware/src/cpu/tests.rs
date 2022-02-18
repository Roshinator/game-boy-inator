#[cfg(test)]
mod tests
{
    use crate::cpu::{self, *};

    #[test]
    fn test_aux_read_bit()
    {
        let mut cpu = Cpu::new();
        cpu.regs[cpu::REG_F] = 0b01000000;
        let true_flag = cpu.aux_read_flag(6);
        assert_eq!(true_flag, true);
        cpu.regs[cpu::REG_F] = 0;
        let false_flag = cpu.aux_read_flag(6);
        assert_eq!(false_flag, false);
    }

    #[test]
    fn test_aux_write_bit()
    {
        let mut cpu = Cpu::new();
        cpu.regs[cpu::REG_F] = 0;
        cpu.aux_write_flag(6, true);
        assert_eq!(cpu.regs[cpu::REG_F], 0b01000000);
        cpu.aux_write_flag(6, false);
        assert_eq!(cpu.regs[cpu::REG_F], 0);
    }

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
        assert_eq!(cpu.regs[REG_L], 0xFF);
        //CHECK CARRY FLAGS
    }
}