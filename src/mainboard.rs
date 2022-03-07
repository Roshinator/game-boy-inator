use std::{cell::{RefCell}, rc::Rc};
use crate::{cpu::Cpu, ram::Ram, rom::Rom, timer::Timer, ppu::{self, Ppu}};

pub const CLOCK_EDGE:f64 = 8_338_608_f64;

pub struct Mainboard
{
    cpu: Cpu,
    ram: Ram,
    ppu: Ppu,
    timer: Timer,
    clock_enable: bool,
    cycles: u64,
    t_cycles: u64,
    m_cycles: u64,
    hardware_handle: crate::HardwareHandle
}

impl Mainboard
{
    pub fn new<T: crate::Frontend + 'static>(hardware_handle: T) -> Mainboard
    {
        Mainboard
        {
            cpu: Cpu::new(),
            ram: Ram::new(),
            ppu: Ppu::new(),
            timer: Timer::new(),
            clock_enable: true,
            cycles: 0,
            t_cycles: 0,
            m_cycles: 0,
            hardware_handle: Rc::new(RefCell::new(hardware_handle))
        }
    }

    pub fn load_game(&mut self, path: &std::path::Path) -> Result<(), std::io::Error>
    {
        let file_result = std::fs::File::open(path);
        match file_result
        {
            Ok(f) =>
            {
                self.ram.load_rom(&Rom::new(f, Rc::clone(&self.hardware_handle)));
                Ok(())
            },
            Err(x) => Err(x)
        }
    }

    pub fn execute_frame(&mut self)
    {
        for _ in 0..ppu::CYCLES_PER_FRAME
        {
            if self.clock_enable
            {
                if self.cycles % 2 == 0 //T-cycle-pos (4,194,304 hz)
                {
                    self.t_cycles += 1;
                }
                else //T-cycle-neg (4,194,304 hz)
                {

                }

                if self.cycles % 8 == 0 //M-cycle-pos (1,048,576 hz)
                {
                    self.cpu.execute(&mut self.ram);
                    if !self.cpu.halted
                    {
                        self.ram.execute();
                    }
                    self.ppu.execute(&mut self.ram, Rc::clone(&self.hardware_handle));

                    self.m_cycles += 1;
                }
                else //M-cycle-neg (1,048,576 hz)
                {
                    if !self.cpu.halted
                    {
                        self.timer.execute(&mut self.ram, self.m_cycles);
                    }
                }

                if self.cpu.stopped
                {
                    self.clock_enable = false;
                }
            }
        }
        self.hardware_handle.borrow_mut().event_poll();
    }
}