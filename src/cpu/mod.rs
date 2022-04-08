#[cfg(test)]
mod tests;
use crate::ram::{self, Ram};

//in an AF situation, A is msh, F is lsh, little endian

bitflags::bitflags!
{
    pub struct CpuFlags: u8
    {
        const FLAG_Z = 1 << 7;
        const FLAG_N = 1 << 6;
        const FLAG_H = 1 << 5;
        const FLAG_C = 1 << 4;
    }
}

const ZERO_INSTRUCTION_TIME_TABLE:[u8;0x100] = //M-cycle timings
    [1,3,2,2,1,1,2,1,5,2,2,2,1,1,2,1,
     1,3,2,2,1,1,2,1,3,2,2,2,1,1,2,1,
     2,3,2,2,1,1,2,1,2,2,2,2,1,1,2,1,
     2,3,2,2,3,3,3,1,2,2,2,2,1,1,2,1,
     1,1,1,1,1,1,2,1,1,1,1,1,1,1,2,1,
     1,1,1,1,1,1,2,1,1,1,1,1,1,1,2,1,
     1,1,1,1,1,1,2,1,1,1,1,1,1,1,2,1,
     2,2,2,2,2,2,1,2,1,1,1,1,1,1,2,1,
     1,1,1,1,1,1,2,1,1,1,1,1,1,1,2,1,
     1,1,1,1,1,1,2,1,1,1,1,1,1,1,2,1,
     1,1,1,1,1,1,2,1,1,1,1,1,1,1,2,1,
     1,1,1,1,1,1,2,1,1,1,1,1,1,1,2,1,
     2,3,3,4,3,4,2,4,2,4,3,1,3,6,2,4,
     1,3,3,0,3,4,2,4,2,4,3,0,3,0,2,4,
     3,3,2,0,0,4,2,4,4,1,4,0,0,0,2,4,
     3,3,2,1,0,4,2,4,3,2,4,1,0,0,2,4];

const CB_INSTRUCTION_TIME_TABLE:[u8;0x100] = //M-Cycle timings
    [2,2,2,2,2,2,4,2,2,2,2,2,2,2,4,2,
     2,2,2,2,2,2,4,2,2,2,2,2,2,2,4,2,
     2,2,2,2,2,2,4,2,2,2,2,2,2,2,4,2,
     2,2,2,2,2,2,4,2,2,2,2,2,2,2,4,2,
     2,2,2,2,2,2,3,2,2,2,2,2,2,2,3,2,
     2,2,2,2,2,2,3,2,2,2,2,2,2,2,3,2,
     2,2,2,2,2,2,3,2,2,2,2,2,2,2,3,2,
     2,2,2,2,2,2,3,2,2,2,2,2,2,2,3,2,
     2,2,2,2,2,2,4,2,2,2,2,2,2,2,4,2,
     2,2,2,2,2,2,4,2,2,2,2,2,2,2,4,2,
     2,2,2,2,2,2,4,2,2,2,2,2,2,2,4,2,
     2,2,2,2,2,2,4,2,2,2,2,2,2,2,4,2,
     2,2,2,2,2,2,4,2,2,2,2,2,2,2,4,2,
     2,2,2,2,2,2,4,2,2,2,2,2,2,2,4,2,
     2,2,2,2,2,2,4,2,2,2,2,2,2,2,4,2,
     2,2,2,2,2,2,4,2,2,2,2,2,2,2,4,2];

#[derive(Clone, Copy)]
pub struct ProgramCounter
{
    pub reg: u16,
    pub should_increment: bool,
    pub current_instruction_width: u16,
    pub current_instruction_cycles: u8
}

pub struct Cpu
{
    reg_a: u8,
    reg_b: u8,
    reg_c: u8,
    reg_d: u8,
    reg_e: u8,
    reg_f: CpuFlags,
    reg_h: u8,
    reg_l: u8,
    sp: u16,
    pc: ProgramCounter,
    ime: bool,
    pub halted: bool,
    pub stopped: bool
}

impl Cpu
{
    pub fn new() -> Cpu
    {
        Cpu
        {
            reg_a: 0x01,
            reg_b: 0x00,
            reg_c: 0x13,
            reg_d: 0x00,
            reg_e: 0xD8,
            reg_f: CpuFlags::from_bits(0xB0).unwrap(),
            reg_h: 0x01,
            reg_l: 0x4D,
            sp: 0xFFFE,
            pc: ProgramCounter
            {
                reg: 0x0000, should_increment: true, current_instruction_width: 1, current_instruction_cycles: 0
            },
            ime: false,
            halted: false,
            stopped: false
        }
    }
    //Format [name]_[param1]_[param2]
    //r is a register
    //sp/pc are stack pointer and program counter
    //hl is hl register, a alone is a register, etc.
    //a suffix means parameter is an address (dereference)
    //i(num) is a signed value

    fn halt(&mut self)
    {
        self.halted = true;
    }

    fn stop(&mut self)
    {
        self.halted = true;
        self.stopped = true;
    }

    fn ld_r16_16(msh_reg: &mut u8, lsh_reg: &mut u8, msh_num: u8, lsh_num: u8)
    {
        *msh_reg = msh_num;
        *lsh_reg = lsh_num;
    }

    fn ld_hl_sp_plus(sp: &mut u16, reg_h: &mut u8, reg_l: &mut u8, p1: i8)
    {
        let conv = p1.unsigned_abs() as u16;
        let negative = p1 < 0;
        if negative
        {
            let bytes = u16::to_le_bytes(*sp - conv);
            *reg_h = bytes[1];
            *reg_l = bytes[0];
        }
        else
        {
            let bytes = u16::to_le_bytes(*sp + conv);
            *reg_h = bytes[1];
            *reg_l = bytes[0];
        }
    }

    fn ld_sp_16(sp: &mut u16, msh_num: u8, lsh_num: u8)
    {
        *sp = u16::from_le_bytes([lsh_num, msh_num]);
    }

    fn ld_r8_8(p1: &mut u8, p2: u8)
    {
        *p1 = p2;
    }

    fn ld_16a_sp(sp: &mut u16, ram: &mut Ram, msh: u8, lsh: u8)
    {
        let bytes = sp.to_le_bytes();
        ram.write_rp(msh, lsh, bytes[0]);

        let result = Cpu::aux_inc_16(msh, lsh);
        ram.write_rp(result.1, result.0, bytes[1]);
    }

    fn ld_r8_r8(p1: &mut u8, p2: &mut u8)
    {
        *p1 = *p2;
    }

    fn ld_r8_r8_s(_p1: &mut u8)
    {
        // *p1 = *p1; Basically a NOP
    }

    fn ld_sp_r16(sp: &mut u16, msh: &mut u8, lsh: &mut u8)
    {
        *sp = u16::from_le_bytes([*lsh, *msh]);
    }

    // TODO: See if the flags are modified

    ///Returns (lsh, msh)
    fn aux_inc_16(msh: u8, lsh: u8) -> (u8, u8)
    {
        let lsh_result = u8::overflowing_add(lsh, 1);
        let msh_result = u8::overflowing_add(msh, lsh_result.1 as u8);
        (lsh_result.0, msh_result.0)
    }

    fn inc_r16(msh: &mut u8, lsh: &mut u8)
    {
        let lsh_result = lsh.overflowing_add(1);
        *lsh = lsh_result.0;
        let msh_result = msh.overflowing_add(lsh_result.1 as u8);
        *msh = msh_result.0;
    }

    fn inc_sp(sp: &mut u16)
    {
        let result = u16::overflowing_add(*sp, 1);
        *sp = result.0;
    }

    fn inc_r8(reg: &mut u8, flags: &mut CpuFlags)
    {
        let result = reg.overflowing_add(1);
        *reg = result.0;

        flags.set(CpuFlags::FLAG_Z, result.0 == 0);
        flags.set(CpuFlags::FLAG_N, false);
        flags.set(CpuFlags::FLAG_H, result.1);
    }

    fn dec_r8(reg: &mut u8, flags: &mut CpuFlags)
    {
        let result = reg.overflowing_sub(1);
        *reg = result.0;

        flags.set(CpuFlags::FLAG_Z, result.0 == 0);
        flags.set(CpuFlags::FLAG_N, true);
        flags.set(CpuFlags::FLAG_H, result.1);
    }

    fn dec_r16(msh: &mut u8, lsh: &mut u8)
    {
        let lsh_result = lsh.overflowing_sub(1);
        *lsh = lsh_result.0;
        let msh_result = msh.overflowing_sub(lsh_result.1 as u8);
        *msh = msh_result.0;
    }

    fn dec_sp(sp: &mut u16)
    {
        let result = u16::overflowing_sub(*sp, 1);
        *sp = result.0;
    }

    fn add_r8_r8(p1: &mut u8, p2: &mut u8, flags: &mut CpuFlags)
    {
        let half_carry_pre = ((*p1 ^ *p2) >> 4) & 1;
        let result = p1.overflowing_add(*p2);
        *p1 = result.0;
        let half_carry_post = (result.0 >> 4) & 1;

        flags.set(CpuFlags::FLAG_Z, result.0 == 0);
        flags.set(CpuFlags::FLAG_H, half_carry_pre != half_carry_post);
        flags.set(CpuFlags::FLAG_N, false);
        flags.set(CpuFlags::FLAG_C, result.1);
    }

    fn add_r8_r8_s(p1: &mut u8, flags: &mut CpuFlags)
    {
        let half_carry_pre = 0; //((*p1 ^ *p1) >> 4) & 1;
        let result = p1.overflowing_add(*p1);
        *p1 = result.0;
        let half_carry_post = (result.0 >> 4) & 1;

        flags.set(CpuFlags::FLAG_Z, result.0 == 0);
        flags.set(CpuFlags::FLAG_H, half_carry_pre != half_carry_post);
        flags.set(CpuFlags::FLAG_N, false);
        flags.set(CpuFlags::FLAG_C, result.1);
    }

    fn add_r8_8(p1: &mut u8, p2: u8, flags: &mut CpuFlags)
    {
        let half_carry_pre = ((*p1 ^ p2) >> 4) & 1;
        let result = p1.overflowing_add(p2);
        *p1 = result.0;
        let half_carry_post = (result.0 >> 4) & 1;

        flags.set(CpuFlags::FLAG_Z, result.0 == 0);
        flags.set(CpuFlags::FLAG_H, half_carry_pre != half_carry_post);
        flags.set(CpuFlags::FLAG_N, false);
        flags.set(CpuFlags::FLAG_C, result.1);
    }

    fn add_r16_r16(p1_msh: &mut u8, p1_lsh: &mut u8, p2_msh: &mut u8, p2_lsh: &mut u8, flags: &mut CpuFlags)
    {
        let z = flags.contains(CpuFlags::FLAG_Z);
        Cpu::add_r8_r8(p1_lsh, p2_lsh, flags);
        Cpu::adc_r8_r8(p1_msh, p2_msh, flags);

        flags.set(CpuFlags::FLAG_Z, z);
        flags.set(CpuFlags::FLAG_N, false);
    }

    fn add_r16_r16_s(p1_msh: &mut u8, p1_lsh: &mut u8, flags: &mut CpuFlags)
    {
        let z = flags.contains(CpuFlags::FLAG_Z);
        Cpu::add_r8_r8_s(p1_lsh, flags);
        Cpu::adc_r8_r8_s(p1_msh, flags);

        flags.set(CpuFlags::FLAG_Z, z);
        flags.set(CpuFlags::FLAG_N, false);
    }

    fn add_r16_sp(p1_msh: &mut u8, p1_lsh: &mut u8, sp: &mut u16, flags: &mut CpuFlags)
    {
        let z = flags.contains(CpuFlags::FLAG_Z);
        let reg = sp.to_le_bytes();

        //ADD
        let result = p1_lsh.overflowing_add(reg[0]);
        *p1_lsh = result.0;
        flags.set(CpuFlags::FLAG_C, result.1);

        //ADC
        let carry = flags.contains(CpuFlags::FLAG_C) as u8;
        let half_carry_pre1 = ((*p1_msh ^ reg[1]) >> 4) & 1;
        let result1 = p1_msh.overflowing_add(reg[1]);
        let half_carry_post1 = (result1.0 >> 4) & 1;
        let half_carry_pre2 = ((result1.0 ^ carry) >> 4) & 1;
        let result2 = result1.0.overflowing_add(carry);
        *p1_msh = result2.0;
        let half_carry_post2 = (result2.0 >> 4) & 1;

        flags.set(CpuFlags::FLAG_H, half_carry_pre1 != half_carry_post1 || half_carry_pre2 != half_carry_post2);
        flags.set(CpuFlags::FLAG_N, false);
        flags.set(CpuFlags::FLAG_C, result1.1 || result2.1);
        flags.set(CpuFlags::FLAG_Z, z);
    }

