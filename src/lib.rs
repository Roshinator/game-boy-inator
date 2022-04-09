use std::{cell::RefCell, rc::Rc};

pub mod cpu;
pub mod ram;
pub mod mainboard;
pub mod ppu;

mod rom;
mod timer;

#[cfg(test)]
mod tests;

type HardwareHandle = Rc<RefCell<dyn crate::Frontend>>;
pub trait Frontend
{
    fn receive_rom_information(&mut self, title: &str);
    fn event_poll(&mut self);
    fn video_update(&mut self, buffer: &[[u8; ppu::SCREEN_HEIGHT];ppu::SCREEN_WIDTH]);
}