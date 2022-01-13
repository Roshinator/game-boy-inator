use std::ops::{Index, IndexMut};

use super::mainboard::Mainboard;

//DIV: Divider
pub const DIV:u16 = 0xFF04;
//TIMA: Timer counter
pub const TIMA:u16 = 0xFF05;
//TMA: Timer modulo register
pub const TMA:u16 = 0xFF06;
//TAC: Timer control register
pub const TAC:u16 = 0xFF07;
//Interrupt request
pub const IF:u16 = 0xFF0F;
//Interrupt enable
pub const IE:u16 = 0xFFFF;

pub const INTERRUPT_VB:u8 = 1 << 0;
pub const INTERRUPT_LCDC:u8 = 1 << 1;
pub const INTERRUPT_TIMA:u8 = 1 << 2;
pub const INTERRUPT_SIO_TRANSFER_COMPLETE:u8 = 1 << 3;
pub const INTERRUPT_P1X_NEG_EDGE:u8 = 1 << 4;

#[derive(Clone, Copy)]
pub struct Ram
{
    mem: [u8;0x10000]
}

impl Ram
{
    pub fn new() -> Ram
    {
        Ram { mem: [0; 0x10000] }
    }

    pub fn write(&mut self, address: u16, data: u8)
    {
        self.mem[address as usize] = data;
    }

    pub fn write_rp(&mut self, msh: u8, lsh: u8, data: u8)
    {
        self.mem[u16::from_le_bytes([msh, lsh]) as usize] = data;
    }

    pub fn read(&self, address: u16) -> u8
    {
        self.mem[address as usize]
    }

    pub fn read_rp(&self, msh: u8, lsh: u8) -> u8
    {
        self.mem[u16::from_le_bytes([msh, lsh]) as usize]
    }

    //Interrupts
    //bit 0: vblank
    //bit 1: LCDC (STAT References)
    //bit 2: Timer overflow
    //bit 3: Serial I/O transfer complete
    //bit 4: P10-P13 negative edge

    pub fn set_interrupt(&mut self, interrupt: u8)
    {
        self.mem[IF as usize] |= interrupt;
    }

    pub fn reset_interrupt(&mut self, interrupt: u8)
    {
        self.mem[IF as usize] = !(!self.mem[IF as usize] | interrupt);
    }

    pub fn test(&mut self, mb: &Mainboard)
    {

    }

    //Timers
    

}

impl Index<u16> for Ram
{
    type Output = u8;

    fn index(&self, index: u16) -> &u8
    {
        &self.mem[index as usize]
    }
}

// impl IndexMut<u16> for Ram
// {
//     fn index_mut(&mut self, index: u16) -> &mut u8
//     {
//         &mut self.mem[index as usize]
//     }
// }