    fn add_sp_i8(sp: &mut u16, p1: i8)
    {
        let conv = p1.unsigned_abs() as u16;
        let negative = p1 < 0;
        if negative
        {
            *sp -= conv;
        }
        else
        {
            *sp += conv;
        }
    }

    fn adc_r8_r8(p1: &mut u8, p2: &mut u8, flags: &mut CpuFlags)
    {
        let carry = flags.contains(CpuFlags::FLAG_C) as u8;
        let half_carry_pre1 = ((*p1 ^ *p2) >> 4) & 1;
        let result1 = p1.overflowing_add(*p2);
        let half_carry_post1 = (result1.0 >> 4) & 1;
        let half_carry_pre2 = ((result1.0 ^ carry) >> 4) & 1;
        let result2 = result1.0.overflowing_add(carry);
        *p1 = result2.0;
        let half_carry_post2 = (result2.0 >> 4) & 1;

        flags.set(CpuFlags::FLAG_Z, result2.0 == 0);
        flags.set(CpuFlags::FLAG_H, half_carry_pre1 != half_carry_post1 || half_carry_pre2 != half_carry_post2);
        flags.set(CpuFlags::FLAG_N, false);
        flags.set(CpuFlags::FLAG_C, result1.1 || result2.1);
    }

    fn adc_r8_r8_s(p1: &mut u8, flags: &mut CpuFlags)
    {
        let carry = flags.contains(CpuFlags::FLAG_C) as u8;
        let half_carry_pre1 = 0; //((*p1 ^ *p1) >> 4) & 1;
        let result1 = p1.overflowing_add(*p1);
        let half_carry_post1 = (result1.0 >> 4) & 1;
        let half_carry_pre2 = ((result1.0 ^ carry) >> 4) & 1;
        let result2 = result1.0.overflowing_add(carry);
        *p1 = result2.0;
        let half_carry_post2 = (result2.0 >> 4) & 1;

        flags.set(CpuFlags::FLAG_Z, result2.0 == 0);
        flags.set(CpuFlags::FLAG_H, half_carry_pre1 != half_carry_post1 || half_carry_pre2 != half_carry_post2);
        flags.set(CpuFlags::FLAG_N, false);
        flags.set(CpuFlags::FLAG_C, result1.1 || result2.1);
    }

    fn adc_r8_8(p1: &mut u8, p2: u8, flags: &mut CpuFlags)
    {
        let carry = flags.contains(CpuFlags::FLAG_C) as u8;
        let half_carry_pre1 = ((*p1 ^ p2) >> 4) & 1;
        let result1 = p1.overflowing_add(p2);
        let half_carry_post1 = (result1.0 >> 4) & 1;
        let half_carry_pre2 = ((result1.0 ^ carry) >> 4) & 1;
        let result2 = result1.0.overflowing_add(carry);
        *p1 = result2.0;
        let half_carry_post2 = (result2.0 >> 4) & 1;

        flags.set(CpuFlags::FLAG_Z, result2.0 == 0);
        flags.set(CpuFlags::FLAG_H, half_carry_pre1 != half_carry_post1 || half_carry_pre2 != half_carry_post2);
        flags.set(CpuFlags::FLAG_N, false);
        flags.set(CpuFlags::FLAG_C, result1.1 || result2.1);
    }

    //TODO: Check subtraction half carry calculations
    fn sub_r8_r8(p1: &mut u8, p2: &mut u8, flags: &mut CpuFlags)
    {
        let half_carry_pre = ((*p1 ^ *p2) >> 4) & 1;
        let result = p1.overflowing_sub(*p2);
        *p1 = result.0;
        let half_carry_post = (result.0 >> 4) & 1;

        flags.set(CpuFlags::FLAG_Z, result.0 == 0);
        flags.set(CpuFlags::FLAG_H, half_carry_pre != half_carry_post);
        flags.set(CpuFlags::FLAG_N, true);
        flags.set(CpuFlags::FLAG_C, result.1);
    }

    fn sub_r8_r8_s(p1: &mut u8, flags: &mut CpuFlags)
    {
        let half_carry_pre = 0; //((*p1 ^ *p1) >> 4) & 1;
        let result = p1.overflowing_sub(*p1);
        *p1 = result.0;
        let half_carry_post = (result.0 >> 4) & 1;

        flags.set(CpuFlags::FLAG_Z, result.0 == 0);
        flags.set(CpuFlags::FLAG_H, half_carry_pre != half_carry_post);
        flags.set(CpuFlags::FLAG_N, true);
        flags.set(CpuFlags::FLAG_C, result.1);
    }

    fn sub_r8_8(p1: &mut u8, p2: u8, flags: &mut CpuFlags)
    {
        let half_carry_pre = ((*p1 ^ p2) >> 4) & 1;
        let result = p1.overflowing_sub(p2);
        *p1 = result.0;
        let half_carry_post = (result.0 >> 4) & 1;

        flags.set(CpuFlags::FLAG_Z, result.0 == 0);
        flags.set(CpuFlags::FLAG_H, half_carry_pre != half_carry_post);
        flags.set(CpuFlags::FLAG_N, true);
        flags.set(CpuFlags::FLAG_C, result.1);
    }

    fn sbc_r8_r8(p1: &mut u8, p2: &mut u8, flags: &mut CpuFlags)
    {
        let carry = flags.contains(CpuFlags::FLAG_C) as u8;
        let half_carry_pre1 = ((*p1 ^ *p2) >> 4) & 1;
        let result1 = p1.overflowing_sub(*p2);
        let half_carry_post1 = (result1.0 >> 4) & 1;
        let half_carry_pre2 = ((result1.0 ^ carry) >> 4) & 1;
        let result2 = result1.0.overflowing_sub(carry);
        *p1 = result2.0;
        let half_carry_post2 = (result2.0 >> 4) & 1;

        flags.set(CpuFlags::FLAG_Z, result2.0 == 0);
        flags.set(CpuFlags::FLAG_H, half_carry_pre1 != half_carry_post1 || half_carry_pre2 != half_carry_post2);
        flags.set(CpuFlags::FLAG_N, true);
        flags.set(CpuFlags::FLAG_C, result1.1 || result2.1);
    }

    fn sbc_r8_r8_s(p1: &mut u8, flags: &mut CpuFlags)
    {
        let carry = flags.contains(CpuFlags::FLAG_C) as u8;
        let half_carry_pre1 = 0; //((*p1 ^ *p1) >> 4) & 1;
        let result1 = p1.overflowing_sub(*p1);
        let half_carry_post1 = (result1.0 >> 4) & 1;
        let half_carry_pre2 = ((result1.0 ^ carry) >> 4) & 1;
        let result2 = result1.0.overflowing_sub(carry);
        *p1 = result2.0;
        let half_carry_post2 = (result2.0 >> 4) & 1;

        flags.set(CpuFlags::FLAG_Z, result2.0 == 0);
        flags.set(CpuFlags::FLAG_H, half_carry_pre1 != half_carry_post1 || half_carry_pre2 != half_carry_post2);
        flags.set(CpuFlags::FLAG_N, true);
        flags.set(CpuFlags::FLAG_C, result1.1 || result2.1);
    }

    fn sbc_r8_8(p1: &mut u8, p2: u8, flags: &mut CpuFlags)
    {
        let carry = flags.contains(CpuFlags::FLAG_C) as u8;
        let half_carry_pre1 = ((*p1 ^ p2) >> 4) & 1;
        let result1 = p1.overflowing_sub(p2);
        let half_carry_post1 = (result1.0 >> 4) & 1;
        let half_carry_pre2 = ((result1.0 ^ carry) >> 4) & 1;
        let result2 = result1.0.overflowing_sub(carry);
        *p1 = result2.0;
        let half_carry_post2 = (result2.0 >> 4) & 1;

        flags.set(CpuFlags::FLAG_Z, result2.0 == 0);
        flags.set(CpuFlags::FLAG_H, half_carry_pre1 != half_carry_post1 || half_carry_pre2 != half_carry_post2);
        flags.set(CpuFlags::FLAG_N, true);
        flags.set(CpuFlags::FLAG_C, result1.1 || result2.1);
    }

    fn and_r8_r8(p1: &mut u8, p2: &mut u8, flags: &mut CpuFlags)
    {
        *p1 &= *p2;

        flags.set(CpuFlags::FLAG_Z, *p1 == 0);
        flags.set(CpuFlags::FLAG_H, true);
        flags.set(CpuFlags::FLAG_N, false);
        flags.set(CpuFlags::FLAG_C, false);
    }

    fn and_r8_r8_s(p1: &mut u8, flags: &mut CpuFlags)
    {
        *p1 &= *p1;

        flags.set(CpuFlags::FLAG_Z, *p1 == 0);
        flags.set(CpuFlags::FLAG_H, true);
        flags.set(CpuFlags::FLAG_N, false);
        flags.set(CpuFlags::FLAG_C, false);
    }

    fn and_r8_8(p1: &mut u8, p2: u8, flags: &mut CpuFlags)
    {
        *p1 &= p2;

        flags.set(CpuFlags::FLAG_Z, *p1 == 0);
        flags.set(CpuFlags::FLAG_H, true);
        flags.set(CpuFlags::FLAG_N, false);
        flags.set(CpuFlags::FLAG_C, false);
    }

    fn xor_r8_r8(p1: &mut u8, p2: &mut u8, flags: &mut CpuFlags)
    {
        *p1 ^= *p2;

        flags.set(CpuFlags::FLAG_Z, *p1 == 0);
        flags.set(CpuFlags::FLAG_H, false);
        flags.set(CpuFlags::FLAG_N, false);
        flags.set(CpuFlags::FLAG_C, false);
    }

    fn xor_r8_r8_s(p1: &mut u8, flags: &mut CpuFlags)
    {
        *p1 ^= *p1;

        flags.set(CpuFlags::FLAG_Z, *p1 == 0);
        flags.set(CpuFlags::FLAG_H, false);
        flags.set(CpuFlags::FLAG_N, false);
        flags.set(CpuFlags::FLAG_C, false);
    }

    fn xor_r8_8(p1: &mut u8, p2: u8, flags: &mut CpuFlags)
    {
        *p1 ^= p2;

        flags.set(CpuFlags::FLAG_Z, *p1 == 0);
        flags.set(CpuFlags::FLAG_H, false);
        flags.set(CpuFlags::FLAG_N, false);
        flags.set(CpuFlags::FLAG_C, false);
    }

    fn or_r8_r8(p1: &mut u8, p2: &mut u8, flags: &mut CpuFlags)
    {
        *p1 |= *p2;

        flags.set(CpuFlags::FLAG_Z, *p1 == 0);
        flags.set(CpuFlags::FLAG_H, false);
        flags.set(CpuFlags::FLAG_N, false);
        flags.set(CpuFlags::FLAG_C, false);
    }

    fn or_r8_r8_s(p1: &mut u8, flags: &mut CpuFlags)
    {
        *p1 |= *p1;

        flags.set(CpuFlags::FLAG_Z, *p1 == 0);
        flags.set(CpuFlags::FLAG_H, false);
        flags.set(CpuFlags::FLAG_N, false);
        flags.set(CpuFlags::FLAG_C, false);
    }

    fn or_r8_8(p1: &mut u8, p2: u8, flags: &mut CpuFlags)
    {
        *p1 |= p2;

        flags.set(CpuFlags::FLAG_Z, *p1 == 0);
        flags.set(CpuFlags::FLAG_H, false);
        flags.set(CpuFlags::FLAG_N, false);
        flags.set(CpuFlags::FLAG_C, false);
    }

    fn cp_r8_r8(p1: &mut u8, p2: &mut u8, flags: &mut CpuFlags)
    {
        let half_carry_pre = ((*p1 ^ *p2) >> 4) & 1;
        let result = p1.overflowing_sub(*p2);
        *p1 = result.0;
        let half_carry_post = (result.0 >> 4) & 1;

        flags.set(CpuFlags::FLAG_Z, result.0 == 0);
        flags.set(CpuFlags::FLAG_H, half_carry_pre != half_carry_post);
        flags.set(CpuFlags::FLAG_N, true);
        flags.set(CpuFlags::FLAG_C, result.1);
    }

    fn cp_r8_r8_s(p1: &mut u8, flags: &mut CpuFlags)
    {
        let half_carry_pre = 0; //((*p1 ^ *p1) >> 4) & 1;
        let result = p1.overflowing_sub(*p1);
        *p1 = result.0;
        let half_carry_post = (result.0 >> 4) & 1;

        flags.set(CpuFlags::FLAG_Z, result.0 == 0);
        flags.set(CpuFlags::FLAG_H, half_carry_pre != half_carry_post);
        flags.set(CpuFlags::FLAG_N, true);
        flags.set(CpuFlags::FLAG_C, result.1);
    }

