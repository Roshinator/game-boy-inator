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
        //INCS cant be performed, flags not modified
    }
}