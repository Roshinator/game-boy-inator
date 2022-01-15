use std::ops::{RangeInclusive};

use super::{rom::{self, Rom}};

//----Timer Registers----
//DIV: Divider
pub const DIV:u16 = 0xFF04;
//TIMA: Timer counter
pub const TIMA:u16 = 0xFF05;
//TMA: Timer modulo register
pub const TMA:u16 = 0xFF06;
//TAC: Timer control register
pub const TAC:u16 = 0xFF07;

//----LCD Registers----
pub const LCDC:u16 = 0xFF40;
pub const STAT:u16 = 0xFF41;
pub const SCY:u16 = 0xFF42;
pub const SCX:u16 = 0xFF43;
pub const LY:u16 = 0xFF44;
pub const LYC:u16 = 0xFF45;
pub const BGP:u16 = 0xFF47;
pub const OBP0:u16 = 0xFF48;
pub const OBP1:u16 = 0xFF49;
pub const WY:u16 = 0xFF4A;
pub const WX:u16 = 0xFF4B;
pub const OAM:RangeInclusive<u16> = 0xFE00..=0xFE9F;

//----Interrupt Registers----
//Interrupt request
pub const IF:u16 = 0xFF0F;
//Interrupt enable
pub const IE:u16 = 0xFFFF;

//----Interrupt Bits----
pub const INTERRUPT_VB:u8 = 1 << 0;
pub const INTERRUPT_LCDC:u8 = 1 << 1;
pub const INTERRUPT_TIMA:u8 = 1 << 2;
pub const INTERRUPT_SIO_TRANSFER_COMPLETE:u8 = 1 << 3;
pub const INTERRUPT_P1X_NEG_EDGE:u8 = 1 << 4;

pub const SC_BOOT_ROM_DISABLE:u16 = 0xFF50;

#[derive(Clone)]
pub struct Ram
{
    mem: [u8;0x10000],
    rom: Rom
}

impl Ram
{
    pub fn new(rom: Rom) -> Ram
    {
        Ram
        {
            mem: [0; 0x10000],
            rom: rom
        }
    }

    pub fn write(&mut self, address: u16, data: u8)
    {
        match address
        {
            //Boot rom disable
            SC_BOOT_ROM_DISABLE => {self.rom.boot_rom_enabled = false;},
            _ => {}
        }

        self.mem[address as usize] = data;
    }

    pub fn write_rp(&mut self, msh: u8, lsh: u8, data: u8)
    {
        self.write(u16::from_le_bytes([msh, lsh]), data);
    }

    pub fn read(&self, address: u16) -> u8
    {
        match address
        {
            0x0000..=0x0100 =>
            {
                if self.rom.boot_rom_enabled
                {
                    return rom::BOOT_ROM[address as usize];
                }
                self.mem[address as usize]
            },
            _ => self.mem[address as usize]
        }
    }

    pub fn read_rp(&self, msh: u8, lsh: u8) -> u8
    {
        self.read(u16::from_le_bytes([msh, lsh]))
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
}

// impl Index<u16> for Ram
// {
//     type Output = u8;

//     fn index(&self, index: u16) -> &u8
//     {
//         &self.mem[index as usize]
//     }
// }

// impl IndexMut<u16> for Ram
// {
//     fn index_mut(&mut self, index: u16) -> &mut u8
//     {
//         &mut self.mem[index as usize]
//     }
// }