    fn cp_r8_8(p1: &mut u8, p2: u8, flags: &mut CpuFlags)
    {
        let half_carry_pre = ((*p1 ^ p2) >> 4) & 1;
        let result = p1.overflowing_sub(p2);
        *p1 = result.0;
        let half_carry_post = (result.0 >> 4) & 1;

        flags.set(CpuFlags::FLAG_Z, result.0 == 0);
        flags.set(CpuFlags::FLAG_H, half_carry_pre != half_carry_post);
        flags.set(CpuFlags::FLAG_N, true);
        flags.set(CpuFlags::FLAG_C, result.1);
    }

    fn daa(reg_a: &mut u8, flags: &mut CpuFlags)
    {
        let c_before = flags.contains(CpuFlags::FLAG_C);
        let bits_47 = (*reg_a >> 4) & 0b00001111;
        let h_before = flags.contains(CpuFlags::FLAG_H);
        let bits_03 = *reg_a & 0b00001111;
        if flags.contains(CpuFlags::FLAG_N) //Add preceded instruction
        {
            match (c_before, bits_47, h_before, bits_03)
            {
                (false, 0x0..=0x9, false, 0x0..=0x9) => {
                    *reg_a = reg_a.wrapping_add(0x00);
                    flags.set(CpuFlags::FLAG_C, false);
                },
                (false, 0x0..=0x8, false, 0xA..=0xF) => {
                    *reg_a = reg_a.wrapping_add(0x06);
                    flags.set(CpuFlags::FLAG_C, false);
                },
                (false, 0x0..=0x9, true, 0x0..=0x3) => {
                    *reg_a = reg_a.wrapping_add(0x06);
                    flags.set(CpuFlags::FLAG_C, false);
                },
                (false, 0xA..=0xF, false, 0x0..=0x9) => {
                    *reg_a = reg_a.wrapping_add(0x60);
                    flags.set(CpuFlags::FLAG_C, true);
                },
                (false, 0x9..=0xF, false, 0xA..=0xF) => {
                    *reg_a = reg_a.wrapping_add(0x66);
                    flags.set(CpuFlags::FLAG_C, true);
                },
                (false, 0xA..=0xF, true, 0x0..=0x3) => {
                    *reg_a = reg_a.wrapping_add(0x66);
                    flags.set(CpuFlags::FLAG_C, true);
                },
                (true, 0x0..=0x2, false, 0x0..=0x9) => {
                    *reg_a = reg_a.wrapping_add(0x60);
                    flags.set(CpuFlags::FLAG_C, true);
                },
                (true, 0x0..=0x2, false, 0xA..=0xF) => {
                    *reg_a = reg_a.wrapping_add(0x66);
                    flags.set(CpuFlags::FLAG_C, true);
                },
                (true, 0x0..=0x3, true, 0x0..=0x3) => {
                    *reg_a = reg_a.wrapping_add(0x66);
                    flags.set(CpuFlags::FLAG_C, true);
                },
                _ => panic!("Invalid BDC conversion")
            }
        }
        else //subtract preceded instruction
        {
            match (c_before, bits_47, h_before, bits_03)
            {
                (false, 0x0..=0x9, false, 0x0..=0x9) => {
                    *reg_a = reg_a.wrapping_add(0x00);
                    flags.set(CpuFlags::FLAG_C, false);
                },
                (false, 0x0..=0x8, true, 0x6..=0xF) => {
                    *reg_a = reg_a.wrapping_add(0xFA);
                    flags.set(CpuFlags::FLAG_C, false);
                },
                (true, 0x7..=0xF, false, 0x0..=0x9) => {
                    *reg_a = reg_a.wrapping_add(0xA0);
                    flags.set(CpuFlags::FLAG_C, true);
                },
                (true, 0x6..=0xF, true, 0x6..=0xF) => {
                    *reg_a = reg_a.wrapping_add(0x9A);
                    flags.set(CpuFlags::FLAG_C, true);
                },
                _ => panic!("Invalid BDC conversion")
            }
        }
    }

    fn cpl(reg_a: &mut u8, flags: &mut CpuFlags)
    {
        *reg_a = !*reg_a;
        flags.set(CpuFlags::FLAG_N, true);
        flags.set(CpuFlags::FLAG_H, true);
    }

    fn ccf(flags: &mut CpuFlags)
    {
        flags.set(CpuFlags::FLAG_C, !flags.contains(CpuFlags::FLAG_C));
        flags.set(CpuFlags::FLAG_N, false);
        flags.set(CpuFlags::FLAG_H, false);
    }

    fn scf(flags: &mut CpuFlags)
    {
        flags.set(CpuFlags::FLAG_C, true);
        flags.set(CpuFlags::FLAG_N, false);
        flags.set(CpuFlags::FLAG_H, false);
    }

    fn jp_pc_16(pc: &mut ProgramCounter, msh: u8, lsh: u8)
    {
        pc.reg = u16::from_le_bytes([lsh, msh]);
        pc.should_increment = false;
    }

    fn jp_flag_pc_16(pc: &mut ProgramCounter, flag: CpuFlags, msh: u8, lsh: u8, flags: &mut CpuFlags)
    {
        if flags.contains(flag)
        {
            Cpu::jp_pc_16(pc, msh, lsh);
        }
    }

    fn jp_nflag_pc_16(pc: &mut ProgramCounter, flag: CpuFlags, msh: u8, lsh: u8, flags: &mut CpuFlags)
    {
        if !flags.contains(flag)
        {
            Cpu::jp_pc_16(pc, msh, lsh);
        }
    }

    fn jr_i8(pc: &mut ProgramCounter, p1: i8)
    {
        let conv = p1.unsigned_abs() as u16;
        let negative = p1 < 0;
        if negative
        {
            pc.reg -= conv;
        }
        else
        {
            pc.reg += conv;
        }
        pc.should_increment = false;
    }

    fn jr_flag_i8(pc: &mut ProgramCounter, flag: CpuFlags, p1: i8, flags: &mut CpuFlags)
    {
        if flags.contains(flag)
        {
            Cpu::jr_i8(pc, p1);
        }
    }

    fn jr_nflag_i8(pc: &mut ProgramCounter, flag: CpuFlags, p1: i8, flags: &mut CpuFlags)
    {
        if !flags.contains(flag)
        {
            Cpu::jr_i8(pc, p1);
        }
    }

    fn call_16(ram: &mut Ram, msh: u8, lsh: u8, pc: &mut ProgramCounter, sp: &mut u16)
    {
        let pc_bytes = pc.reg.to_le_bytes();
        ram.write(*sp - 1, pc_bytes[1]);
        ram.write(*sp - 2, pc_bytes[0]);
        pc.reg = u16::from_le_bytes([lsh, msh]);
        *sp -= 2;
    }

    fn call_flag_16(ram: &mut Ram, flag: CpuFlags, msh: u8, lsh: u8, pc: &mut ProgramCounter, sp: &mut u16, flags: &mut CpuFlags)
    {
        if !flags.contains(flag)
        {
            Cpu::call_16(ram, msh, lsh, pc, sp);
        }
    }

    fn call_nflag_16(ram: &mut Ram, flag: CpuFlags, msh: u8, lsh: u8, pc: &mut ProgramCounter, sp: &mut u16, flags: &mut CpuFlags)
    {
        if !flags.contains(flag)
        {
            Cpu::call_16(ram, msh, lsh, pc, sp);
        }
    }

    fn ret(ram: &mut Ram, pc: &mut ProgramCounter, sp: &mut u16)
    {
        let sp_lsh = ram.read(*sp);
        let sp_msh = ram.read(*sp + 1);
        pc.reg = u16::from_le_bytes([sp_lsh, sp_msh]);
        *sp += 2;
    }

    fn ret_flag(ram: &mut Ram, pc: &mut ProgramCounter, sp: &mut u16, flag: CpuFlags, flags: &mut CpuFlags)
    {
        if !flags.contains(flag)
        {
            Cpu::ret(ram, pc, sp);
        }
    }

    fn ret_nflag(ram: &mut Ram, pc: &mut ProgramCounter, sp: &mut u16, flag: CpuFlags, flags: &mut CpuFlags)
    {
        if !flags.contains(flag)
        {
            Cpu::ret(ram, pc, sp);
        }
    }

    fn rst(ram: &mut Ram, loc: u8, pc: &mut ProgramCounter, sp: &mut u16)
    {
        let pc_bytes = pc.reg.to_le_bytes();
        ram.write(*sp - 1, pc_bytes[1]);
        ram.write(*sp - 2, pc_bytes[0]);
        *sp -= 2;
        pc.reg = u16::from_le_bytes([loc, 0]);
    }


    //--------------------16 BIT OPCODES--------------------

    fn rlc_r8(p1: &mut u8, flags: &mut CpuFlags)
    {
        flags.set(CpuFlags::FLAG_C, (*p1 >> 7) & 1 != 0);
        *p1 = p1.rotate_left(1);

        flags.set(CpuFlags::FLAG_Z, *p1 == 0);
        flags.set(CpuFlags::FLAG_N, false);
        flags.set(CpuFlags::FLAG_H, false);
    }

    //Note the different flag behavior preventing it from being merged into the r8 ver
    //RLC &self.reg_a and RLCA are both possible
    fn rlca(reg_a: &mut u8, flags: &mut CpuFlags)
    {
        flags.set(CpuFlags::FLAG_C, (*reg_a >> 7) & 1 != 0);
        *reg_a = reg_a.rotate_left(1);

        flags.set(CpuFlags::FLAG_Z, false);
        flags.set(CpuFlags::FLAG_N, false);
        flags.set(CpuFlags::FLAG_H, false);
    }

    fn rrc_r8(p1: &mut u8, flags: &mut CpuFlags)
    {
        flags.set(CpuFlags::FLAG_C, *p1 & 1 != 0);
        *p1 = p1.rotate_right(1);

        flags.set(CpuFlags::FLAG_Z, *p1 == 0);
        flags.set(CpuFlags::FLAG_N, false);
        flags.set(CpuFlags::FLAG_H, false);
    }

    //Note the different flag behavior preventing it from being merged into the r8 ver
    //RRC &self.reg_a and RRCA are both possible
    fn rrca(reg_a: &mut u8, flags: &mut CpuFlags)
    {
        flags.set(CpuFlags::FLAG_C, *reg_a & 1 != 0);
        *reg_a = reg_a.rotate_right(1);

        flags.set(CpuFlags::FLAG_Z, false);
        flags.set(CpuFlags::FLAG_N, false);
        flags.set(CpuFlags::FLAG_H, false);
    }

    fn rl_r8(p1: &mut u8, flags: &mut CpuFlags)
    {
        let cin = flags.contains(CpuFlags::FLAG_C) as u8;
        flags.set(CpuFlags::FLAG_C, (*p1 >> 7) & 1 != 0);
        *p1 = (*p1 << 1u8) | cin;

        flags.set(CpuFlags::FLAG_Z, *p1 == 0);
        flags.set(CpuFlags::FLAG_N, false);
        flags.set(CpuFlags::FLAG_H, false);
    }

    //Note the different flag behavior preventing it from being merged into the r8 ver
    //RL &self.reg_a and RLA are both possible
    fn rla(reg_a: &mut u8, flags: &mut CpuFlags)
    {
        let cin = flags.contains(CpuFlags::FLAG_C) as u8;
        flags.set(CpuFlags::FLAG_C, (*reg_a >> 7) & 1 != 0);
        *reg_a = (*reg_a << 1u8) | cin;

        flags.set(CpuFlags::FLAG_Z, false);
        flags.set(CpuFlags::FLAG_N, false);
        flags.set(CpuFlags::FLAG_H, false);
    }

    fn rr_r8(p1: &mut u8, flags: &mut CpuFlags)
    {
        let cin = flags.contains(CpuFlags::FLAG_C) as u8;
        flags.set(CpuFlags::FLAG_C, *p1 & 1 != 0);
        *p1 = (*p1 >> 1u8) | (cin << 7u8);

        flags.set(CpuFlags::FLAG_Z, *p1 == 0);
        flags.set(CpuFlags::FLAG_N, false);
        flags.set(CpuFlags::FLAG_H, false);
    }

    //Note the different flag behavior preventing it from being merged into the r8 ver
    //RR &self.reg_a and RRA are both possible
    fn rra(reg_a: &mut u8, flags: &mut CpuFlags)
    {
        let cin = flags.contains(CpuFlags::FLAG_C) as u8;
        flags.set(CpuFlags::FLAG_C, *reg_a & 1 != 0);
        *reg_a = (*reg_a >> 1u8) | (cin << 7u8);

        flags.set(CpuFlags::FLAG_Z, false);
        flags.set(CpuFlags::FLAG_N, false);
        flags.set(CpuFlags::FLAG_H, false);
    }

    fn sla_r8(p1: &mut u8, flags: &mut CpuFlags)
    {
        flags.set(CpuFlags::FLAG_C, (*p1 >> 7) & 1 != 0);
        *p1 <<= 1u8;

        flags.set(CpuFlags::FLAG_Z, *p1 == 0);
        flags.set(CpuFlags::FLAG_N, false);
        flags.set(CpuFlags::FLAG_H, false);
    }

