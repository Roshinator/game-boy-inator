pub struct RAM
{
    mem: [u8;0xFFFF]
}

impl RAM
{
    #[inline]
    pub fn writeToAddress(&mut self, address: usize, data: u8)
    {
        self.mem[address] = data
    }

    #[inline]
    pub fn readFromAddress(&self, address: usize) -> u8
    {
        self.mem[address]
    }
}