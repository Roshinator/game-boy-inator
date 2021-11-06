pub struct Ram
{
    mem: [u8;0xFFFF]
}

impl Ram
{
    #[inline]
    pub fn write_to_address(&mut self, address: usize, data: u8)
    {
        self.mem[address] = data
    }

    #[inline]
    pub fn read_from_address(&self, address: usize) -> u8
    {
        self.mem[address]
    }
}