use sdl2::timer::TimerCallback;

use super::{mainboard::*, ram::{self, Ram}};

//TIMER USES A T-CYCLE-NEG

//Timer clock select values
const INPUT_CLOCK_SELECT_CYCLE_COUNT:[u64; 4] = [1024, 16, 64, 256];

pub struct Timer
{
    internal_counter: u16,
    tima_start: u64,
    tima_enabled: bool,
    tima_overflow: bool
}

impl Timer
{
    pub fn new() -> Timer
    {
        Timer { internal_counter: 0, tima_start: 0, tima_enabled: false, tima_overflow: false }
    }

    pub fn execute(&mut self, ram: &mut Ram, m_cycles: u64)
    {
        //Writing to the divider resets the internal counter
        let divider = ram.read_from_address(ram::DIV);
        if divider != self.internal_counter.to_le_bytes()[1]
        {
            self.internal_counter = 0;
        }

        //Increment internal counter
        self.internal_counter = self.internal_counter.wrapping_add(1);
        let bytes = self.internal_counter.to_le_bytes();
        ram.write_to_address(ram::DIV, bytes[1]);

        let tac_val = ram.read_from_address(ram::TAC);
        let old_timer_enable = self.tima_enabled;
        self.tima_enabled = tac_val & (1 << 2) != 0;
        let timer_clock = INPUT_CLOCK_SELECT_CYCLE_COUNT[(tac_val & 0b00000011_u8) as usize];
        
        //TIMA overflow timing may be incorrect
        if self.tima_overflow
        {
            ram.set_interrupt(ram::INTERRUPT_TIMA);
            self.tima_overflow = false;

            ram.write_to_address(ram::TIMA, ram.read_from_address(ram::TMA));
        }

        if self.tima_enabled
        {
            if !old_timer_enable
            {
                self.tima_start = m_cycles;
            }
            else if m_cycles - self.tima_start >= timer_clock
            {
                self.tima_overflow = self.timer_increment(ram); //After X cycles, increment
            }
        }

        
    }

    fn timer_increment(&mut self, ram: &mut Ram) -> bool
    {
        let tima_val = ram.read_from_address(ram::TIMA);
        let tima_inc = tima_val.overflowing_add(1);
        ram.write_to_address(ram::TIMA, tima_inc.0);
        tima_inc.1
    }
}