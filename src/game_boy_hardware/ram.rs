use std::ops::{Index, IndexMut};
use std::slice::SliceIndex;

#[derive(Clone, Copy)]
pub struct Ram
{
    mem: [u8;0xFFFF]
}

impl Ram
{
    pub fn new() -> Ram
    {
        Ram { mem: [0; 0xFFFF] }
    }

    pub fn write_to_address(&mut self, address: u16, data: u8)
    {
        self.mem[address as usize] = data;
    }

    pub fn write_to_address_rp(&mut self, msh: u8, lsh: u8, data: u8)
    {
        self.mem[u16::from_le_bytes([msh, lsh]) as usize] = data;
    }

    pub fn read_from_address(&self, address: u16) -> u8
    {
        self.mem[address as usize]
    }

    pub fn read_from_address_rp(&self, msh: u8, lsh: u8) -> u8
    {
        self.mem[u16::from_le_bytes([msh, lsh]) as usize]
    }
}

impl<Idx> Index<Idx> for Ram where Idx: SliceIndex<[u8]>
{
    type Output = Idx::Output;

    fn index(&self, index: Idx) -> &Self::Output 
    {
        &self.mem[index]
    }
}

impl<Idx> IndexMut<Idx> for Ram where Idx: SliceIndex<[u8]>
{
    fn index_mut(&mut self, index: Idx) -> &mut Self::Output 
    {
        &mut self.mem[index]
    }
}