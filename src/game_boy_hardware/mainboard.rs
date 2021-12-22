use std::{time::Duration, thread::{self, Thread}, ptr::{null, null_mut}};

use super::{cpu::Cpu, ram::Ram};

const CLOCK_EDGE:f64 = 8_338_608_f64;

pub struct Mainboard
{
    cpu: Cpu,
    ram: Ram,
    clock: Duration,
    clock_enable: bool,
    ticks: u64
}

impl Mainboard
{
    pub fn new() -> Mainboard
    {
        Mainboard
        {
            cpu: Cpu::new(),
            ram: Ram::new(),
            clock: Duration::from_secs_f64(1_f64 / CLOCK_EDGE),
            clock_enable: false,
            ticks: 0
        }
    }

    pub fn begin_execution(&mut self)
    {
        while self.clock_enable
        {
            if self.ticks % 2 == 0 //T-cycle (4,194,304 hz)
            {

            }

            if self.ticks % 8 == 0 //M-cycle (1,048,576 hz)
            {
                self.cpu.execute(&mut self.ram);
            }
            thread::sleep(self.clock);
        }
    }
}