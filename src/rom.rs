use core::panic;
use std::{fs::File, io::Read};

pub const BOOT_ROM:[u8;256] =
[
    0x31,0xFE,0xFF,0xAF,0x21,0xFF,0x9F,0x32,0xCB,0x7C,0x20,0xFB,0x21,0x26,0xFF,0x0E,
    0x11,0x3E,0x80,0x32,0xE2,0x0C,0x3E,0xF3,0xE2,0x32,0x3E,0x77,0x77,0x3E,0xFC,0xE0,
    0x47,0x11,0x04,0x01,0x21,0x10,0x80,0x1A,0xCD,0x95,0x00,0xCD,0x96,0x00,0x13,0x7B,
    0xFE,0x34,0x20,0xF3,0x11,0xD8,0x00,0x06,0x08,0x1A,0x13,0x22,0x23,0x05,0x20,0xF9,
    0x3E,0x19,0xEA,0x10,0x99,0x21,0x2F,0x99,0x0E,0x0C,0x3D,0x28,0x08,0x32,0x0D,0x20,
    0xF9,0x2E,0x0F,0x18,0xF3,0x67,0x3E,0x64,0x57,0xE0,0x42,0x3E,0x91,0xE0,0x40,0x04,
    0x1E,0x02,0x0E,0x0C,0xF0,0x44,0xFE,0x90,0x20,0xFA,0x0D,0x20,0xF7,0x1D,0x20,0xF2,
    0x0E,0x13,0x24,0x7C,0x1E,0x83,0xFE,0x62,0x28,0x06,0x1E,0xC1,0xFE,0x64,0x20,0x06,
    0x7B,0xE2,0x0C,0x3E,0x87,0xE2,0xF0,0x42,0x90,0xE0,0x42,0x15,0x20,0xD2,0x05,0x20,
    0x4F,0x16,0x20,0x18,0xCB,0x4F,0x06,0x04,0xC5,0xCB,0x11,0x17,0xC1,0xCB,0x11,0x17,
    0x05,0x20,0xF5,0x22,0x23,0x22,0x23,0xC9,0xCE,0xED,0x66,0x66,0xCC,0x0D,0x00,0x0B,
    0x03,0x73,0x00,0x83,0x00,0x0C,0x00,0x0D,0x00,0x08,0x11,0x1F,0x88,0x89,0x00,0x0E,
    0xDC,0xCC,0x6E,0xE6,0xDD,0xDD,0xD9,0x99,0xBB,0xBB,0x67,0x63,0x6E,0x0E,0xEC,0xCC,
    0xDD,0xDC,0x99,0x9F,0xBB,0xB9,0x33,0x3E,0x3C,0x42,0xB9,0xA5,0xB9,0xA5,0x42,0x3C,
    0x21,0x04,0x01,0x11,0xA8,0x00,0x1A,0x13,0xBE,0x20,0xFE,0x23,0x7D,0xFE,0x34,0x20,
    0xF5,0x06,0x19,0x78,0x86,0x23,0x05,0x20,0xFB,0x86,0x20,0xFE,0x3E,0x01,0xE0,0x50
];

#[derive(Clone, PartialEq, Debug)]
pub enum MBCModel {MbcNone, Mbc1_16_8, Mbc2, Mbc3, Mbc5}

#[derive(Clone)]
pub struct Rom
{
    pub bytes: Vec<u8>,
    pub mbc_model: MBCModel,
    pub rom_size: usize,
    pub ram_size: usize,
    pub has_battery: bool,
    pub has_rtc: bool,
    pub has_rumble: bool,
    pub title: String
}

impl Rom
{
    pub fn new(mut f: File) -> Rom
    {
        let mut bytes = Vec::new();
        let _result = f.read_to_end(&mut bytes);
        let mut rom = Rom
        {
            bytes: bytes,
            mbc_model: MBCModel::MbcNone,
            rom_size: 0,
            ram_size: 0,
            has_battery: false,
            has_rtc: false,
            has_rumble: false,
            title: Default::default()
        };

        let mut ttl:Vec<u8> = Vec::<u8>::new();
        for i in 0..16
        {
            let x = rom.bytes[0x134 + i];
            if x <= 127
            {
                if !x.is_ascii_control()
                {
                    ttl.push(x);
                }
            }
        }
        rom.title = match String::from_utf8(ttl)
        {
            Ok(res) => res,
            Err(_) => String::from("Unknown")
        };

        rom.mbc_model = match rom.bytes[0x147]
        {
            0x00 | 0x08 => MBCModel::MbcNone,
            0x01 | 0x02 => MBCModel::Mbc1_16_8,
            0x03 => {rom.has_battery = true; MBCModel::Mbc1_16_8},
            0x05 => MBCModel::Mbc2,
            0x06 => {rom.has_battery = true; MBCModel::Mbc2},
            0x09 => {rom.has_battery = true; MBCModel::MbcNone},
            0x0F | 0x10 => {rom.has_battery = true; rom.has_rtc = true; MBCModel::Mbc3},
            0x11 | 0x12 => {MBCModel::Mbc3},
            0x13 => {rom.has_battery = true; MBCModel::Mbc3},
            0x19 | 0x1A => MBCModel::Mbc5,
            0x1B => {rom.has_battery = true; MBCModel::Mbc5},
            0x1C | 0x1D => {rom.has_rumble = true; MBCModel::Mbc5},
            0x1E => {rom.has_battery = true; rom.has_rumble = true; MBCModel::Mbc5},
            _ => panic!("Unsupported hardware in ROM")
        };

        rom.rom_size = match rom.bytes[0x148] //Values are address count (*8 for size)
        {
            0x00 => 32768, /* 256 Kbit */
            0x01 => 65536, /* 512 Kbit */
            0x02 => 131072, /* 1 Mbit */
            0x03 => 262144, /* 2 Mbit */
            0x04 => 524288, /* 4 Mbit */
            0x05 => 1048576, /* 8 Mbit */
            0x06 => 2097152, /* 16 Mbit */
            0x52 => 1179648, /* 9 Mbit */
            0x53 => 1310720, /* 10 Mbit */
            0x54 => 1572864, /* 12 Mbit */
            _ => panic!("Invalid rom size")
        };

        rom.ram_size = match rom.bytes[0x149] //Values are address count (*8 for size)
        {
            _ if rom.mbc_model == MBCModel::Mbc2 => 512,
            0 => 0,
            1 => 2048,
            2 => 8192,
            3 => 32768,
            4 => 131072,
            _ => panic!("Invalid ram size")
        };

        //Check checksum here

        //Print ROM details
        println!("{:?}", rom);
        return rom;
    }
}

impl std::fmt::Debug for Rom
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.debug_struct("Rom").field("mbc_model", &self.mbc_model).field("rom_size", &self.rom_size).field("ram_size", &self.ram_size).field("has_battery", &self.has_battery).field("has_rtc", &self.has_rtc).field("has_rumble", &self.has_rumble).field("title", &self.title).finish()
    }
}