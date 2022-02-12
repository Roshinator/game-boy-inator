use std::{time::Duration, thread, fs::*, io::Read};

use super::{cpu::Cpu, ram::Ram, rom::Rom, timer::Timer, ppu::Ppu};

pub const CLOCK_EDGE:f64 = 8_338_608_f64;

pub struct Mainboard
{
    cpu: Cpu,
    ram: Ram,
    ppu: Ppu,
    timer: Timer,
    clock: Duration,
    clock_enable: bool,
    cycles: u64,
    t_cycles: u64,
    m_cycles: u64
}

impl Mainboard
{
    pub fn new(rom: File) -> Mainboard
    {
        Mainboard
        {
            cpu: Cpu::new(),
            ram: Ram::new(Rom::new(rom)),
            ppu: Ppu::new(),
            timer: Timer::new(),
            clock: Duration::from_secs_f64(1_f64 / CLOCK_EDGE),
            clock_enable: true,
            cycles: 0,
            t_cycles: 0,
            m_cycles: 0
        }
    }

    pub fn begin_execution(&mut self)
    {
        loop
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
                    self.ppu.execute(&mut self.ram);

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
    }
}