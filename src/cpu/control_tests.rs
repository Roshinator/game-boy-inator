#[cfg(test)]
mod control_tests
{
    use crate::cpu::{self, *};

    #[test]
    fn test_aux_read_bit()
    {
        let mut cpu = Cpu::new();
        cpu.regs[cpu::REG_F] = CpuFlags::FLAG_H.bits;
        let true_flag = cpu.aux_read_flag(CpuFlags::FLAG_H);
        assert_eq!(true_flag, true);
        cpu.regs[cpu::REG_F] = 0;
        let false_flag = cpu.aux_read_flag(CpuFlags::FLAG_H);
        assert_eq!(false_flag, false);
    }

    #[test]
    fn test_aux_write_bit()
    {
        let mut cpu = Cpu::new();
        cpu.regs[cpu::REG_F] = 0;
        cpu.aux_write_flag(CpuFlags::FLAG_N, true);
        assert_eq!(cpu.regs[cpu::REG_F], CpuFlags::FLAG_N.bits);
        cpu.aux_write_flag(CpuFlags::FLAG_N, false);
        assert_eq!(cpu.regs[cpu::REG_F], CpuFlags::empty().bits);
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
    fn test_aux_inc_pc()
    {
        let mut cpu = Cpu::new();
        cpu.pc.current_instruction_width = 3;
        cpu.pc.should_increment = false;
        cpu.aux_inc_pc();
        assert_eq!(cpu.pc.reg, 0x0000);
        assert!(cpu.pc.current_instruction_width != 0);
        cpu.pc.should_increment = true;
        cpu.aux_inc_pc();
        assert_eq!(cpu.pc.reg, 0x0001);
        assert!(cpu.pc.current_instruction_width == 0);
    }

    #[test]
    fn test_aux_read_pc()
    {
        let mut cpu = Cpu::new();
        let mut ram = Ram::new();
        ram.write(0x5050, 0x12);
        cpu.pc.reg = 0x5050;
        let result = cpu.aux_read_pc(&mut ram);
        assert_eq!(result, 0x12);
    }

    #[test]
    fn aux_read_immediate_data()
    {
        let mut cpu = Cpu::new();
        let mut ram = Ram::new();
        ram.write(0x5050, 0x01);
        ram.write(0x5051, 0x02);
        cpu.pc.reg = 0x5050;
        let result = cpu.aux_read_immediate_data(&mut ram);
        assert_eq!(result, 0x02);
    }
}