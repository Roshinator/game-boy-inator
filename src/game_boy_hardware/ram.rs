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
pub const DMA:u16 = 0xFF46;
pub const BGP:u16 = 0xFF47;
pub const OBP0:u16 = 0xFF48;
pub const OBP1:u16 = 0xFF49;
pub const WY:u16 = 0xFF4A;
pub const WX:u16 = 0xFF4B;
pub const OAM:RangeInclusive<u16> = 0xFE00..=0xFE9F; //OAM slot is 4 bytes, 0=Ycoord, 1=Xcoord, 2=TileIdx, 3=Attributes (b4=pallete, b5=Xflip, b6=Yflip, b7=priority)
pub const OBJ1:RangeInclusive<u16> = 0x8000..=0x8FFF;
pub const OBJ2:RangeInclusive<u16> = 0x8800..=0x97FF;
pub const VRAM1:RangeInclusive<u16> = 0x9800..=0x9BFF;
pub const VRAM2:RangeInclusive<u16> = 0x9C00..=0x9FFF;

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
    rom: Rom,
    dma: Dma
}
#[derive(Clone)]
struct Dma
{
    delay_start: bool,
    pending_source: u8,
    source: u16,
    active: bool
}

impl Ram
{
    pub fn new(rom: Rom) -> Ram
    {
        let mut ram = Ram
        {
            mem: [0; 0x10000],
            rom: rom,
            dma: Dma { delay_start: false, pending_source: 0, source: 0, active: false }
        };
        ram.mem[0x0000..=0x3FFF].copy_from_slice(&ram.rom.bytes[0x0000..=0x3FFF]);
        return ram;
    }

    pub fn write(&mut self, address: u16, data: u8)
    {
        match address
        {
            //Boot rom disable
            SC_BOOT_ROM_DISABLE => {self.rom.boot_rom_enabled = false;},
            DMA =>
            {
                if data < 0xF1
                {
                    self.dma.pending_source = data;
                    self.dma.delay_start = true;
                }
            }
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

    pub fn execute(&mut self)
    {
        self.dma_update();
    }

    fn dma_update(&mut self)
    {
        if self.dma.pending_source != 0
        {
            if !self.dma.delay_start
            {
                self.dma.source = (self.dma.pending_source as u16) << 8;
                self.dma.pending_source = 0;
            }
            self.dma.delay_start = false;
        }

        if self.dma.source != 0 && (self.dma.source & 0xFF) < 160
        {
            self.dma.active = true;
            self.write(self.dma.source & 0xFF, self.read(self.dma.source));
            self.dma.source += 1;
        }
        else
        {
            self.dma.active = false;    
        }
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