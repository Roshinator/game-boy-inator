#[cfg(test)]
mod control_tests
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
}