    fn sra_r8(p1: &mut u8, flags: &mut CpuFlags)
    {
        flags.set(CpuFlags::FLAG_C, *p1 & 1 != 0);
        *p1 = (*p1 >> 1u8) | (*p1 & 0b10000000u8); //fill with leftmost

        flags.set(CpuFlags::FLAG_Z, *p1 == 0);
        flags.set(CpuFlags::FLAG_N, false);
        flags.set(CpuFlags::FLAG_H, false);
    }

    fn srl_r8(p1: &mut u8, flags: &mut CpuFlags)
    {
        flags.set(CpuFlags::FLAG_C, *p1 & 1 != 0);
        *p1 >>= 1u8; //fill with leftmost

        flags.set(CpuFlags::FLAG_Z, *p1 == 0);
        flags.set(CpuFlags::FLAG_N, false);
        flags.set(CpuFlags::FLAG_H, false);
    }

    fn swap_r8(p1: &mut u8)
    {
        let lower_to_upper_half = *p1 << 4u8;
        let upper_to_lower_half = *p1 >> 4u8;
        *p1 = lower_to_upper_half | upper_to_lower_half;
    }

    fn bit_r8(p1: u8, p2: &mut u8, flags: &mut CpuFlags)
    {
        flags.set(CpuFlags::FLAG_H, true);
        flags.set(CpuFlags::FLAG_N, false);
        flags.set(CpuFlags::FLAG_Z, (*p2 & (1u8 << p1)) == 0);
    }

    fn res_r8(p1: u8, p2: &mut u8)
    {
        *p2 &= !(1u8 << p1);
    }

    fn set_r8(p1: u8, p2: &mut u8)
    {
        *p2 |= 1u8 << p1;
    }

    fn push_r16(ram: &mut Ram, sp: &mut u16, msh: &mut u8, lsh: &mut u8)
    {
        ram.write(*sp - 1, *msh);
        ram.write(*sp - 2, *lsh);
        *sp -= 2;
    }

    fn push_pc(ram: &mut Ram, sp: &mut u16, pc: &mut ProgramCounter)
    {
        let bytes = pc.reg.to_le_bytes();
        ram.write(*sp - 1, bytes[1]);
        ram.write(*sp - 2, bytes[0]);
        *sp -= 2;
    }

    fn pop_r16(ram: &mut Ram, sp: &mut u16, msh: &mut u8, lsh: &mut u8)
    {
        *lsh = ram.read(*sp);
        *msh = ram.read(*sp + 1);
        *sp += 2;
    }

    //----------INTERRUPT MANAGEMENT----------

    fn reti(ram: &mut Ram, sp: &mut u16, pc: &mut ProgramCounter, ime: &mut bool)
    {
        let l_bytes = ram.read(*sp);
        Cpu::inc_sp(sp);
        let h_bytes = ram.read(*sp);
        Cpu::inc_sp(sp);
        pc.reg = u16::from_le_bytes([l_bytes, h_bytes]);
        *ime = true;
    }

    fn ei(ime: &mut bool)
    {
        *ime = true;
    }

    fn di(ime: &mut bool)
    {
        *ime = false;
    }

    //----------EXECUTION FUNCTIONS----------

    fn aux_inc_pc(&mut self)
    {
        if self.pc.should_increment
        {
            self.pc.reg += 1;
            self.pc.current_instruction_width = 0;
        }
        else
        {
            self.pc.should_increment = true;
        }
    }

    fn aux_read_pc(&self,  ram: &mut Ram) -> u8
    {
        ram.read(self.pc.reg)
    }

    fn aux_read_immediate_data(&mut self, ram: &mut Ram) -> u8
    {
        self.pc.reg += 1;
        self.pc.current_instruction_width += 1;
        ram.read(self.pc.reg)
    }

