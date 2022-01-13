#[cfg(test)]
mod tests
{
    use super::super::{ram::Ram, cpu::Cpu};

    #[test]
    fn ram_write()
    {
        let mut ram = Ram::new();
        ram.write(0x0420, 69);
        assert_eq!(ram.read(0x0420), 69);
    }
}