pub struct Ram
{
    mem: [u8;0xFFFF]
}

impl Ram
{
    pub fn new() -> Ram
    {
        return Ram { mem: [0; 0xFFFF] }
    }

    #[inline]
    pub fn write_to_address(&mut self, address: usize, data: u8)
    {
        self.mem[address] = data;
    }

    #[inline]
    pub fn write_to_address_reg_pair(&mut self, msh: u8, lsh: u8, data: u8)
    {
        self.mem[u16::from_le_bytes([msh, lsh]) as usize] = data;
    }

    #[inline]
    pub fn read_from_address(&self, address: usize) -> u8
    {
        self.mem[address]
    }

    #[inline]
    pub fn read_from_address_reg_pair(&self, msh: u8, lsh: u8) -> u8
    {
        self.mem[u16::from_le_bytes([msh, lsh]) as usize]
    }
}