    pub fn execute(&mut self, ram: &mut Ram)
    {
        if self.pc.current_instruction_cycles > 1
        {
            self.pc.current_instruction_cycles -= 1;
            return;
        }

        //Fetch
        let valid_interrupts = ram::InterruptFlag::from_bits(ram.read(ram::IF) & ram.read(ram::IE)).unwrap();
        if self.ime
        {
            if !valid_interrupts.is_empty()
            {
                self.ime = false;
                Cpu::push_pc(ram, &mut self.sp, &mut self.pc);
            }
            if valid_interrupts.contains(ram::InterruptFlag::VB)
            {
                self.pc.reg = 0x0040;
            }
            else if valid_interrupts.contains(ram::InterruptFlag::LCDC)
            {
                self.pc.reg = 0x0048;
            }
            else if valid_interrupts.contains(ram::InterruptFlag::TIMA)
            {
                self.pc.reg = 0x0050;
            }
            else if valid_interrupts.contains(ram::InterruptFlag::SIO_TRANSFER_COMPLETE)
            {
                self.pc.reg = 0x0058;
            }
            else if valid_interrupts.contains(ram::InterruptFlag::P1X_NEG_EDGE)
            {
                self.pc.reg = 0x0060;
            }

        }

        if self.halted
        {
            if !valid_interrupts.is_empty()
            {
                self.halted = false;
            }
            else
            {
                return;
            }
        }

        let instruction = self.aux_read_pc(ram);

        #[cfg(feature = "cpu-debug")]
        println!("Instruction: 0x{:02X?}, Program Counter: 0x{:02X?}", instruction, &self.pc.reg);

        if self.pc.reg == 0xFA
        {
            println!("Copy check");
        }

        if instruction != 0xCB
        {
            match instruction
            {
                0x00 => {/* NOP */},
                0x01 => {
                    let lsh = self.aux_read_immediate_data(ram);
                    let msh = self.aux_read_immediate_data(ram);
                    Cpu::ld_r16_16(&mut self.reg_b, &mut self.reg_c, msh, lsh);
                },
                0x02 => {Cpu::ld_r8_r8(ram.get_rp_ref(self.reg_b, self.reg_c), &mut self.reg_a);},
                0x03 => {Cpu::inc_r16(&mut self.reg_b, &mut self.reg_c);},
                0x04 => {Cpu::inc_r8(&mut self.reg_b, &mut self.reg_f);},
                0x05 => {Cpu::dec_r8(&mut self.reg_b, &mut self.reg_f);},
                0x06 => {
                    let num = self.aux_read_immediate_data(ram);
                    Cpu::ld_r8_8(&mut self.reg_b, num);
                },
                0x07 => {Cpu::rlca(&mut self.reg_a, &mut self.reg_f);},
                0x08 => {
                    let lsh = self.aux_read_immediate_data(ram);
                    let msh = self.aux_read_immediate_data(ram);
                    Cpu::ld_16a_sp(&mut self.sp, ram, msh, lsh);
                },
                0x09 => {Cpu::add_r16_r16(&mut self.reg_h, &mut self.reg_l, &mut self.reg_b, &mut self.reg_c, &mut self.reg_f);},
                0x0A => {Cpu::ld_r8_r8(&mut self.reg_a, ram.get_rp_ref(self.reg_b, self.reg_c));},
                0x0B => {Cpu::dec_r16(&mut self.reg_b, &mut self.reg_c);},
                0x0C => {Cpu::inc_r8(&mut self.reg_c, &mut self.reg_f);},
                0x0D => {Cpu::dec_r8(&mut self.reg_c, &mut self.reg_f);},
                0x0E => {
                    let num = self.aux_read_immediate_data(ram);
                    Cpu::ld_r8_8(&mut self.reg_c, num);
                },
                0x0F => {Cpu::rrca(&mut self.reg_a, &mut self.reg_f);},
                0x10 => {self.stop();},
                0x11 => {
                    let lsh = self.aux_read_immediate_data(ram);
                    let msh = self.aux_read_immediate_data(ram);
                    Cpu::ld_r16_16(&mut self.reg_d, &mut self.reg_e, msh, lsh);
                },
                0x12 => {Cpu::ld_r8_r8(ram.get_rp_ref(self.reg_d, self.reg_e), &mut self.reg_a);},
                0x13 => {Cpu::inc_r16(&mut self.reg_d, &mut self.reg_e);},
                0x14 => {Cpu::inc_r8(&mut self.reg_d, &mut self.reg_f);},
                0x15 => {Cpu::dec_r8(&mut self.reg_d, &mut self.reg_f);},
                0x16 => {
                    let num = self.aux_read_immediate_data(ram);
                    Cpu::ld_r8_8(&mut self.reg_d, num);
                },
                0x17 => {Cpu::rla(&mut self.reg_a, &mut self.reg_f);},
                0x18 => {
                    let immediate = self.aux_read_immediate_data(ram) as i8;
                    Cpu::jr_i8(&mut self.pc, immediate);
                },
                0x19 => {Cpu::add_r16_r16(&mut self.reg_h, &mut self.reg_l, &mut self.reg_d, &mut self.reg_e, &mut self.reg_f);},
                0x1A => {Cpu::ld_r8_r8(&mut self.reg_a, ram.get_rp_ref(self.reg_d, self.reg_e));},
                0x1B => {Cpu::dec_r16(&mut self.reg_d, &mut self.reg_e);},
                0x1C => {Cpu::inc_r8(&mut self.reg_e, &mut self.reg_f);},
                0x1D => {Cpu::dec_r8(&mut self.reg_e, &mut self.reg_f);},
                0x1E => {
                    let num = self.aux_read_immediate_data(ram);
                    Cpu::ld_r8_8(&mut self.reg_e, num);
                },
                0x1F => {Cpu::rra(&mut self.reg_a, &mut self.reg_f);},
                0x20 => {
                    let immediate = self.aux_read_immediate_data(ram) as i8;
                    Cpu::jr_nflag_i8(&mut self.pc, CpuFlags::FLAG_Z, immediate, &mut self.reg_f);
                },
                0x21 => {
                    let lsh = self.aux_read_immediate_data(ram);
                    let msh = self.aux_read_immediate_data(ram);
                    Cpu::ld_r16_16(&mut self.reg_h, &mut self.reg_l, msh, lsh);
                },
                0x22 => {
                    Cpu::ld_r8_r8(ram.get_rp_ref(self.reg_h, self.reg_l), &mut self.reg_a);
                    Cpu::inc_r16(&mut self.reg_h, &mut self.reg_l);
                },
                0x23 => {Cpu::inc_r16(&mut self.reg_h, &mut self.reg_l);},
                0x24 => {Cpu::inc_r8(&mut self.reg_h, &mut self.reg_f);},
                0x25 => {Cpu::dec_r8(&mut self.reg_h, &mut self.reg_f);},
                0x26 => {
                    let num = self.aux_read_immediate_data(ram);
                    Cpu::ld_r8_8(&mut self.reg_h, num);
                },
                0x27 => {Cpu::daa(&mut self.reg_a, &mut self.reg_f);},
                0x28 => {
                    let immediate = self.aux_read_immediate_data(ram) as i8;
                    Cpu::jr_flag_i8(&mut self.pc, CpuFlags::FLAG_Z, immediate, &mut self.reg_f);
                },
                0x29 => {
                    Cpu::add_r16_r16_s(&mut self.reg_h, &mut self.reg_l, &mut self.reg_f)},
                0x2A => {
                    Cpu::ld_r8_r8(&mut self.reg_a, ram.get_rp_ref(self.reg_h, self.reg_l));
                    Cpu::inc_r16(&mut self.reg_h, &mut self.reg_l);
                },
                0x2B => {Cpu::dec_r16(&mut self.reg_h, &mut self.reg_l);},
                0x2C => {Cpu::inc_r8(&mut self.reg_l, &mut self.reg_f);},
                0x2D => {Cpu::dec_r8(&mut self.reg_l, &mut self.reg_f);},
                0x2E => {
                    let num = self.aux_read_immediate_data(ram);
                    Cpu::ld_r8_8(&mut self.reg_l, num);
                },
                0x2F => {Cpu::cpl(&mut self.reg_a, &mut self.reg_f);},
                0x30 => {
                    let immediate = self.aux_read_immediate_data(ram) as i8;
                    Cpu::jr_nflag_i8(&mut self.pc, CpuFlags::FLAG_C, immediate, &mut self.reg_f);
                },
                0x31 => {
                    let lsh = self.aux_read_immediate_data(ram);
                    let msh = self.aux_read_immediate_data(ram);
                    Cpu::ld_sp_16( &mut self.sp, msh, lsh);
                },
                0x32 => {
                    Cpu::ld_r8_r8(ram.get_rp_ref(self.reg_h, self.reg_l), &mut self.reg_a);
                    Cpu::dec_r16(&mut self.reg_h, &mut self.reg_l);
                },
                0x33 => {Cpu::inc_sp(&mut self.sp);},
                0x34 => {Cpu::inc_r8(ram.get_rp_ref(self.reg_h, self.reg_l), &mut self.reg_f);},
                0x35 => {Cpu::dec_r8(ram.get_rp_ref(self.reg_h, self.reg_l), &mut self.reg_f);},
                0x36 => {
                    let num = self.aux_read_immediate_data(ram);
                    Cpu::ld_r8_8(ram.get_rp_ref(self.reg_h, self.reg_l), num);
                },
                0x37 => {Cpu::scf(&mut self.reg_f);},
                0x38 => {
                    let immediate = self.aux_read_immediate_data(ram) as i8;
                    Cpu::jr_flag_i8(&mut self.pc, CpuFlags::FLAG_C, immediate, &mut self.reg_f);
                },
                0x39 => {Cpu::add_r16_sp( &mut self.reg_h, &mut self.reg_l, &mut self.sp, &mut self.reg_f);},
                0x3A => {
                    Cpu::ld_r8_r8(&mut self.reg_a, ram.get_rp_ref(self.reg_h, self.reg_l));
                    Cpu::dec_r16(&mut self.reg_h, &mut self.reg_l);
                },
                0x3B => {Cpu::dec_sp(&mut self.sp);},
                0x3C => {Cpu::inc_r8(&mut self.reg_a, &mut self.reg_f);},
                0x3D => {Cpu::dec_r8(&mut self.reg_a, &mut self.reg_f);},
                0x3E => {
                    let num = self.aux_read_immediate_data(ram);
                    Cpu::ld_r8_8(&mut self.reg_a, num);
                },
                0x3F => {Cpu::ccf(&mut self.reg_f);},
                0x40 => {Cpu::ld_r8_r8_s(&mut self.reg_b);},
                0x41 => {Cpu::ld_r8_r8(&mut self.reg_b, &mut self.reg_c);},
                0x42 => {Cpu::ld_r8_r8(&mut self.reg_b, &mut self.reg_d);},
                0x43 => {Cpu::ld_r8_r8(&mut self.reg_b, &mut self.reg_e);},
                0x44 => {Cpu::ld_r8_r8(&mut self.reg_b, &mut self.reg_h);},
                0x45 => {Cpu::ld_r8_r8(&mut self.reg_b, &mut self.reg_l);},
                0x46 => {Cpu::ld_r8_r8(&mut self.reg_b, ram.get_rp_ref(self.reg_h, self.reg_l));},
                0x47 => {Cpu::ld_r8_r8(&mut self.reg_b, &mut self.reg_a);},
                0x48 => {Cpu::ld_r8_r8(&mut self.reg_c, &mut self.reg_b);},
                0x49 => {Cpu::ld_r8_r8_s(&mut self.reg_c);},
                0x4A => {Cpu::ld_r8_r8(&mut self.reg_c, &mut self.reg_d);},
                0x4B => {Cpu::ld_r8_r8(&mut self.reg_c, &mut self.reg_e);},
                0x4C => {Cpu::ld_r8_r8(&mut self.reg_c, &mut self.reg_h);},
                0x4D => {Cpu::ld_r8_r8(&mut self.reg_c, &mut self.reg_l);},
                0x4E => {Cpu::ld_r8_r8(&mut self.reg_c, ram.get_rp_ref(self.reg_h, self.reg_l));},
                0x4F => {Cpu::ld_r8_r8(&mut self.reg_c, &mut self.reg_a);},
                0x50 => {Cpu::ld_r8_r8(&mut self.reg_d, &mut self.reg_b);},
                0x51 => {Cpu::ld_r8_r8(&mut self.reg_d, &mut self.reg_c);},
                0x52 => {Cpu::ld_r8_r8_s(&mut self.reg_d);},
                0x53 => {Cpu::ld_r8_r8(&mut self.reg_d, &mut self.reg_e);},
                0x54 => {Cpu::ld_r8_r8(&mut self.reg_d, &mut self.reg_h);},
                0x55 => {Cpu::ld_r8_r8(&mut self.reg_d, &mut self.reg_l);},
                0x56 => {Cpu::ld_r8_r8(&mut self.reg_d, ram.get_rp_ref(self.reg_h, self.reg_l));},
                0x57 => {Cpu::ld_r8_r8(&mut self.reg_d, &mut self.reg_a);},
                0x58 => {Cpu::ld_r8_r8(&mut self.reg_e, &mut self.reg_b);},
                0x59 => {Cpu::ld_r8_r8(&mut self.reg_e, &mut self.reg_c);},
                0x5A => {Cpu::ld_r8_r8(&mut self.reg_e, &mut self.reg_d);},
                0x5B => {Cpu::ld_r8_r8_s(&mut self.reg_e);},
                0x5C => {Cpu::ld_r8_r8(&mut self.reg_e, &mut self.reg_h);},
                0x5D => {Cpu::ld_r8_r8(&mut self.reg_e, &mut self.reg_l);},
                0x5E => {Cpu::ld_r8_r8(&mut self.reg_e, ram.get_rp_ref(self.reg_h, self.reg_l));},
                0x5F => {Cpu::ld_r8_r8(&mut self.reg_e, &mut self.reg_a);},
                0x60 => {Cpu::ld_r8_r8(&mut self.reg_h, &mut self.reg_b);},
                0x61 => {Cpu::ld_r8_r8(&mut self.reg_h, &mut self.reg_c);},
                0x62 => {Cpu::ld_r8_r8(&mut self.reg_h, &mut self.reg_d);},
                0x63 => {Cpu::ld_r8_r8(&mut self.reg_h, &mut self.reg_e);},
                0x64 => {Cpu::ld_r8_r8_s(&mut self.reg_h);},
                0x65 => {Cpu::ld_r8_r8(&mut self.reg_h, &mut self.reg_l);},
                0x66 => {
                    let mut ram_read = ram.read_rp(self.reg_h, self.reg_l);
                    Cpu::ld_r8_r8(&mut self.reg_h, &mut ram_read);
                },
                0x67 => {Cpu::ld_r8_r8(&mut self.reg_h, &mut self.reg_a);},
                0x68 => {Cpu::ld_r8_r8(&mut self.reg_l, &mut self.reg_b);},
                0x69 => {Cpu::ld_r8_r8(&mut self.reg_l, &mut self.reg_c);},
                0x6A => {Cpu::ld_r8_r8(&mut self.reg_l, &mut self.reg_d);},
                0x6B => {Cpu::ld_r8_r8(&mut self.reg_l, &mut self.reg_e);},
                0x6C => {Cpu::ld_r8_r8(&mut self.reg_l, &mut self.reg_h);},
                0x6D => {Cpu::ld_r8_r8_s(&mut self.reg_l);},
                0x6E => {
                    let mut ram_read = ram.read_rp(self.reg_h, self.reg_l);
                    Cpu::ld_r8_r8(&mut self.reg_l, &mut ram_read);
                },
                0x6F => {Cpu::ld_r8_r8(&mut self.reg_l, &mut self.reg_a);},
                0x70 => {Cpu::ld_r8_r8(ram.get_rp_ref(self.reg_h, self.reg_l), &mut self.reg_b);},
                0x71 => {Cpu::ld_r8_r8(ram.get_rp_ref(self.reg_h, self.reg_l), &mut self.reg_c);},
                0x72 => {Cpu::ld_r8_r8(ram.get_rp_ref(self.reg_h, self.reg_l), &mut self.reg_d);},
                0x73 => {Cpu::ld_r8_r8(ram.get_rp_ref(self.reg_h, self.reg_l), &mut self.reg_e);},
                0x74 => {
                    let ram_read = ram.get_rp_ref(self.reg_h, self.reg_l);
                    Cpu::ld_r8_r8(ram_read, &mut self.reg_h);
                },
                0x75 => {
                    let ram_read = ram.get_rp_ref(self.reg_h, self.reg_l);
                    Cpu::ld_r8_r8(ram_read, &mut self.reg_l);
                },
                0x76 => {self.halt();},
                0x77 => {Cpu::ld_r8_r8(ram.get_rp_ref(self.reg_h, self.reg_l), &mut self.reg_a);},
                0x78 => {Cpu::ld_r8_r8(&mut self.reg_a, &mut self.reg_b);},
                0x79 => {Cpu::ld_r8_r8(&mut self.reg_a, &mut self.reg_c);},
                0x7A => {Cpu::ld_r8_r8(&mut self.reg_a, &mut self.reg_d);},
                0x7B => {Cpu::ld_r8_r8(&mut self.reg_a, &mut self.reg_e);},
                0x7C => {Cpu::ld_r8_r8(&mut self.reg_a, &mut self.reg_h);},
                0x7D => {Cpu::ld_r8_r8(&mut self.reg_a, &mut self.reg_l);},
                0x7E => {Cpu::ld_r8_r8(&mut self.reg_a, ram.get_rp_ref(self.reg_h, self.reg_l));},
                0x7F => {Cpu::ld_r8_r8_s(&mut self.reg_a);},
                0x80 => {Cpu::add_r8_r8(&mut self.reg_a, &mut self.reg_b, &mut self.reg_f);},
                0x81 => {Cpu::add_r8_r8(&mut self.reg_a, &mut self.reg_c, &mut self.reg_f);},
                0x82 => {Cpu::add_r8_r8(&mut self.reg_a, &mut self.reg_d, &mut self.reg_f);},
                0x83 => {Cpu::add_r8_r8(&mut self.reg_a, &mut self.reg_e, &mut self.reg_f);},
                0x84 => {Cpu::add_r8_r8(&mut self.reg_a, &mut self.reg_h, &mut self.reg_f);},
                0x85 => {Cpu::add_r8_r8(&mut self.reg_a, &mut self.reg_l, &mut self.reg_f);},
                0x86 => {Cpu::add_r8_r8(&mut self.reg_a, ram.get_rp_ref(self.reg_h, self.reg_l), &mut self.reg_f);},
                0x87 => {Cpu::add_r8_r8_s(&mut self.reg_a, &mut self.reg_f);},
                0x88 => {Cpu::adc_r8_r8(&mut self.reg_a, &mut self.reg_b, &mut self.reg_f);},
                0x89 => {Cpu::adc_r8_r8(&mut self.reg_a, &mut self.reg_c, &mut self.reg_f);},
                0x8A => {Cpu::adc_r8_r8(&mut self.reg_a, &mut self.reg_d, &mut self.reg_f);},
                0x8B => {Cpu::adc_r8_r8(&mut self.reg_a, &mut self.reg_e, &mut self.reg_f);},
                0x8C => {Cpu::adc_r8_r8(&mut self.reg_a, &mut self.reg_h, &mut self.reg_f);},
                0x8D => {Cpu::adc_r8_r8(&mut self.reg_a, &mut self.reg_l, &mut self.reg_f);},
                0x8E => {Cpu::adc_r8_r8(&mut self.reg_a, ram.get_rp_ref(self.reg_h, self.reg_l), &mut self.reg_f);},
                0x8F => {Cpu::adc_r8_r8_s(&mut self.reg_a, &mut self.reg_f);},
                0x90 => {Cpu::sub_r8_r8(&mut self.reg_a, &mut self.reg_b, &mut self.reg_f);},
                0x91 => {Cpu::sub_r8_r8(&mut self.reg_a, &mut self.reg_c, &mut self.reg_f);},
                0x92 => {Cpu::sub_r8_r8(&mut self.reg_a, &mut self.reg_d, &mut self.reg_f);},
                0x93 => {Cpu::sub_r8_r8(&mut self.reg_a, &mut self.reg_e, &mut self.reg_f);},
                0x94 => {Cpu::sub_r8_r8(&mut self.reg_a, &mut self.reg_h, &mut self.reg_f);},
                0x95 => {Cpu::sub_r8_r8(&mut self.reg_a, &mut self.reg_l, &mut self.reg_f);},
                0x96 => {Cpu::sub_r8_r8(&mut self.reg_a, ram.get_rp_ref(self.reg_h, self.reg_l), &mut self.reg_f);},
                0x97 => {Cpu::sub_r8_r8_s(&mut self.reg_a, &mut self.reg_f);},
                0x98 => {Cpu::sbc_r8_r8(&mut self.reg_a, &mut self.reg_b, &mut self.reg_f);},
                0x99 => {Cpu::sbc_r8_r8(&mut self.reg_a, &mut self.reg_c, &mut self.reg_f);},
                0x9A => {Cpu::sbc_r8_r8(&mut self.reg_a, &mut self.reg_d, &mut self.reg_f);},
                0x9B => {Cpu::sbc_r8_r8(&mut self.reg_a, &mut self.reg_e, &mut self.reg_f);},
                0x9C => {Cpu::sbc_r8_r8(&mut self.reg_a, &mut self.reg_h, &mut self.reg_f);},
                0x9D => {Cpu::sbc_r8_r8(&mut self.reg_a, &mut self.reg_l, &mut self.reg_f);},
                0x9E => {Cpu::sbc_r8_r8(&mut self.reg_a, ram.get_rp_ref(self.reg_h, self.reg_l), &mut self.reg_f);},
                0x9F => {Cpu::sbc_r8_r8_s(&mut self.reg_a, &mut self.reg_f);},
                0xA0 => {Cpu::and_r8_r8(&mut self.reg_a, &mut self.reg_b, &mut self.reg_f);},
                0xA1 => {Cpu::and_r8_r8(&mut self.reg_a, &mut self.reg_c, &mut self.reg_f);},
                0xA2 => {Cpu::and_r8_r8(&mut self.reg_a, &mut self.reg_d, &mut self.reg_f);},
                0xA3 => {Cpu::and_r8_r8(&mut self.reg_a, &mut self.reg_e, &mut self.reg_f);},
                0xA4 => {Cpu::and_r8_r8(&mut self.reg_a, &mut self.reg_h, &mut self.reg_f);},
                0xA5 => {Cpu::and_r8_r8_s(&mut self.reg_a, &mut self.reg_f);},
                0xA6 => {Cpu::and_r8_r8(&mut self.reg_a, ram.get_rp_ref(self.reg_h, self.reg_l), &mut self.reg_f);},
                0xA7 => {Cpu::and_r8_r8_s(&mut self.reg_a, &mut self.reg_f);},
                0xA8 => {Cpu::xor_r8_r8(&mut self.reg_a, &mut self.reg_b, &mut self.reg_f);},
                0xA9 => {Cpu::xor_r8_r8(&mut self.reg_a, &mut self.reg_c, &mut self.reg_f);},
                0xAA => {Cpu::xor_r8_r8(&mut self.reg_a, &mut self.reg_d, &mut self.reg_f);},
                0xAB => {Cpu::xor_r8_r8(&mut self.reg_a, &mut self.reg_e, &mut self.reg_f);},
                0xAC => {Cpu::xor_r8_r8(&mut self.reg_a, &mut self.reg_h, &mut self.reg_f);},
                0xAD => {Cpu::xor_r8_r8(&mut self.reg_a, &mut self.reg_l, &mut self.reg_f);},
                0xAE => {Cpu::xor_r8_r8(&mut self.reg_a, ram.get_rp_ref(self.reg_h, self.reg_l), &mut self.reg_f);},
                0xAF => {Cpu::xor_r8_r8_s(&mut self.reg_a, &mut self.reg_f);},
                0xB0 => {Cpu::or_r8_r8(&mut self.reg_a, &mut self.reg_b, &mut self.reg_f);},
                0xB1 => {Cpu::or_r8_r8(&mut self.reg_a, &mut self.reg_c, &mut self.reg_f);},
                0xB2 => {Cpu::or_r8_r8(&mut self.reg_a, &mut self.reg_d, &mut self.reg_f);},
                0xB3 => {Cpu::or_r8_r8(&mut self.reg_a, &mut self.reg_e, &mut self.reg_f);},
                0xB4 => {Cpu::or_r8_r8(&mut self.reg_a, &mut self.reg_h, &mut self.reg_f);},
                0xB5 => {Cpu::or_r8_r8(&mut self.reg_a, &mut self.reg_l, &mut self.reg_f);},
                0xB6 => {Cpu::or_r8_r8(&mut self.reg_a, ram.get_rp_ref(self.reg_h, self.reg_l), &mut self.reg_f);},
                0xB7 => {Cpu::or_r8_r8_s(&mut self.reg_a, &mut self.reg_f);},
                0xB8 => {Cpu::cp_r8_r8(&mut self.reg_a, &mut self.reg_b, &mut self.reg_f);},
                0xB9 => {Cpu::cp_r8_r8(&mut self.reg_a, &mut self.reg_c, &mut self.reg_f);},
                0xBA => {Cpu::cp_r8_r8(&mut self.reg_a, &mut self.reg_d, &mut self.reg_f);},
                0xBB => {Cpu::cp_r8_r8(&mut self.reg_a, &mut self.reg_e, &mut self.reg_f);},
                0xBC => {Cpu::cp_r8_r8(&mut self.reg_a, &mut self.reg_h, &mut self.reg_f);},
                0xBD => {Cpu::cp_r8_r8(&mut self.reg_a, &mut self.reg_l, &mut self.reg_f);},
                0xBE => {Cpu::cp_r8_r8(&mut self.reg_a, ram.get_rp_ref(self.reg_h, self.reg_l), &mut self.reg_f);},
                0xBF => {Cpu::cp_r8_r8_s(&mut self.reg_a, &mut self.reg_f);},
                0xC0 => {Cpu::ret_nflag(ram, &mut self.pc, &mut self.sp, CpuFlags::FLAG_Z, &mut self.reg_f);},
                0xC1 => {Cpu::pop_r16(ram, &mut self.sp, &mut self.reg_b, &mut self.reg_c);},
                0xC2 => {
                    let lsh = self.aux_read_immediate_data(ram);
                    let msh = self.aux_read_immediate_data(ram);
                    Cpu::jp_nflag_pc_16(&mut self.pc, CpuFlags::FLAG_Z, msh, lsh, &mut self.reg_f);
                },
                0xC3 => {
                    let lsh = self.aux_read_immediate_data(ram);
                    let msh = self.aux_read_immediate_data(ram);
                    Cpu::jp_pc_16(&mut self.pc, msh, lsh);
                },
                0xC4 => {
                    let lsh = self.aux_read_immediate_data(ram);
                    let msh = self.aux_read_immediate_data(ram);
                    Cpu::call_nflag_16(ram, CpuFlags::FLAG_Z, msh, lsh, &mut self.pc, &mut self.sp, &mut self.reg_f);
                },
                0xC5 => {Cpu::push_r16(ram, &mut self.sp, &mut self.reg_b, &mut self.reg_c);},
                0xC6 => {
                    let num = self.aux_read_immediate_data(ram);
                    Cpu::add_r8_8(&mut self.reg_a, num, &mut self.reg_f);
                },
                0xC7 => {Cpu::rst(ram, 0x00, &mut self.pc, &mut self.sp);},
                0xC8 => {Cpu::ret_flag(ram, &mut self.pc, &mut self.sp, CpuFlags::FLAG_Z, &mut self.reg_f);},
                0xC9 => {Cpu::ret(ram, &mut self.pc, &mut self.sp);},
                0xCA => {
                    let lsh = self.aux_read_immediate_data(ram);
                    let msh = self.aux_read_immediate_data(ram);
                    Cpu::jp_flag_pc_16(&mut self.pc, CpuFlags::FLAG_Z, msh, lsh, &mut self.reg_f);
                },
                0xCB => {/*Prefix for the next instruction, handled earlier*/},
                0xCC => {
                    let lsh = self.aux_read_immediate_data(ram);
                    let msh = self.aux_read_immediate_data(ram);
                    Cpu::call_flag_16(ram, CpuFlags::FLAG_Z, msh, lsh, &mut self.pc, &mut self.sp, &mut self.reg_f);
                },
                0xCD => {let lsh = self.aux_read_immediate_data(ram);
                    let msh = self.aux_read_immediate_data(ram);
                    Cpu::call_16(ram, msh, lsh, &mut self.pc, &mut self.sp);},
                0xCE => {
                    let num = self.aux_read_immediate_data(ram);
                    Cpu::adc_r8_8(&mut self.reg_a, num, &mut self.reg_f);
                },
                0xCF => {Cpu::rst(ram, 0x08, &mut self.pc, &mut self.sp);},
                0xD0 => {Cpu::ret_nflag(ram, &mut self.pc, &mut self.sp, CpuFlags::FLAG_C, &mut self.reg_f);},
                0xD1 => {Cpu::pop_r16(ram, &mut self.sp, &mut self.reg_d, &mut self.reg_e);},
                0xD2 => {
                    let lsh = self.aux_read_immediate_data(ram);
                    let msh = self.aux_read_immediate_data(ram);
                    Cpu::jp_nflag_pc_16(&mut self.pc, CpuFlags::FLAG_C, msh, lsh, &mut self.reg_f);
                },
                0xD3 => {self.invalid_instruction(0xD3);},
                0xD4 => {
                    let lsh = self.aux_read_immediate_data(ram);
                    let msh = self.aux_read_immediate_data(ram);
                    Cpu::call_nflag_16(ram, CpuFlags::FLAG_C, msh, lsh, &mut self.pc, &mut self.sp, &mut self.reg_f);
                },
                0xD5 => {Cpu::push_r16(ram, &mut self.sp, &mut self.reg_d, &mut self.reg_e);},
                0xD6 => {
                    let num = self.aux_read_immediate_data(ram);
                    Cpu::sub_r8_8(&mut self.reg_a, num, &mut self.reg_f);
                },
                0xD7 => {Cpu::rst(ram, 0x10, &mut self.pc, &mut self.sp);},
                0xD8 => {Cpu::ret_flag(ram, &mut self.pc, &mut self.sp, CpuFlags::FLAG_C, &mut self.reg_f);},
                0xD9 => {Cpu::reti(ram, &mut self.sp, &mut self.pc, &mut self.ime);},
                0xDA => {
                    let lsh = self.aux_read_immediate_data(ram);
                    let msh = self.aux_read_immediate_data(ram);
                    Cpu::jp_flag_pc_16(&mut self.pc, CpuFlags::FLAG_C, msh, lsh, &mut self.reg_f);
                },
                0xDB => {self.invalid_instruction(0xDB);},
                0xDC => {
                    let lsh = self.aux_read_immediate_data(ram);
                    let msh = self.aux_read_immediate_data(ram);
                    Cpu::call_flag_16(ram, CpuFlags::FLAG_C, msh, lsh, &mut self.pc, &mut self.sp, &mut self.reg_f);
                },
                0xDD => {self.invalid_instruction(0xDD);},
                0xDE => {
                    let num = self.aux_read_immediate_data(ram);
                    Cpu::sbc_r8_8(&mut self.reg_a, num, &mut self.reg_f);
                },
                0xDF => {Cpu::rst(ram, 0x18, &mut self.pc, &mut self.sp);},
                0xE0 => {
                    let lsh = self.aux_read_immediate_data(ram);
                    Cpu::ld_r8_r8(ram.get_rp_ref(0xFF, lsh), &mut self.reg_a);
                },
                0xE1 => {Cpu::pop_r16(ram, &mut self.sp, &mut self.reg_h, &mut self.reg_l);},
                0xE2 => {Cpu::ld_r8_r8(ram.get_rp_ref(0xFF, self.reg_c), &mut self.reg_a);},
                0xE3 => {self.invalid_instruction(0xE3);},
                0xE4 => {self.invalid_instruction(0xE4);},
                0xE5 => {Cpu::push_r16(ram, &mut self.sp, &mut self.reg_h, &mut self.reg_l);},
                0xE6 => {
                    let num = self.aux_read_immediate_data(ram);
                    Cpu::and_r8_8(&mut self.reg_a, num, &mut self.reg_f);
                },
                0xE7 => {Cpu::rst(ram, 0x20, &mut self.pc, &mut self.sp);},
                0xE8 => {
                    let immediate = self.aux_read_immediate_data(ram) as i8;
                    Cpu::add_sp_i8(&mut self.sp, immediate);
                },
                0xE9 => {Cpu::jp_pc_16(&mut self.pc, self.reg_h, self.reg_l);},
                0xEA => {
                    let lsh = self.aux_read_immediate_data(ram);
                    let msh = self.aux_read_immediate_data(ram);
                    Cpu::ld_r8_r8(ram.get_rp_ref(msh, lsh), &mut self.reg_a);
                },
                0xEB => {self.invalid_instruction(0xEB);},
                0xEC => {self.invalid_instruction(0xEC);},
                0xED => {self.invalid_instruction(0xED);},
                0xEE => {
                    let num = self.aux_read_immediate_data(ram);
                    Cpu::xor_r8_8(&mut self.reg_a, num, &mut self.reg_f);
                },
                0xEF => {Cpu::rst(ram, 0x28, &mut self.pc, &mut self.sp);},
                0xF0 => {
                    let lsh = self.aux_read_immediate_data(ram);
                    Cpu::ld_r8_r8(&mut self.reg_a, ram.get_rp_ref(0xFF, lsh))
                },
                0xF1 => {Cpu::pop_r16(ram, &mut self.sp, &mut self.reg_a, &mut self.reg_f.bits);},
                0xF2 => {Cpu::ld_r8_r8(&mut self.reg_a, ram.get_rp_ref(0xFF, self.reg_c));},
                0xF3 => {Cpu::di(&mut self.ime);},
                0xF4 => {self.invalid_instruction(0xF4);},
                0xF5 => {Cpu::push_r16(ram, &mut self.sp, &mut self.reg_a, &mut self.reg_f.bits);},
                0xF6 => {
                    let num = self.aux_read_immediate_data(ram);
                    Cpu::or_r8_8(&mut self.reg_a, num, &mut self.reg_f);
                },
                0xF7 => {Cpu::rst(ram, 0x30, &mut self.pc, &mut self.sp);},
                0xF8 => {
                    let immediate = self.aux_read_immediate_data(ram) as i8;
                    Cpu::ld_hl_sp_plus(&mut self.sp, &mut self.reg_h, &mut self.reg_l, immediate);
                },
                0xF9 => {Cpu::ld_sp_r16(&mut self.sp, &mut self.reg_h, &mut self.reg_l);},
                0xFA => {
                    let lsh = self.aux_read_immediate_data(ram);
                    let msh = self.aux_read_immediate_data(ram);
                    Cpu::ld_r8_r8(&mut self.reg_a, ram.get_rp_ref(msh, lsh));
                },
                0xFB => {Cpu::ei(&mut self.ime);},
                0xFC => {self.invalid_instruction(0xFC);},
                0xFD => {self.invalid_instruction(0xFD);},
                0xFE => {
                    let num = self.aux_read_immediate_data(ram);
                    Cpu::cp_r8_8(&mut self.reg_a, num, &mut self.reg_f);
                },
                0xFF => {Cpu::rst(ram, 0x38, &mut self.pc, &mut self.sp);}
            }

            self.pc.current_instruction_cycles += ZERO_INSTRUCTION_TIME_TABLE[instruction as usize];
        }
        else
        {
            let cb_instruction = self.aux_read_immediate_data(ram);

            // println!("Instruction: 0x{:02X?}, Program Counter: 0x{:02X?}", cb_instruction, self.pc.reg);

            match cb_instruction //CB Prefix
            {
                0x00 => {Cpu::rlc_r8(&mut self.reg_b, &mut self.reg_f);},
                0x01 => {Cpu::rlc_r8(&mut self.reg_c, &mut self.reg_f);},
                0x02 => {Cpu::rlc_r8(&mut self.reg_d, &mut self.reg_f);},
                0x03 => {Cpu::rlc_r8(&mut self.reg_e, &mut self.reg_f);},
                0x04 => {Cpu::rlc_r8(&mut self.reg_h, &mut self.reg_f);},
                0x05 => {Cpu::rlc_r8(&mut self.reg_l, &mut self.reg_f);},
                0x06 => {Cpu::rlc_r8(ram.get_rp_ref(self.reg_h, self.reg_l), &mut self.reg_f);},
                0x07 => {Cpu::rlc_r8(&mut self.reg_a, &mut self.reg_f);},
                0x08 => {Cpu::rrc_r8(&mut self.reg_b, &mut self.reg_f);},
                0x09 => {Cpu::rrc_r8(&mut self.reg_c, &mut self.reg_f);},
                0x0A => {Cpu::rrc_r8(&mut self.reg_d, &mut self.reg_f);},
                0x0B => {Cpu::rrc_r8(&mut self.reg_e, &mut self.reg_f);},
                0x0C => {Cpu::rrc_r8(&mut self.reg_h, &mut self.reg_f);},
                0x0D => {Cpu::rrc_r8(&mut self.reg_l, &mut self.reg_f);},
                0x0E => {Cpu::rrc_r8(ram.get_rp_ref(self.reg_h, self.reg_l), &mut self.reg_f);},
                0x0F => {Cpu::rrc_r8(&mut self.reg_a, &mut self.reg_f);},
                0x10 => {Cpu::rl_r8(&mut self.reg_b, &mut self.reg_f);},
                0x11 => {Cpu::rl_r8(&mut self.reg_c, &mut self.reg_f);},
                0x12 => {Cpu::rl_r8(&mut self.reg_d, &mut self.reg_f);},
                0x13 => {Cpu::rl_r8(&mut self.reg_e, &mut self.reg_f);},
                0x14 => {Cpu::rl_r8(&mut self.reg_h, &mut self.reg_f);},
                0x15 => {Cpu::rl_r8(&mut self.reg_l, &mut self.reg_f);},
                0x16 => {Cpu::rl_r8(ram.get_rp_ref(self.reg_h, self.reg_l), &mut self.reg_f);},
                0x17 => {Cpu::rl_r8(&mut self.reg_a, &mut self.reg_f);},
                0x18 => {Cpu::rr_r8(&mut self.reg_b, &mut self.reg_f);},
                0x19 => {Cpu::rr_r8(&mut self.reg_c, &mut self.reg_f);},
                0x1A => {Cpu::rr_r8(&mut self.reg_d, &mut self.reg_f);},
                0x1B => {Cpu::rr_r8(&mut self.reg_e, &mut self.reg_f);},
                0x1C => {Cpu::rr_r8(&mut self.reg_h, &mut self.reg_f);},
                0x1D => {Cpu::rr_r8(&mut self.reg_l, &mut self.reg_f);},
                0x1E => {Cpu::rr_r8(ram.get_rp_ref(self.reg_h, self.reg_l), &mut self.reg_f);},
                0x1F => {Cpu::rr_r8(&mut self.reg_a, &mut self.reg_f);},
                0x20 => {Cpu::sla_r8(&mut self.reg_b, &mut self.reg_f);},
                0x21 => {Cpu::sla_r8(&mut self.reg_c, &mut self.reg_f);},
                0x22 => {Cpu::sla_r8(&mut self.reg_d, &mut self.reg_f);},
                0x23 => {Cpu::sla_r8(&mut self.reg_e, &mut self.reg_f);},
                0x24 => {Cpu::sla_r8(&mut self.reg_h, &mut self.reg_f);},
                0x25 => {Cpu::sla_r8(&mut self.reg_l, &mut self.reg_f);},
                0x26 => {Cpu::sla_r8(ram.get_rp_ref(self.reg_h, self.reg_l), &mut self.reg_f);},
                0x27 => {Cpu::sla_r8(&mut self.reg_a, &mut self.reg_f);},
                0x28 => {Cpu::sra_r8(&mut self.reg_b, &mut self.reg_f);},
                0x29 => {Cpu::sra_r8(&mut self.reg_c, &mut self.reg_f);},
                0x2A => {Cpu::sra_r8(&mut self.reg_d, &mut self.reg_f);},
                0x2B => {Cpu::sra_r8(&mut self.reg_e, &mut self.reg_f);},
                0x2C => {Cpu::sra_r8(&mut self.reg_h, &mut self.reg_f);},
                0x2D => {Cpu::sra_r8(&mut self.reg_l, &mut self.reg_f);},
                0x2E => {Cpu::sra_r8(ram.get_rp_ref(self.reg_h, self.reg_l), &mut self.reg_f);},
                0x2F => {Cpu::sra_r8(&mut self.reg_a, &mut self.reg_f);},
                0x30 => {Cpu::swap_r8(&mut self.reg_b);},
                0x31 => {Cpu::swap_r8(&mut self.reg_c);},
                0x32 => {Cpu::swap_r8(&mut self.reg_d);},
                0x33 => {Cpu::swap_r8(&mut self.reg_e);},
                0x34 => {Cpu::swap_r8(&mut self.reg_h);},
                0x35 => {Cpu::swap_r8(&mut self.reg_l);},
                0x36 => {Cpu::swap_r8(ram.get_rp_ref(self.reg_h, self.reg_l));},
                0x37 => {Cpu::swap_r8(&mut self.reg_a);},
                0x38 => {Cpu::srl_r8(&mut self.reg_b, &mut self.reg_f);},
                0x39 => {Cpu::srl_r8(&mut self.reg_c, &mut self.reg_f);},
                0x3A => {Cpu::srl_r8(&mut self.reg_d, &mut self.reg_f);},
                0x3B => {Cpu::srl_r8(&mut self.reg_e, &mut self.reg_f);},
                0x3C => {Cpu::srl_r8(&mut self.reg_h, &mut self.reg_f);},
                0x3D => {Cpu::srl_r8(&mut self.reg_l, &mut self.reg_f);},
                0x3E => {Cpu::srl_r8(ram.get_rp_ref(self.reg_h, self.reg_l), &mut self.reg_f);},
                0x3F => {Cpu::srl_r8(&mut self.reg_a, &mut self.reg_f);},
                0x40 => {Cpu::bit_r8(0, &mut self.reg_b, &mut self.reg_f);},
                0x41 => {Cpu::bit_r8(0, &mut self.reg_c, &mut self.reg_f);},
                0x42 => {Cpu::bit_r8(0, &mut self.reg_d, &mut self.reg_f);},
                0x43 => {Cpu::bit_r8(0, &mut self.reg_e, &mut self.reg_f);},
                0x44 => {Cpu::bit_r8(0, &mut self.reg_h, &mut self.reg_f);},
                0x45 => {Cpu::bit_r8(0, &mut self.reg_l, &mut self.reg_f);},
                0x46 => {Cpu::bit_r8(0, ram.get_rp_ref(self.reg_h, self.reg_l), &mut self.reg_f);},
                0x47 => {Cpu::bit_r8(0, &mut self.reg_a, &mut self.reg_f);},
                0x48 => {Cpu::bit_r8(1, &mut self.reg_b, &mut self.reg_f);},
                0x49 => {Cpu::bit_r8(1, &mut self.reg_c, &mut self.reg_f);},
                0x4A => {Cpu::bit_r8(1, &mut self.reg_d, &mut self.reg_f);},
                0x4B => {Cpu::bit_r8(1, &mut self.reg_e, &mut self.reg_f);},
                0x4C => {Cpu::bit_r8(1, &mut self.reg_h, &mut self.reg_f);},
                0x4D => {Cpu::bit_r8(1, &mut self.reg_l, &mut self.reg_f);},
                0x4E => {Cpu::bit_r8(1, ram.get_rp_ref(self.reg_h, self.reg_l), &mut self.reg_f);},
                0x4F => {Cpu::bit_r8(1, &mut self.reg_a, &mut self.reg_f);},
                0x50 => {Cpu::bit_r8(2, &mut self.reg_b, &mut self.reg_f);},
                0x51 => {Cpu::bit_r8(2, &mut self.reg_c, &mut self.reg_f);},
                0x52 => {Cpu::bit_r8(2, &mut self.reg_d, &mut self.reg_f);},
                0x53 => {Cpu::bit_r8(2, &mut self.reg_e, &mut self.reg_f);},
                0x54 => {Cpu::bit_r8(2, &mut self.reg_h, &mut self.reg_f);},
                0x55 => {Cpu::bit_r8(2, &mut self.reg_l, &mut self.reg_f);},
                0x56 => {Cpu::bit_r8(2, ram.get_rp_ref(self.reg_h, self.reg_l), &mut self.reg_f);},
                0x57 => {Cpu::bit_r8(2, &mut self.reg_a, &mut self.reg_f);},
                0x58 => {Cpu::bit_r8(3, &mut self.reg_b, &mut self.reg_f);},
                0x59 => {Cpu::bit_r8(3, &mut self.reg_c, &mut self.reg_f);},
                0x5A => {Cpu::bit_r8(3, &mut self.reg_d, &mut self.reg_f);},
                0x5B => {Cpu::bit_r8(3, &mut self.reg_e, &mut self.reg_f);},
                0x5C => {Cpu::bit_r8(3, &mut self.reg_h, &mut self.reg_f);},
                0x5D => {Cpu::bit_r8(3, &mut self.reg_l, &mut self.reg_f);},
                0x5E => {Cpu::bit_r8(3, ram.get_rp_ref(self.reg_h, self.reg_l), &mut self.reg_f);},
                0x5F => {Cpu::bit_r8(3, &mut self.reg_a, &mut self.reg_f);},
                0x60 => {Cpu::bit_r8(4, &mut self.reg_b, &mut self.reg_f);},
                0x61 => {Cpu::bit_r8(4, &mut self.reg_c, &mut self.reg_f);},
                0x62 => {Cpu::bit_r8(4, &mut self.reg_d, &mut self.reg_f);},
                0x63 => {Cpu::bit_r8(4, &mut self.reg_e, &mut self.reg_f);},
                0x64 => {Cpu::bit_r8(4, &mut self.reg_h, &mut self.reg_f);},
                0x65 => {Cpu::bit_r8(4, &mut self.reg_l, &mut self.reg_f);},
                0x66 => {Cpu::bit_r8(4, ram.get_rp_ref(self.reg_h, self.reg_l), &mut self.reg_f);},
                0x67 => {Cpu::bit_r8(4, &mut self.reg_a, &mut self.reg_f);},
                0x68 => {Cpu::bit_r8(5, &mut self.reg_b, &mut self.reg_f);},
                0x69 => {Cpu::bit_r8(5, &mut self.reg_c, &mut self.reg_f);},
                0x6A => {Cpu::bit_r8(5, &mut self.reg_d, &mut self.reg_f);},
                0x6B => {Cpu::bit_r8(5, &mut self.reg_e, &mut self.reg_f);},
                0x6C => {Cpu::bit_r8(5, &mut self.reg_h, &mut self.reg_f);},
                0x6D => {Cpu::bit_r8(5, &mut self.reg_l, &mut self.reg_f);},
                0x6E => {Cpu::bit_r8(5, ram.get_rp_ref(self.reg_h, self.reg_l), &mut self.reg_f);},
                0x6F => {Cpu::bit_r8(5, &mut self.reg_a, &mut self.reg_f);},
                0x70 => {Cpu::bit_r8(6, &mut self.reg_b, &mut self.reg_f);},
                0x71 => {Cpu::bit_r8(6, &mut self.reg_c, &mut self.reg_f);},
                0x72 => {Cpu::bit_r8(6, &mut self.reg_d, &mut self.reg_f);},
                0x73 => {Cpu::bit_r8(6, &mut self.reg_e, &mut self.reg_f);},
                0x74 => {Cpu::bit_r8(6, &mut self.reg_h, &mut self.reg_f);},
                0x75 => {Cpu::bit_r8(6, &mut self.reg_l, &mut self.reg_f);},
                0x76 => {Cpu::bit_r8(6, ram.get_rp_ref(self.reg_h, self.reg_l), &mut self.reg_f);},
                0x77 => {Cpu::bit_r8(6, &mut self.reg_a, &mut self.reg_f);},
                0x78 => {Cpu::bit_r8(7, &mut self.reg_b, &mut self.reg_f);},
                0x79 => {Cpu::bit_r8(7, &mut self.reg_c, &mut self.reg_f);},
                0x7A => {Cpu::bit_r8(7, &mut self.reg_d, &mut self.reg_f);},
                0x7B => {Cpu::bit_r8(7, &mut self.reg_e, &mut self.reg_f);},
                0x7C => {Cpu::bit_r8(7, &mut self.reg_h, &mut self.reg_f);},
                0x7D => {Cpu::bit_r8(7, &mut self.reg_l, &mut self.reg_f);},
                0x7E => {Cpu::bit_r8(7, ram.get_rp_ref(self.reg_h, self.reg_l), &mut self.reg_f);},
                0x7F => {Cpu::bit_r8(7, &mut self.reg_a, &mut self.reg_f);},
                0x80 => {Cpu::res_r8(0, &mut self.reg_b);},
                0x81 => {Cpu::res_r8(0, &mut self.reg_c);},
                0x82 => {Cpu::res_r8(0, &mut self.reg_d);},
                0x83 => {Cpu::res_r8(0, &mut self.reg_e);},
                0x84 => {Cpu::res_r8(0, &mut self.reg_h);},
                0x85 => {Cpu::res_r8(0, &mut self.reg_l);},
                0x86 => {Cpu::res_r8(0, ram.get_rp_ref(self.reg_h, self.reg_l));},
                0x87 => {Cpu::res_r8(0, &mut self.reg_a);},
                0x88 => {Cpu::res_r8(1, &mut self.reg_b);},
                0x89 => {Cpu::res_r8(1, &mut self.reg_c);},
                0x8A => {Cpu::res_r8(1, &mut self.reg_d);},
                0x8B => {Cpu::res_r8(1, &mut self.reg_e);},
                0x8C => {Cpu::res_r8(1, &mut self.reg_h);},
                0x8D => {Cpu::res_r8(1, &mut self.reg_l);},
                0x8E => {Cpu::res_r8(1, ram.get_rp_ref(self.reg_h, self.reg_l));},
                0x8F => {Cpu::res_r8(1, &mut self.reg_a);},
                0x90 => {Cpu::res_r8(2, &mut self.reg_b);},
                0x91 => {Cpu::res_r8(2, &mut self.reg_c);},
                0x92 => {Cpu::res_r8(2, &mut self.reg_d);},
                0x93 => {Cpu::res_r8(2, &mut self.reg_e);},
                0x94 => {Cpu::res_r8(2, &mut self.reg_h);},
                0x95 => {Cpu::res_r8(2, &mut self.reg_l);},
                0x96 => {Cpu::res_r8(2, ram.get_rp_ref(self.reg_h, self.reg_l));},
                0x97 => {Cpu::res_r8(2, &mut self.reg_a);},
                0x98 => {Cpu::res_r8(3, &mut self.reg_b);},
                0x99 => {Cpu::res_r8(3, &mut self.reg_c);},
                0x9A => {Cpu::res_r8(3, &mut self.reg_d);},
                0x9B => {Cpu::res_r8(3, &mut self.reg_e);},
                0x9C => {Cpu::res_r8(3, &mut self.reg_h);},
                0x9D => {Cpu::res_r8(3, &mut self.reg_l);},
                0x9E => {Cpu::res_r8(3, ram.get_rp_ref(self.reg_h, self.reg_l));},
                0x9F => {Cpu::res_r8(3, &mut self.reg_a);},
                0xA0 => {Cpu::res_r8(4, &mut self.reg_b);},
                0xA1 => {Cpu::res_r8(4, &mut self.reg_c);},
                0xA2 => {Cpu::res_r8(4, &mut self.reg_d);},
                0xA3 => {Cpu::res_r8(4, &mut self.reg_e);},
                0xA4 => {Cpu::res_r8(4, &mut self.reg_h);},
                0xA5 => {Cpu::res_r8(4, &mut self.reg_l);},
                0xA6 => {Cpu::res_r8(4, ram.get_rp_ref(self.reg_h, self.reg_l));},
                0xA7 => {Cpu::res_r8(4, &mut self.reg_a);},
                0xA8 => {Cpu::res_r8(5, &mut self.reg_b);},
                0xA9 => {Cpu::res_r8(5, &mut self.reg_c);},
                0xAA => {Cpu::res_r8(5, &mut self.reg_d);},
                0xAB => {Cpu::res_r8(5, &mut self.reg_e);},
                0xAC => {Cpu::res_r8(5, &mut self.reg_h);},
                0xAD => {Cpu::res_r8(5, &mut self.reg_l);},
                0xAE => {Cpu::res_r8(5, ram.get_rp_ref(self.reg_h, self.reg_l));},
                0xAF => {Cpu::res_r8(5, &mut self.reg_a);},
                0xB0 => {Cpu::res_r8(6, &mut self.reg_b);},
                0xB1 => {Cpu::res_r8(6, &mut self.reg_c);},
                0xB2 => {Cpu::res_r8(6, &mut self.reg_d);},
                0xB3 => {Cpu::res_r8(6, &mut self.reg_e);},
                0xB4 => {Cpu::res_r8(6, &mut self.reg_h);},
                0xB5 => {Cpu::res_r8(6, &mut self.reg_l);},
                0xB6 => {Cpu::res_r8(6, ram.get_rp_ref(self.reg_h, self.reg_l));},
                0xB7 => {Cpu::res_r8(6, &mut self.reg_a);},
                0xB8 => {Cpu::res_r8(7, &mut self.reg_b);},
                0xB9 => {Cpu::res_r8(7, &mut self.reg_c);},
                0xBA => {Cpu::res_r8(7, &mut self.reg_d);},
                0xBB => {Cpu::res_r8(7, &mut self.reg_e);},
                0xBC => {Cpu::res_r8(7, &mut self.reg_h);},
                0xBD => {Cpu::res_r8(7, &mut self.reg_l);},
                0xBE => {Cpu::res_r8(7, ram.get_rp_ref(self.reg_h, self.reg_l));},
                0xBF => {Cpu::res_r8(7, &mut self.reg_a);},
                0xC0 => {Cpu::set_r8(0, &mut self.reg_b);},
                0xC1 => {Cpu::set_r8(0, &mut self.reg_c);},
                0xC2 => {Cpu::set_r8(0, &mut self.reg_d);},
                0xC3 => {Cpu::set_r8(0, &mut self.reg_e);},
                0xC4 => {Cpu::set_r8(0, &mut self.reg_h);},
                0xC5 => {Cpu::set_r8(0, &mut self.reg_l);},
                0xC6 => {Cpu::set_r8(0, ram.get_rp_ref(self.reg_h, self.reg_l));},
                0xC7 => {Cpu::set_r8(0, &mut self.reg_a);},
                0xC8 => {Cpu::set_r8(1, &mut self.reg_b);},
                0xC9 => {Cpu::set_r8(1, &mut self.reg_c);},
                0xCA => {Cpu::set_r8(1, &mut self.reg_d);},
                0xCB => {Cpu::set_r8(1, &mut self.reg_e);},
                0xCC => {Cpu::set_r8(1, &mut self.reg_h);},
                0xCD => {Cpu::set_r8(1, &mut self.reg_l);},
                0xCE => {Cpu::set_r8(1, ram.get_rp_ref(self.reg_h, self.reg_l));},
                0xCF => {Cpu::set_r8(1, &mut self.reg_a);},
                0xD0 => {Cpu::set_r8(2, &mut self.reg_b);},
                0xD1 => {Cpu::set_r8(2, &mut self.reg_c);},
                0xD2 => {Cpu::set_r8(2, &mut self.reg_d);},
                0xD3 => {Cpu::set_r8(2, &mut self.reg_e);},
                0xD4 => {Cpu::set_r8(2, &mut self.reg_h);},
                0xD5 => {Cpu::set_r8(2, &mut self.reg_l);},
                0xD6 => {Cpu::set_r8(2, ram.get_rp_ref(self.reg_h, self.reg_l));},
                0xD7 => {Cpu::set_r8(2, &mut self.reg_a);},
                0xD8 => {Cpu::set_r8(3, &mut self.reg_b);},
                0xD9 => {Cpu::set_r8(3, &mut self.reg_c);},
                0xDA => {Cpu::set_r8(3, &mut self.reg_d);},
                0xDB => {Cpu::set_r8(3, &mut self.reg_e);},
                0xDC => {Cpu::set_r8(3, &mut self.reg_h);},
                0xDD => {Cpu::set_r8(3, &mut self.reg_l);},
                0xDE => {Cpu::set_r8(3, ram.get_rp_ref(self.reg_h, self.reg_l));},
                0xDF => {Cpu::set_r8(3, &mut self.reg_a);},
                0xE0 => {Cpu::set_r8(4, &mut self.reg_b);},
                0xE1 => {Cpu::set_r8(4, &mut self.reg_c);},
                0xE2 => {Cpu::set_r8(4, &mut self.reg_d);},
                0xE3 => {Cpu::set_r8(4, &mut self.reg_e);},
                0xE4 => {Cpu::set_r8(4, &mut self.reg_h);},
                0xE5 => {Cpu::set_r8(4, &mut self.reg_l);},
                0xE6 => {Cpu::set_r8(4, ram.get_rp_ref(self.reg_h, self.reg_l));},
                0xE7 => {Cpu::set_r8(4, &mut self.reg_a);},
                0xE8 => {Cpu::set_r8(5, &mut self.reg_b);},
                0xE9 => {Cpu::set_r8(5, &mut self.reg_c);},
                0xEA => {Cpu::set_r8(5, &mut self.reg_d);},
                0xEB => {Cpu::set_r8(5, &mut self.reg_e);},
                0xEC => {Cpu::set_r8(5, &mut self.reg_h);},
                0xED => {Cpu::set_r8(5, &mut self.reg_l);},
                0xEE => {Cpu::set_r8(5, ram.get_rp_ref(self.reg_h, self.reg_l));},
                0xEF => {Cpu::set_r8(5, &mut self.reg_a);},
                0xF0 => {Cpu::set_r8(6, &mut self.reg_b);},
                0xF1 => {Cpu::set_r8(6, &mut self.reg_c);},
                0xF2 => {Cpu::set_r8(6, &mut self.reg_d);},
                0xF3 => {Cpu::set_r8(6, &mut self.reg_e);},
                0xF4 => {Cpu::set_r8(6, &mut self.reg_h);},
                0xF5 => {Cpu::set_r8(6, &mut self.reg_l);},
                0xF6 => {Cpu::set_r8(6, ram.get_rp_ref(self.reg_h, self.reg_l));},
                0xF7 => {Cpu::set_r8(6, &mut self.reg_a);},
                0xF8 => {Cpu::set_r8(7, &mut self.reg_b);},
                0xF9 => {Cpu::set_r8(7, &mut self.reg_c);},
                0xFA => {Cpu::set_r8(7, &mut self.reg_d);},
                0xFB => {Cpu::set_r8(7, &mut self.reg_e);},
                0xFC => {Cpu::set_r8(7, &mut self.reg_h);},
                0xFD => {Cpu::set_r8(7, &mut self.reg_l);},
                0xFE => {Cpu::set_r8(7, ram.get_rp_ref(self.reg_h, self.reg_l));},
                0xFF => {Cpu::set_r8(7, &mut self.reg_a);}
            }

            self.pc.current_instruction_cycles += CB_INSTRUCTION_TIME_TABLE[cb_instruction as usize];
        }

        self.aux_inc_pc();
    }

    fn invalid_instruction(&self, opcode: u8)
    {
        panic!("Tried to execute invalid instruction {:X?}", opcode);
    }

    // regs: [u8;8],
    // sp: u16,
    // ram: Ram,
    // pc: ProgramCounter
    // #[allow(dead_code)]
    // fn aux_get_reg(&self, regnum: usize) -> u8 { regnum }
    // #[allow(dead_code)]
    // fn aux_get_sp(&self) -> u16 { self.sp }
    // #[allow(dead_code)]
    // fn aux_get_pc(&self) -> ProgramCounter { self.pc }

}

impl Default for Cpu
{
    fn default() -> Self { Self::new() }
}