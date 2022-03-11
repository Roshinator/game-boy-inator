mod ld_tests;
mod control_tests;
mod alu_tests;
mod jump_branch_tests;
use crate::ram::{self, Ram};

type Reg = usize;
const REG_A:Reg = 0;
const REG_B:Reg = 1;
const REG_C:Reg = 2;
const REG_D:Reg = 3;
const REG_E:Reg = 4;
const REG_F:Reg = 5;
const REG_H:Reg = 6;
const REG_L:Reg = 7;

//in an AF situation, A is msh, F is lsh, little endian

bitflags::bitflags! 
{
    pub struct Flag: u8
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
    regs: [u8;8],
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
            regs: [0x01, 0x00, 0x13, 0x00, 0xD8, 0xB0, 0x01, 0x4D],
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

    fn aux_read_flag(&self, param: Flag) -> bool
    {
        (self.regs[REG_F] & param.bits) != 0
    }

    fn aux_write_flag(&mut self, param: Flag, data: bool)
    {
        if data
        {
            self.regs[REG_F] = self.regs[REG_F] | (param.bits);
        }
        else
        {
            self.regs[REG_F] = self.regs[REG_F] & (!param.bits);
        }
    }

    fn halt(&mut self)
    {
        self.halted = true;
    }

    fn stop(&mut self)
    {
        self.halted = true;
        self.stopped = true;
    }

    fn ld_r16_16(&mut self, msh_reg: Reg, lsh_reg: Reg, msh_num: u8, lsh_num: u8)
    {
        self.regs[msh_reg] = msh_num;
        self.regs[lsh_reg] = lsh_num;
    }

    fn ld_hl_sp_plus(&mut self, p1: i8)
    {
        let conv = p1.unsigned_abs() as u16;
        let negative = p1 < 0;
        if negative
        {
            let bytes = u16::to_le_bytes(self.sp - conv);
            self.regs[REG_H] = bytes[1];
            self.regs[REG_L] = bytes[0];
        }
        else
        {
            let bytes = u16::to_le_bytes(self.sp + conv);
            self.regs[REG_H] = bytes[1];
            self.regs[REG_L] = bytes[0];
        }
    }

    fn ld_sp_16(&mut self, msh_num: u8, lsh_num: u8)
    {
        self.sp = u16::from_le_bytes([lsh_num, msh_num]);
    }

    fn ld_r8_8(&mut self, p1: Reg, p2: u8)
    {
        self.regs[p1] = p2;
    }

    fn ld_r16a_8(&mut self, ram: &mut Ram, msh: Reg, lsh: Reg, p2: u8)
    {
        ram.write_rp(self.regs[msh], self.regs[lsh], p2);
    }

    fn ld_16a_r8(&mut self, ram: &mut Ram, msh: u8, lsh: u8, p2: Reg)
    {
        ram.write_rp(msh, lsh, self.regs[p2]);
    }

    fn ld_16a_sp(&mut self, ram: &mut Ram, msh: u8, lsh: u8)
    {
        let bytes = self.sp.to_le_bytes();
        ram.write_rp(msh, lsh, bytes[0]);

        let result = self.aux_inc_16(msh, lsh);
        ram.write_rp(result.1, result.0, bytes[1]);
    }

    fn ld_r8_r8(&mut self, p1: Reg, p2: Reg)
    {
        self.regs[p1] = self.regs[p2];
    }

    fn ld_sp_r16(&mut self, msh: Reg, lsh: Reg)
    {
        self.sp = u16::from_le_bytes([self.regs[lsh], self.regs[msh]]);
    }

    fn ld_r8_r16a(&mut self, ram: &mut Ram, p1: Reg, msh: Reg, lsh: Reg)
    {
        let x = ram.read_rp(self.regs[msh], self.regs[lsh]);
        self.regs[p1] = x;
    }

    fn ld_r8_16a(&mut self, ram: &mut Ram, p1: Reg, msh: u8, lsh: u8)
    {
        let x = ram.read_rp(msh, lsh);
        self.regs[p1] = x;
    }

    fn ld_r16a_r8(&mut self, ram: &mut Ram, msh: Reg, lsh: Reg, p2: Reg)
    {
        ram.write_rp(self.regs[msh], self.regs[lsh], self.regs[p2]);
    }

    // TODO: See if the flags are modified

    ///Returns (lsh, msh)
    fn aux_inc_16(&mut self, msh: u8, lsh: u8) -> (u8, u8)
    {
        let lsh_result = u8::overflowing_add(lsh, 1);
        let msh_result = u8::overflowing_add(msh, lsh_result.1 as u8);
        (lsh_result.0, msh_result.0)
    }

    fn inc_r16(&mut self, msh: Reg, lsh: Reg)
    {
        let lsh_result = u8::overflowing_add(self.regs[lsh], 1);
        self.regs[lsh] = lsh_result.0;
        let msh_result = u8::overflowing_add(self.regs[msh], lsh_result.1 as u8);
        self.regs[msh] = msh_result.0;
    }

    fn inc_sp(&mut self)
    {
        let result = u16::overflowing_add(self.sp, 1);
        self.sp = result.0;
    }

    fn inc_r8(&mut self, reg: Reg)
    {
        let result = u8::overflowing_add(self.regs[reg], 1);
        self.regs[reg] = result.0;

        self.aux_write_flag(Flag::FLAG_Z, result.0 == 0);
        self.aux_write_flag(Flag::FLAG_N, false);
        self.aux_write_flag(Flag::FLAG_H, result.1);
    }

    fn inc_r16a(&mut self, ram: &mut Ram, msh: Reg, lsh: Reg)
    {
        let result = ram.read_rp(
            self.regs[msh], self.regs[lsh]).overflowing_add(1);
        ram.write_rp(self.regs[msh], self.regs[lsh], result.0);

        self.aux_write_flag(Flag::FLAG_Z, result.0 == 0);
        self.aux_write_flag(Flag::FLAG_N, false);
        self.aux_write_flag(Flag::FLAG_H, result.1);
    }

    fn dec_r8(&mut self, reg: Reg)
    {
        let result = u8::overflowing_sub(self.regs[reg], 1);
        self.regs[reg] = result.0;

        self.aux_write_flag(Flag::FLAG_Z, result.0 == 0);
        self.aux_write_flag(Flag::FLAG_N, true);
        self.aux_write_flag(Flag::FLAG_H, result.1);
    }

    fn dec_r16a(&mut self, ram: &mut Ram, msh: Reg, lsh: Reg)
    {
        let result = ram.read_rp(
            self.regs[msh], self.regs[lsh]).overflowing_sub(1);
        ram.write_rp(self.regs[msh], self.regs[lsh], result.0);

        self.aux_write_flag(Flag::FLAG_Z, result.0 == 0);
        self.aux_write_flag(Flag::FLAG_N, true);
        self.aux_write_flag(Flag::FLAG_H, result.1);
    }

    fn dec_r16(&mut self, msh: Reg, lsh: Reg)
    {
        let lsh_result = u8::overflowing_sub(self.regs[lsh], 1);
        self.regs[lsh] = lsh_result.0;
        let msh_result = u8::overflowing_sub(self.regs[msh], lsh_result.1 as u8);
        self.regs[msh] = msh_result.0;
    }

    fn dec_sp(&mut self)
    {
        let result = u16::overflowing_sub(self.sp, 1);
        self.sp = result.0;
    }

    fn add_r8_r8(&mut self, p1: Reg, p2: Reg)
    {
        let half_carry_pre = ((self.regs[p1] ^ self.regs[p2]) >> 4) & 1;
        let result = self.regs[p1].overflowing_add(self.regs[p2]);
        self.regs[p1] = result.0;
        let half_carry_post = (result.0 >> 4) & 1;

        self.aux_write_flag(Flag::FLAG_Z, result.0 == 0);
        self.aux_write_flag(Flag::FLAG_H, half_carry_pre != half_carry_post);
        self.aux_write_flag(Flag::FLAG_N, false);
        self.aux_write_flag(Flag::FLAG_C, result.1);
    }

    fn add_r8_8(&mut self, p1: Reg, p2: u8)
    {
        let half_carry_pre = ((self.regs[p1] ^ p2) >> 4) & 1;
        let result = self.regs[p1].overflowing_add(p2);
        self.regs[p1] = result.0;
        let half_carry_post = (result.0 >> 4) & 1;

        self.aux_write_flag(Flag::FLAG_Z, result.0 == 0);
        self.aux_write_flag(Flag::FLAG_H, half_carry_pre != half_carry_post);
        self.aux_write_flag(Flag::FLAG_N, false);
        self.aux_write_flag(Flag::FLAG_C, result.1);
    }

    fn add_r16_r16(&mut self, p1_msh: Reg, p1_lsh: Reg, p2_msh: Reg, p2_lsh: Reg)
    {
        let z = self.aux_read_flag(Flag::FLAG_Z);
        self.add_r8_r8(p1_lsh, p2_lsh);
        self.adc_r8_r8(p1_msh, p2_msh);

        self.aux_write_flag(Flag::FLAG_Z, z);
        self.aux_write_flag(Flag::FLAG_N, false);
    }

    fn add_r16_sp(&mut self, p1_msh: Reg, p1_lsh: Reg)
    {
        let z = self.aux_read_flag(Flag::FLAG_Z);
        let reg = self.sp.to_le_bytes();

        //ADD
        let result = self.regs[p1_lsh].overflowing_add(reg[0]);
        self.regs[p1_lsh] = result.0;
        self.aux_write_flag(Flag::FLAG_C, result.1);

        //ADC
        let carry = self.aux_read_flag(Flag::FLAG_C) as u8;
        let half_carry_pre1 = ((self.regs[p1_msh] ^ reg[1]) >> 4) & 1;
        let result1 = self.regs[p1_msh].overflowing_add(reg[1]);
        let half_carry_post1 = (result1.0 >> 4) & 1;
        let half_carry_pre2 = ((result1.0 ^ carry) >> 4) & 1;
        let result2 = result1.0.overflowing_add(carry);
        self.regs[p1_msh] = result2.0;
        let half_carry_post2 = (result2.0 >> 4) & 1;

        self.aux_write_flag(Flag::FLAG_H, half_carry_pre1 != half_carry_post1 || half_carry_pre2 != half_carry_post2);
        self.aux_write_flag(Flag::FLAG_N, false);
        self.aux_write_flag(Flag::FLAG_C, result1.1 || result2.1);
        self.aux_write_flag(Flag::FLAG_Z, z);
    }

    fn add_r8_r16a(&mut self, ram: &mut Ram, p1: Reg, msh: Reg, lsh: Reg)
    {
        let p2 = ram.read_rp(self.regs[msh], self.regs[lsh]);
        let half_carry_pre = ((self.regs[p1] ^ p2) >> 4) & 1;
        let result = self.regs[p1].overflowing_add(p2);
        self.regs[p1] = result.0;
        let half_carry_post = (result.0 >> 4) & 1;

        self.aux_write_flag(Flag::FLAG_Z, result.0 == 0);
        self.aux_write_flag(Flag::FLAG_H, half_carry_pre == half_carry_post);
        self.aux_write_flag(Flag::FLAG_N, false);
        self.aux_write_flag(Flag::FLAG_C, result.1);
    }

    fn add_sp_i8(&mut self, p1: i8)
    {
        let conv = p1.unsigned_abs() as u16;
        let negative = p1 < 0;
        if negative
        {
            self.sp -= conv;
        }
        else
        {
            self.sp += conv;
        }
    }

    fn adc_r8_r8(&mut self, p1: Reg, p2: Reg)
    {
        let carry = self.aux_read_flag(Flag::FLAG_C) as u8;
        let half_carry_pre1 = ((self.regs[p1] ^ self.regs[p2]) >> 4) & 1;
        let result1 = self.regs[p1].overflowing_add(self.regs[p2]);
        let half_carry_post1 = (result1.0 >> 4) & 1;
        let half_carry_pre2 = ((result1.0 ^ carry) >> 4) & 1;
        let result2 = result1.0.overflowing_add(carry);
        self.regs[p1] = result2.0;
        let half_carry_post2 = (result2.0 >> 4) & 1;

        self.aux_write_flag(Flag::FLAG_Z, result2.0 == 0);
        self.aux_write_flag(Flag::FLAG_H, half_carry_pre1 != half_carry_post1 || half_carry_pre2 != half_carry_post2);
        self.aux_write_flag(Flag::FLAG_N, false);
        self.aux_write_flag(Flag::FLAG_C, result1.1 || result2.1);
    }

    fn adc_r8_8(&mut self, p1: Reg, p2: u8)
    {
        let carry = self.aux_read_flag(Flag::FLAG_C) as u8;
        let half_carry_pre1 = ((self.regs[p1] ^ p2) >> 4) & 1;
        let result1 = self.regs[p1].overflowing_add(p2);
        let half_carry_post1 = (result1.0 >> 4) & 1;
        let half_carry_pre2 = ((result1.0 ^ carry) >> 4) & 1;
        let result2 = result1.0.overflowing_add(carry);
        self.regs[p1] = result2.0;
        let half_carry_post2 = (result2.0 >> 4) & 1;

        self.aux_write_flag(Flag::FLAG_Z, result2.0 == 0);
        self.aux_write_flag(Flag::FLAG_H, half_carry_pre1 != half_carry_post1 || half_carry_pre2 != half_carry_post2);
        self.aux_write_flag(Flag::FLAG_N, false);
        self.aux_write_flag(Flag::FLAG_C, result1.1 || result2.1);
    }

    fn adc_r8_r16a(&mut self, ram: &mut Ram, p1: Reg, msh: Reg, lsh: Reg)
    {
        let carry = self.aux_read_flag(Flag::FLAG_C) as u8;
        let p2 = ram.read_rp(self.regs[msh], self.regs[lsh]);
        let half_carry_pre1 = ((self.regs[p1] ^ p2) >> 4) & 1;
        let result1 = self.regs[p1].overflowing_add(p2);
        let half_carry_post1 = (result1.0 >> 4) & 1;
        let half_carry_pre2 = ((result1.0 ^ carry) >> 4) & 1;
        let result2 = result1.0.overflowing_add(carry);
        self.regs[p1] = result2.0;
        let half_carry_post2 = (result2.0 >> 4) & 1;

        self.aux_write_flag(Flag::FLAG_Z, result2.0 == 0);
        self.aux_write_flag(Flag::FLAG_H, half_carry_pre1 != half_carry_post1 || half_carry_pre2 != half_carry_post2);
        self.aux_write_flag(Flag::FLAG_N, false);
        self.aux_write_flag(Flag::FLAG_C, result1.1 || result2.1);
    }

    //TODO: Check subtraction half carry calculations
    fn sub_r8_r8(&mut self, p1: Reg, p2: Reg)
    {
        let half_carry_pre = ((self.regs[p1] ^ self.regs[p2]) >> 4) & 1;
        let result = self.regs[p1].overflowing_sub(self.regs[p2]);
        self.regs[p1] = result.0;
        let half_carry_post = (result.0 >> 4) & 1;

        self.aux_write_flag(Flag::FLAG_Z, result.0 == 0);
        self.aux_write_flag(Flag::FLAG_H, half_carry_pre != half_carry_post);
        self.aux_write_flag(Flag::FLAG_N, true);
        self.aux_write_flag(Flag::FLAG_C, result.1);
    }

    fn sub_r8_8(&mut self, p1: Reg, p2: u8)
    {
        let half_carry_pre = ((self.regs[p1] ^ p2) >> 4) & 1;
        let result = self.regs[p1].overflowing_sub(p2);
        self.regs[p1] = result.0;
        let half_carry_post = (result.0 >> 4) & 1;

        self.aux_write_flag(Flag::FLAG_Z, result.0 == 0);
        self.aux_write_flag(Flag::FLAG_H, half_carry_pre != half_carry_post);
        self.aux_write_flag(Flag::FLAG_N, true);
        self.aux_write_flag(Flag::FLAG_C, result.1);
    }

    fn sub_r8_r16a(&mut self, ram: &mut Ram, p1: Reg, msh: Reg, lsh: Reg)
    {
        let p2 = ram.read_rp(self.regs[msh], self.regs[lsh]);
        let half_carry_pre = ((self.regs[p1] ^ p2) >> 4) & 1;
        let result = self.regs[p1].overflowing_add(p2);
        self.regs[p1] = result.0;
        let half_carry_post = (result.0 >> 4) & 1;

        self.aux_write_flag(Flag::FLAG_Z, result.0 == 0);
        self.aux_write_flag(Flag::FLAG_H, half_carry_pre != half_carry_post);
        self.aux_write_flag(Flag::FLAG_N, true);
        self.aux_write_flag(Flag::FLAG_C, result.1);
    }

    fn sbc_r8_r8(&mut self, p1: Reg, p2: Reg)
    {
        let carry = self.aux_read_flag(Flag::FLAG_C) as u8;
        let half_carry_pre1 = ((self.regs[p1] ^ self.regs[p2]) >> 4) & 1;
        let result1 = self.regs[p1].overflowing_sub(self.regs[p2]);
        let half_carry_post1 = (result1.0 >> 4) & 1;
        let half_carry_pre2 = ((result1.0 ^ carry) >> 4) & 1;
        let result2 = result1.0.overflowing_sub(carry);
        self.regs[p1] = result2.0;
        let half_carry_post2 = (result2.0 >> 4) & 1;

        self.aux_write_flag(Flag::FLAG_Z, result2.0 == 0);
        self.aux_write_flag(Flag::FLAG_H, half_carry_pre1 != half_carry_post1 || half_carry_pre2 != half_carry_post2);
        self.aux_write_flag(Flag::FLAG_N, true);
        self.aux_write_flag(Flag::FLAG_C, result1.1 || result2.1);
    }

    fn sbc_r8_8(&mut self, p1: Reg, p2: u8)
    {
        let carry = self.aux_read_flag(Flag::FLAG_C) as u8;
        let half_carry_pre1 = ((self.regs[p1] ^ p2) >> 4) & 1;
        let result1 = self.regs[p1].overflowing_sub(p2);
        let half_carry_post1 = (result1.0 >> 4) & 1;
        let half_carry_pre2 = ((result1.0 ^ carry) >> 4) & 1;
        let result2 = result1.0.overflowing_sub(carry);
        self.regs[p1] = result2.0;
        let half_carry_post2 = (result2.0 >> 4) & 1;

        self.aux_write_flag(Flag::FLAG_Z, result2.0 == 0);
        self.aux_write_flag(Flag::FLAG_H, half_carry_pre1 != half_carry_post1 || half_carry_pre2 != half_carry_post2);
        self.aux_write_flag(Flag::FLAG_N, true);
        self.aux_write_flag(Flag::FLAG_C, result1.1 || result2.1);
    }

    fn sbc_r8_r16a(&mut self, ram: &mut Ram, p1: Reg, msh: Reg, lsh: Reg)
    {
        let carry = self.aux_read_flag(Flag::FLAG_C) as u8;
        let p2 = ram.read_rp(self.regs[msh], self.regs[lsh]);
        let half_carry_pre1 = ((self.regs[p1] ^ p2) >> 4) & 1;
        let result1 = self.regs[p1].overflowing_sub(p2);
        let half_carry_post1 = (result1.0 >> 4) & 1;
        let half_carry_pre2 = ((result1.0 ^ carry) >> 4) & 1;
        let result2 = result1.0.overflowing_sub(carry);
        self.regs[p1] = result2.0;
        let half_carry_post2 = (result2.0 >> 4) & 1;

        self.aux_write_flag(Flag::FLAG_Z, result2.0 == 0);
        self.aux_write_flag(Flag::FLAG_H, half_carry_pre1 != half_carry_post1 || half_carry_pre2 != half_carry_post2);
        self.aux_write_flag(Flag::FLAG_N, true);
        self.aux_write_flag(Flag::FLAG_C, result1.1 || result2.1);
    }

    fn and_r8_r8(&mut self, p1: Reg, p2: Reg)
    {
        self.regs[p1] &= self.regs[p2];

        self.aux_write_flag(Flag::FLAG_Z, self.regs[p1] == 0);
        self.aux_write_flag(Flag::FLAG_H, true);
        self.aux_write_flag(Flag::FLAG_N, false);
        self.aux_write_flag(Flag::FLAG_C, false);
    }

    fn and_r8_8(&mut self, p1: Reg, p2: u8)
    {
        self.regs[p1] &= p2;

        self.aux_write_flag(Flag::FLAG_Z, self.regs[p1] == 0);
        self.aux_write_flag(Flag::FLAG_H, true);
        self.aux_write_flag(Flag::FLAG_N, false);
        self.aux_write_flag(Flag::FLAG_C, false);
    }

    fn and_r8_r16a(&mut self, ram: &mut Ram, p1: Reg, msh: Reg, lsh: Reg)
    {
        self.regs[p1] &= ram.read_rp(self.regs[msh], self.regs[lsh]);

        self.aux_write_flag(Flag::FLAG_Z, self.regs[p1] == 0);
        self.aux_write_flag(Flag::FLAG_H, true);
        self.aux_write_flag(Flag::FLAG_N, false);
        self.aux_write_flag(Flag::FLAG_C, false);
    }

    fn xor_r8_r8(&mut self, p1: Reg, p2: Reg)
    {
        self.regs[p1] ^= self.regs[p2];

        self.aux_write_flag(Flag::FLAG_Z, self.regs[p1] == 0);
        self.aux_write_flag(Flag::FLAG_H, false);
        self.aux_write_flag(Flag::FLAG_N, false);
        self.aux_write_flag(Flag::FLAG_C, false);
    }

    fn xor_r8_8(&mut self, p1: Reg, p2: u8)
    {
        self.regs[p1] ^= p2;

        self.aux_write_flag(Flag::FLAG_Z, self.regs[p1] == 0);
        self.aux_write_flag(Flag::FLAG_H, false);
        self.aux_write_flag(Flag::FLAG_N, false);
        self.aux_write_flag(Flag::FLAG_C, false);
    }

    fn xor_r8_r16a(&mut self, ram: &mut Ram, p1: Reg, msh: Reg, lsh: Reg)
    {
        self.regs[p1] ^= ram.read_rp(self.regs[msh], self.regs[lsh]);

        self.aux_write_flag(Flag::FLAG_Z, self.regs[p1] == 0);
        self.aux_write_flag(Flag::FLAG_H, false);
        self.aux_write_flag(Flag::FLAG_N, false);
        self.aux_write_flag(Flag::FLAG_C, false);
    }

    fn or_r8_r8(&mut self, p1: Reg, p2: Reg)
    {
        self.regs[p1] |= self.regs[p2];

        self.aux_write_flag(Flag::FLAG_Z, self.regs[p1] == 0);
        self.aux_write_flag(Flag::FLAG_H, false);
        self.aux_write_flag(Flag::FLAG_N, false);
        self.aux_write_flag(Flag::FLAG_C, false);
    }

    fn or_r8_8(&mut self, p1: Reg, p2: u8)
    {
        self.regs[p1] |= p2;

        self.aux_write_flag(Flag::FLAG_Z, self.regs[p1] == 0);
        self.aux_write_flag(Flag::FLAG_H, false);
        self.aux_write_flag(Flag::FLAG_N, false);
        self.aux_write_flag(Flag::FLAG_C, false);
    }

    fn or_r8_r16a(&mut self, ram: &mut Ram, p1: Reg, msh: Reg, lsh: Reg)
    {
        self.regs[p1] |= ram.read_rp(self.regs[msh], self.regs[lsh]);

        self.aux_write_flag(Flag::FLAG_Z, self.regs[p1] == 0);
        self.aux_write_flag(Flag::FLAG_H, false);
        self.aux_write_flag(Flag::FLAG_N, false);
        self.aux_write_flag(Flag::FLAG_C, false);
    }

    fn cp_r8_r8(&mut self, p1: Reg, p2: Reg)
    {
        let half_carry_pre = ((self.regs[p1] ^ self.regs[p2]) >> 4) & 1;
        let result = self.regs[p1].overflowing_sub(self.regs[p2]);
        self.regs[p1] = result.0;
        let half_carry_post = (result.0 >> 4) & 1;

        self.aux_write_flag(Flag::FLAG_Z, result.0 == 0);
        self.aux_write_flag(Flag::FLAG_H, half_carry_pre != half_carry_post);
        self.aux_write_flag(Flag::FLAG_N, true);
        self.aux_write_flag(Flag::FLAG_C, result.1);
    }

    fn cp_r8_8(&mut self, p1: Reg, p2: u8)
    {
        let half_carry_pre = ((self.regs[p1] ^ p2) >> 4) & 1;
        let result = self.regs[p1].overflowing_sub(p2);
        self.regs[p1] = result.0;
        let half_carry_post = (result.0 >> 4) & 1;

        self.aux_write_flag(Flag::FLAG_Z, result.0 == 0);
        self.aux_write_flag(Flag::FLAG_H, half_carry_pre != half_carry_post);
        self.aux_write_flag(Flag::FLAG_N, true);
        self.aux_write_flag(Flag::FLAG_C, result.1);
    }

    fn cp_r8_r16a(&mut self, ram: &mut Ram, p1: Reg, msh: Reg, lsh: Reg)
    {
        let p2 = ram.read_rp(self.regs[msh], self.regs[lsh]);
        let half_carry_pre = ((self.regs[p1] ^ p2) >> 4) & 1;
        let result = self.regs[p1].overflowing_add(p2);
        self.regs[p1] = result.0;
        let half_carry_post = (result.0 >> 4) & 1;

        self.aux_write_flag(Flag::FLAG_Z, result.0 == 0);
        self.aux_write_flag(Flag::FLAG_H, half_carry_pre != half_carry_post);
        self.aux_write_flag(Flag::FLAG_N, true);
        self.aux_write_flag(Flag::FLAG_C, result.1);
    }

    fn daa(&mut self)
    {
        let c_before = self.aux_read_flag(Flag::FLAG_C);
        let bits_47 = (self.regs[REG_A] >> 4) & 0b00001111;
        let h_before = self.aux_read_flag(Flag::FLAG_H);
        let bits_03 = self.regs[REG_A] & 0b00001111;
        if self.aux_read_flag(Flag::FLAG_N) == true //Add preceded instruction
        {
            match (c_before, bits_47, h_before, bits_03)
            {
                (false, 0x0..=0x9, false, 0x0..=0x9) => {
                    self.regs[REG_A] = self.regs[REG_A].wrapping_add(0x00);
                    self.aux_write_flag(Flag::FLAG_C, false);
                },
                (false, 0x0..=0x8, false, 0xA..=0xF) => {
                    self.regs[REG_A] = self.regs[REG_A].wrapping_add(0x06);
                    self.aux_write_flag(Flag::FLAG_C, false);
                },
                (false, 0x0..=0x9, true, 0x0..=0x3) => {
                    self.regs[REG_A] = self.regs[REG_A].wrapping_add(0x06);
                    self.aux_write_flag(Flag::FLAG_C, false);
                },
                (false, 0xA..=0xF, false, 0x0..=0x9) => {
                    self.regs[REG_A] = self.regs[REG_A].wrapping_add(0x60);
                    self.aux_write_flag(Flag::FLAG_C, true);
                },
                (false, 0x9..=0xF, false, 0xA..=0xF) => {
                    self.regs[REG_A] = self.regs[REG_A].wrapping_add(0x66);
                    self.aux_write_flag(Flag::FLAG_C, true);
                },
                (false, 0xA..=0xF, true, 0x0..=0x3) => {
                    self.regs[REG_A] = self.regs[REG_A].wrapping_add(0x66);
                    self.aux_write_flag(Flag::FLAG_C, true);
                },
                (true, 0x0..=0x2, false, 0x0..=0x9) => {
                    self.regs[REG_A] = self.regs[REG_A].wrapping_add(0x60);
                    self.aux_write_flag(Flag::FLAG_C, true);
                },
                (true, 0x0..=0x2, false, 0xA..=0xF) => {
                    self.regs[REG_A] = self.regs[REG_A].wrapping_add(0x66);
                    self.aux_write_flag(Flag::FLAG_C, true);
                },
                (true, 0x0..=0x3, true, 0x0..=0x3) => {
                    self.regs[REG_A] = self.regs[REG_A].wrapping_add(0x66);
                    self.aux_write_flag(Flag::FLAG_C, true);
                },
                _ => panic!("Invalid BDC conversion")
            }
        }
        else //subtract preceded instruction
        {
            match (c_before, bits_47, h_before, bits_03)
            {
                (false, 0x0..=0x9, false, 0x0..=0x9) => {
                    self.regs[REG_A] = self.regs[REG_A].wrapping_add(0x00);
                    self.aux_write_flag(Flag::FLAG_C, false);
                },
                (false, 0x0..=0x8, true, 0x6..=0xF) => {
                    self.regs[REG_A] = self.regs[REG_A].wrapping_add(0xFA);
                    self.aux_write_flag(Flag::FLAG_C, false);
                },
                (true, 0x7..=0xF, false, 0x0..=0x9) => {
                    self.regs[REG_A] = self.regs[REG_A].wrapping_add(0xA0);
                    self.aux_write_flag(Flag::FLAG_C, true);
                },
                (true, 0x6..=0xF, true, 0x6..=0xF) => {
                    self.regs[REG_A] = self.regs[REG_A].wrapping_add(0x9A);
                    self.aux_write_flag(Flag::FLAG_C, true);
                },
                _ => panic!("Invalid BDC conversion")
            }
        }
    }

    fn cpl(&mut self)
    {
        self.regs[REG_A] = !self.regs[REG_A];
        self.aux_write_flag(Flag::FLAG_N, true);
        self.aux_write_flag(Flag::FLAG_H, true);
    }

    fn ccf(&mut self)
    {
        self.aux_write_flag(Flag::FLAG_C, !self.aux_read_flag(Flag::FLAG_C));
        self.aux_write_flag(Flag::FLAG_N, false);
        self.aux_write_flag(Flag::FLAG_H, false);
    }

    fn scf(&mut self)
    {
        self.aux_write_flag(Flag::FLAG_C, true);
        self.aux_write_flag(Flag::FLAG_N, false);
        self.aux_write_flag(Flag::FLAG_H, false);
    }

    fn jp_pc_16(&mut self, msh: u8, lsh: u8)
    {
        self.pc.reg = u16::from_le_bytes([lsh, msh]);
        self.pc.should_increment = false;
    }

    fn jp_flag_pc_16(&mut self, flag: Flag, msh: u8, lsh: u8)
    {
        if self.aux_read_flag(flag) == true
        {
            self.jp_pc_16(msh, lsh);
        }
    }

    fn jp_nflag_pc_16(&mut self, flag: Flag, msh: u8, lsh: u8)
    {
        if self.aux_read_flag(flag) == false
        {
            self.jp_pc_16(msh, lsh);
        }
    }

    fn jr_i8(&mut self, p1: i8)
    {
        let conv = p1.unsigned_abs() as u16;
        let negative = p1 < 0;
        if negative
        {
            self.pc.reg -= conv;
        }
        else
        {
            self.pc.reg += conv;
        }
        self.pc.should_increment = false;
    }

    fn jr_flag_i8(&mut self, flag: Flag, p1: i8)
    {
        if self.aux_read_flag(flag) == true
        {
            self.jr_i8(p1);
        }
    }

    fn jr_nflag_i8(&mut self, flag: Flag, p1: i8)
    {
        if self.aux_read_flag(flag) == false
        {
            self.jr_i8(p1);
        }
    }

    fn call_16(&mut self, ram: &mut Ram, msh: u8, lsh: u8)
    {
        let pc_bytes = self.pc.reg.to_le_bytes();
        ram.write(self.sp - 1, pc_bytes[1]);
        ram.write(self.sp - 2, pc_bytes[0]);
        self.pc.reg = u16::from_le_bytes([lsh, msh]);
        self.sp -= 2;
    }

    fn call_flag_16(&mut self, ram: &mut Ram, flag: Flag, msh: u8, lsh: u8)
    {
        if self.aux_read_flag(flag) == true
        {
            self.call_16(ram, msh, lsh);
        }
    }

    fn call_nflag_16(&mut self, ram: &mut Ram, flag: Flag, msh: u8, lsh: u8)
    {
        if self.aux_read_flag(flag) == false
        {
            self.call_16(ram, msh, lsh);
        }
    }

    fn ret(&mut self, ram: &mut Ram)
    {
        let sp_lsh = ram.read(self.sp);
        let sp_msh = ram.read(self.sp + 1);
        self.pc.reg = u16::from_le_bytes([sp_lsh, sp_msh]);
        self.sp += 2;
    }

    fn ret_flag(&mut self, ram: &mut Ram, flag: Flag)
    {
        if self.aux_read_flag(flag) == true
        {
            self.ret(ram);
        }
    }

    fn ret_nflag(&mut self, ram: &mut Ram, flag: Flag)
    {
        if self.aux_read_flag(flag) == false
        {
            self.ret(ram);
        }
    }

    fn rst(&mut self, ram: &mut Ram, loc: u8)
    {
        let pc_bytes = self.pc.reg.to_le_bytes();
        ram.write(self.sp - 1, pc_bytes[1]);
        ram.write(self.sp - 2, pc_bytes[0]);
        self.sp -= 2;
        self.pc.reg = u16::from_le_bytes([loc, 0]);
    }


    //--------------------16 BIT OPCODES--------------------

    fn rlc_r8(&mut self, p1: Reg)
    {
        self.aux_write_flag(Flag::FLAG_C, (self.regs[p1] >> 7) & 1 != 0);
        self.regs[p1] = self.regs[p1].rotate_left(1);

        self.aux_write_flag(Flag::FLAG_Z, self.regs[p1] == 0);
        self.aux_write_flag(Flag::FLAG_N, false);
        self.aux_write_flag(Flag::FLAG_H, false);
    }

    fn rlca(&mut self)
    {
        self.aux_write_flag(Flag::FLAG_C, (self.regs[REG_A] >> 7) & 1 != 0);
        self.regs[REG_A] = self.regs[REG_A].rotate_left(1);

        self.aux_write_flag(Flag::FLAG_Z, false);
        self.aux_write_flag(Flag::FLAG_N, false);
        self.aux_write_flag(Flag::FLAG_H, false);
    }

    fn rlc_r16a(&mut self, ram: &mut Ram, msh: Reg, lsh: Reg)
    {
        let p1 = ram.read_rp(self.regs[msh], self.regs[lsh]);
        self.aux_write_flag(Flag::FLAG_C, (p1 >> 7) & 1 != 0);
        let result = p1.rotate_left(1);
        ram.write_rp(self.regs[msh], self.regs[lsh], result);

        self.aux_write_flag(Flag::FLAG_Z, result == 0);
        self.aux_write_flag(Flag::FLAG_N, false);
        self.aux_write_flag(Flag::FLAG_H, false);
    }

    fn rrc_r8(&mut self, p1: Reg)
    {
        self.aux_write_flag(Flag::FLAG_C, self.regs[p1] & 1 != 0);
        self.regs[p1] = self.regs[p1].rotate_right(1);

        self.aux_write_flag(Flag::FLAG_Z, self.regs[p1] == 0);
        self.aux_write_flag(Flag::FLAG_N, false);
        self.aux_write_flag(Flag::FLAG_H, false);
    }

    fn rrca(&mut self)
    {
        self.aux_write_flag(Flag::FLAG_C, self.regs[REG_A] & 1 != 0);
        self.regs[REG_A] = self.regs[REG_A].rotate_right(1);

        self.aux_write_flag(Flag::FLAG_Z, false);
        self.aux_write_flag(Flag::FLAG_N, false);
        self.aux_write_flag(Flag::FLAG_H, false);
    }

    fn rrc_r16a(&mut self, ram: &mut Ram, msh: Reg, lsh: Reg)
    {
        let p1 = ram.read_rp(self.regs[msh], self.regs[lsh]);
        self.aux_write_flag(Flag::FLAG_C, p1 & 1 != 0);
        let result = p1.rotate_right(1);
        ram.write_rp(self.regs[msh], self.regs[lsh], result);

        self.aux_write_flag(Flag::FLAG_Z, result == 0);
        self.aux_write_flag(Flag::FLAG_N, false);
        self.aux_write_flag(Flag::FLAG_H, false);
    }

    fn rl_r8(&mut self, p1: Reg)
    {
        let cin = self.aux_read_flag(Flag::FLAG_C) as u8;
        self.aux_write_flag(Flag::FLAG_C, (self.regs[p1] >> 7) & 1 != 0);
        self.regs[p1] = (self.regs[p1] << 1u8) | cin;

        self.aux_write_flag(Flag::FLAG_Z, self.regs[p1] == 0);
        self.aux_write_flag(Flag::FLAG_N, false);
        self.aux_write_flag(Flag::FLAG_H, false);
    }

    fn rla(&mut self)
    {
        let cin = self.aux_read_flag(Flag::FLAG_C) as u8;
        self.aux_write_flag(Flag::FLAG_C, (self.regs[REG_A] >> 7) & 1 != 0);
        self.regs[REG_A] = (self.regs[REG_A] << 1u8) | cin;

        self.aux_write_flag(Flag::FLAG_Z, false);
        self.aux_write_flag(Flag::FLAG_N, false);
        self.aux_write_flag(Flag::FLAG_H, false);
    }

    fn rl_r16a(&mut self, ram: &mut Ram, msh: Reg, lsh: Reg)
    {
        let cin = self.aux_read_flag(Flag::FLAG_C) as u8;
        let p1 = ram.read_rp(self.regs[msh], self.regs[lsh]);
        self.aux_write_flag(Flag::FLAG_C, (p1 >> 7) & 1 != 0);
        let result = (p1 << 1u8) | cin;
        ram.write_rp(self.regs[msh], self.regs[lsh], result);

        self.aux_write_flag(Flag::FLAG_Z, result == 0);
        self.aux_write_flag(Flag::FLAG_N, false);
        self.aux_write_flag(Flag::FLAG_H, false);
    }

    fn rr_r8(&mut self, p1: Reg)
    {
        let cin = self.aux_read_flag(Flag::FLAG_C) as u8;
        self.aux_write_flag(Flag::FLAG_C, self.regs[p1] & 1 != 0);
        self.regs[p1] = (self.regs[p1] >> 1u8) | (cin << 7u8);

        self.aux_write_flag(Flag::FLAG_Z, self.regs[p1] == 0);
        self.aux_write_flag(Flag::FLAG_N, false);
        self.aux_write_flag(Flag::FLAG_H, false);
    }

    fn rra(&mut self)
    {
        let cin = self.aux_read_flag(Flag::FLAG_C) as u8;
        self.aux_write_flag(Flag::FLAG_C, self.regs[REG_A] & 1 != 0);
        self.regs[REG_A] = (self.regs[REG_A] >> 1u8) | (cin << 7u8);

        self.aux_write_flag(Flag::FLAG_Z, false);
        self.aux_write_flag(Flag::FLAG_N, false);
        self.aux_write_flag(Flag::FLAG_H, false);
    }

    fn rr_r16a(&mut self, ram: &mut Ram, msh: Reg, lsh: Reg)
    {
        let cin = self.aux_read_flag(Flag::FLAG_C) as u8;
        let p1 = ram.read_rp(self.regs[msh], self.regs[lsh]);
        self.aux_write_flag(Flag::FLAG_C, p1 & 1 != 0);
        let result = (p1 >> 1u8) | (cin << 7u8);
        ram.write_rp(self.regs[msh], self.regs[lsh], result);

        self.aux_write_flag(Flag::FLAG_Z, result == 0);
        self.aux_write_flag(Flag::FLAG_N, false);
        self.aux_write_flag(Flag::FLAG_H, false);
    }

    fn sla_r8(&mut self, p1: Reg)
    {
        self.aux_write_flag(Flag::FLAG_C, (self.regs[p1] >> 7) & 1 != 0);
        self.regs[p1] <<= 1u8;

        self.aux_write_flag(Flag::FLAG_Z, self.regs[p1] == 0);
        self.aux_write_flag(Flag::FLAG_N, false);
        self.aux_write_flag(Flag::FLAG_H, false);
    }

    fn sla_r16a(&mut self, ram: &mut Ram, msh: Reg, lsh: Reg)
    {
        let p1 = ram.read_rp(self.regs[msh], self.regs[lsh]);
        self.aux_write_flag(Flag::FLAG_C, (p1 >> 7) & 1 != 0);
        let result = p1 << 1u8;
        ram.write_rp(self.regs[msh], self.regs[lsh], result);

        self.aux_write_flag(Flag::FLAG_Z, result == 0);
        self.aux_write_flag(Flag::FLAG_N, false);
        self.aux_write_flag(Flag::FLAG_H, false);
    }

    fn sra_r8(&mut self, p1: Reg)
    {
        self.aux_write_flag(Flag::FLAG_C, self.regs[p1] & 1 != 0);
        self.regs[p1] = (self.regs[p1] >> 1u8) | (self.regs[p1] & 0b10000000u8); //fill with leftmost

        self.aux_write_flag(Flag::FLAG_Z, self.regs[p1] == 0);
        self.aux_write_flag(Flag::FLAG_N, false);
        self.aux_write_flag(Flag::FLAG_H, false);
    }

    fn sra_r16a(&mut self, ram: &mut Ram, msh: Reg, lsh: Reg)
    {
        let p1 = ram.read_rp(self.regs[msh], self.regs[lsh]);
        self.aux_write_flag(Flag::FLAG_C, p1 & 1 != 0);
        let result =( p1 >> 1u8) | (p1 | 0b10000000u8);
        ram.write_rp(self.regs[msh], self.regs[lsh], result);

        self.aux_write_flag(Flag::FLAG_Z, result == 0);
        self.aux_write_flag(Flag::FLAG_N, false);
        self.aux_write_flag(Flag::FLAG_H, false);
    }

    fn srl_r8(&mut self, p1: Reg)
    {
        self.aux_write_flag(Flag::FLAG_C, self.regs[p1] & 1 != 0);
        self.regs[p1] >>= 1u8; //fill with leftmost

        self.aux_write_flag(Flag::FLAG_Z, self.regs[p1] == 0);
        self.aux_write_flag(Flag::FLAG_N, false);
        self.aux_write_flag(Flag::FLAG_H, false);
    }

    fn srl_r16a(&mut self, ram: &mut Ram, msh: Reg, lsh: Reg)
    {
        let p1 = ram.read_rp(self.regs[msh], self.regs[lsh]);
        self.aux_write_flag(Flag::FLAG_C, p1 & 1 != 0);
        let result = p1 >> 1u8;
        ram.write_rp(self.regs[msh], self.regs[lsh], result);

        self.aux_write_flag(Flag::FLAG_Z, result == 0);
        self.aux_write_flag(Flag::FLAG_N, false);
        self.aux_write_flag(Flag::FLAG_H, false);
    }

    fn swap_r8(&mut self, p1: Reg)
    {
        let lower_to_upper_half = self.regs[p1] << 4u8;
        let upper_to_lower_half = self.regs[p1] >> 4u8;
        self.regs[p1] = lower_to_upper_half | upper_to_lower_half;
    }

    fn swap_r16a(&mut self, ram: &mut Ram, msh: Reg, lsh: Reg)
    {
        let p1 = ram.read_rp(self.regs[msh], self.regs[lsh]);
        let lower_to_upper_half = p1 << 4u8;
        let upper_to_lower_half = p1 >> 4u8;
        ram.write_rp(self.regs[msh], self.regs[lsh], lower_to_upper_half | upper_to_lower_half);
    }

    fn bit_r8(&mut self, p1: u8, p2: Reg)
    {
        self.aux_write_flag(Flag::FLAG_H, true);
        self.aux_write_flag(Flag::FLAG_N, false);
        self.aux_write_flag(Flag::FLAG_Z, (self.regs[p2] & (1u8 << p1)) == 0);
    }

    fn bit_r16a(&mut self, ram: &mut Ram, p1: u8, msh: Reg, lsh: Reg)
    {
        self.aux_write_flag(Flag::FLAG_H, true);
        self.aux_write_flag(Flag::FLAG_N, false);
        self.aux_write_flag(Flag::FLAG_Z,
            (ram.read_rp(self.regs[msh], self.regs[lsh]) & (1u8 << p1)) == 0);
    }

    fn res_r8(&mut self, p1: u8, p2: Reg)
    {
        self.regs[p2] &= !(1u8 << p1);
    }

    fn res_r16a(&mut self, ram: &mut Ram, p1: u8, msh: Reg, lsh: Reg)
    {
        ram.write_rp(self.regs[msh], self.regs[lsh],
            ram.read_rp(self.regs[msh], self.regs[lsh]) & (!(1u8 << p1)));
    }

    fn set_r8(&mut self, p1: u8, p2: Reg)
    {
        self.regs[p2] |= 1u8 << p1;
    }

    fn set_r16a(&mut self, ram: &mut Ram, p1: u8, msh: Reg, lsh: Reg)
    {
        ram.write_rp(self.regs[msh], self.regs[lsh],
            ram.read_rp(self.regs[msh], self.regs[lsh]) | (1u8 << p1));
    }

    fn push_r16(&mut self, ram: &mut Ram, msh: Reg, lsh: Reg)
    {
        ram.write(self.sp - 1, self.regs[msh]);
        ram.write(self.sp - 2, self.regs[lsh]);
        self.sp -= 2;
    }

    fn push_pc(&mut self, ram: &mut Ram)
    {
        let bytes = self.pc.reg.to_le_bytes();
        ram.write(self.sp - 1, bytes[1]);
        ram.write(self.sp - 2, bytes[0]);
        self.sp -= 2;
    }

    fn pop_r16(&mut self, ram: &mut Ram, msh: Reg, lsh: Reg)
    {
        self.regs[lsh] = ram.read(self.sp);
        self.regs[msh] = ram.read(self.sp + 1);
        self.sp += 2;
    }

    //----------INTERRUPT MANAGEMENT----------

    fn reti(&mut self, ram: &mut Ram)
    {
        let l_bytes = ram.read(self.sp);
        self.inc_sp();
        let h_bytes = ram.read(self.sp);
        self.inc_sp();
        self.pc.reg = u16::from_le_bytes([l_bytes, h_bytes]);
        self.ime = true;
    }

    fn ei(&mut self)
    {
        self.ime = true;
    }

    fn di(&mut self)
    {
        self.ime = false;
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
                self.push_pc(ram);
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
                    self.ld_r16_16(REG_B, REG_C, msh, lsh);
                },
                0x02 => {self.ld_r16a_r8(ram, REG_B, REG_C, REG_A);},
                0x03 => {self.inc_r16(REG_B, REG_C);},
                0x04 => {self.inc_r8(REG_B);},
                0x05 => {self.dec_r8(REG_B);},
                0x06 => {
                    let num = self.aux_read_immediate_data(ram);
                    self.ld_r8_8(REG_B, num);
                },
                0x07 => {self.rlca();},
                0x08 => {
                    let lsh = self.aux_read_immediate_data(ram);
                    let msh = self.aux_read_immediate_data(ram);
                    self.ld_16a_sp(ram, msh, lsh);
                },
                0x09 => {self.add_r16_r16(REG_H, REG_L, REG_B, REG_C);},
                0x0A => {self.ld_r8_r16a(ram, REG_A, REG_B, REG_C);},
                0x0B => {self.dec_r16(REG_B, REG_C);},
                0x0C => {self.inc_r8(REG_C);},
                0x0D => {self.dec_r8(REG_C);},
                0x0E => {
                    let num = self.aux_read_immediate_data(ram);
                    self.ld_r8_8(REG_C, num);
                },
                0x0F => {self.rrca();},
                0x10 => {self.stop();},
                0x11 => {
                    let lsh = self.aux_read_immediate_data(ram);
                    let msh = self.aux_read_immediate_data(ram);
                    self.ld_r16_16(REG_D, REG_E, msh, lsh);
                },
                0x12 => {self.ld_r16a_r8(ram, REG_D, REG_E, REG_A);},
                0x13 => {self.inc_r16(REG_D, REG_E);},
                0x14 => {self.inc_r8(REG_D);},
                0x15 => {self.dec_r8(REG_D);},
                0x16 => {
                    let num = self.aux_read_immediate_data(ram);
                    self.ld_r8_8(REG_D, num);
                },
                0x17 => {self.rla();},
                0x18 => {
                    let immediate = self.aux_read_immediate_data(ram) as i8;
                    self.jr_i8(immediate);
                },
                0x19 => {self.add_r16_r16(REG_H, REG_L, REG_D, REG_E);},
                0x1A => {self.ld_r8_r16a(ram, REG_A, REG_D, REG_E);},
                0x1B => {self.dec_r16(REG_D, REG_E);},
                0x1C => {self.inc_r8(REG_E);},
                0x1D => {self.dec_r8(REG_E);},
                0x1E => {
                    let num = self.aux_read_immediate_data(ram);
                    self.ld_r8_8(REG_E, num);
                },
                0x1F => {self.rra();},
                0x20 => {
                    let immediate = self.aux_read_immediate_data(ram) as i8;
                    self.jr_nflag_i8(Flag::FLAG_Z, immediate);
                },
                0x21 => {
                    let lsh = self.aux_read_immediate_data(ram);
                    let msh = self.aux_read_immediate_data(ram);
                    self.ld_r16_16(REG_H, REG_L, msh, lsh);
                },
                0x22 => {
                    self.ld_r16a_r8(ram, REG_H, REG_L, REG_A);
                    self.inc_r16(REG_H, REG_L);
                },
                0x23 => {self.inc_r16(REG_H, REG_L);},
                0x24 => {self.inc_r8(REG_H);},
                0x25 => {self.dec_r8(REG_H);},
                0x26 => {
                    let num = self.aux_read_immediate_data(ram);
                    self.ld_r8_8(REG_H, num);
                },
                0x27 => {self.daa();},
                0x28 => {
                    let immediate = self.aux_read_immediate_data(ram) as i8;
                    self.jr_flag_i8(Flag::FLAG_Z, immediate);
                },
                0x29 => {self.add_r16_r16(REG_H, REG_L, REG_H, REG_L)},
                0x2A => {
                    self.ld_r8_r16a(ram, REG_A, REG_H, REG_L);
                    self.inc_r16(REG_H, REG_L);
                },
                0x2B => {self.dec_r16(REG_H, REG_L);},
                0x2C => {self.inc_r8(REG_L);},
                0x2D => {self.dec_r8(REG_L);},
                0x2E => {
                    let num = self.aux_read_immediate_data(ram);
                    self.ld_r8_8(REG_L, num);
                },
                0x2F => {self.cpl();},
                0x30 => {
                    let immediate = self.aux_read_immediate_data(ram) as i8;
                    self.jr_nflag_i8(Flag::FLAG_C, immediate);
                },
                0x31 => {
                    let lsh = self.aux_read_immediate_data(ram);
                    let msh = self.aux_read_immediate_data(ram);
                    self.ld_sp_16( msh, lsh);
                },
                0x32 => {
                    self.ld_r16a_r8(ram, REG_H, REG_L, REG_A);
                    self.dec_r16(REG_H, REG_L);
                },
                0x33 => {self.inc_sp();},
                0x34 => {self.inc_r16a(ram, REG_H, REG_L);},
                0x35 => {self.dec_r16a(ram, REG_H, REG_L);},
                0x36 => {
                    let num = self.aux_read_immediate_data(ram);
                    self.ld_r16a_8(ram, REG_H, REG_L, num);
                },
                0x37 => {self.scf();},
                0x38 => {
                    let immediate = self.aux_read_immediate_data(ram) as i8;
                    self.jr_flag_i8(Flag::FLAG_C, immediate);
                },
                0x39 => {self.add_r16_sp(REG_H, REG_L);},
                0x3A => {
                    self.ld_r8_r16a(ram, REG_A, REG_H, REG_L);
                    self.dec_r16(REG_H, REG_L);
                },
                0x3B => {self.dec_sp();},
                0x3C => {self.inc_r8(REG_A);},
                0x3D => {self.dec_r8(REG_A);},
                0x3E => {
                    let num = self.aux_read_immediate_data(ram);
                    self.ld_r8_8(REG_A, num);
                },
                0x3F => {self.ccf();},
                0x40 => {self.ld_r8_r8(REG_B, REG_B);},
                0x41 => {self.ld_r8_r8(REG_B, REG_C);},
                0x42 => {self.ld_r8_r8(REG_B, REG_D);},
                0x43 => {self.ld_r8_r8(REG_B, REG_E);},
                0x44 => {self.ld_r8_r8(REG_B, REG_H);},
                0x45 => {self.ld_r8_r8(REG_B, REG_L);},
                0x46 => {self.ld_r8_r16a(ram, REG_B, REG_H, REG_L);},
                0x47 => {self.ld_r8_r8(REG_B, REG_A);},
                0x48 => {self.ld_r8_r8(REG_C, REG_B);},
                0x49 => {self.ld_r8_r8(REG_C, REG_C);},
                0x4A => {self.ld_r8_r8(REG_C, REG_D);},
                0x4B => {self.ld_r8_r8(REG_C, REG_E);},
                0x4C => {self.ld_r8_r8(REG_C, REG_H);},
                0x4D => {self.ld_r8_r8(REG_C, REG_L);},
                0x4E => {self.ld_r8_r16a(ram, REG_C, REG_H, REG_L);},
                0x4F => {self.ld_r8_r8(REG_C, REG_A);},
                0x50 => {self.ld_r8_r8(REG_D, REG_B);},
                0x51 => {self.ld_r8_r8(REG_D, REG_C);},
                0x52 => {self.ld_r8_r8(REG_D, REG_D);},
                0x53 => {self.ld_r8_r8(REG_D, REG_E);},
                0x54 => {self.ld_r8_r8(REG_D, REG_H);},
                0x55 => {self.ld_r8_r8(REG_D, REG_L);},
                0x56 => {self.ld_r8_r16a(ram, REG_D, REG_H, REG_L);},
                0x57 => {self.ld_r8_r8(REG_D, REG_A);},
                0x58 => {self.ld_r8_r8(REG_E, REG_B);},
                0x59 => {self.ld_r8_r8(REG_E, REG_C);},
                0x5A => {self.ld_r8_r8(REG_E, REG_D);},
                0x5B => {self.ld_r8_r8(REG_E, REG_E);},
                0x5C => {self.ld_r8_r8(REG_E, REG_H);},
                0x5D => {self.ld_r8_r8(REG_E, REG_L);},
                0x5E => {self.ld_r8_r16a(ram, REG_E, REG_H, REG_L);},
                0x5F => {self.ld_r8_r8(REG_E, REG_A);},
                0x60 => {self.ld_r8_r8(REG_H, REG_B);},
                0x61 => {self.ld_r8_r8(REG_H, REG_C);},
                0x62 => {self.ld_r8_r8(REG_H, REG_D);},
                0x63 => {self.ld_r8_r8(REG_H, REG_E);},
                0x64 => {self.ld_r8_r8(REG_H, REG_H);},
                0x65 => {self.ld_r8_r8(REG_H, REG_L);},
                0x66 => {self.ld_r8_r16a(ram, REG_H, REG_H, REG_L);},
                0x67 => {self.ld_r8_r8(REG_H, REG_A);},
                0x68 => {self.ld_r8_r8(REG_L, REG_B);},
                0x69 => {self.ld_r8_r8(REG_L, REG_C);},
                0x6A => {self.ld_r8_r8(REG_L, REG_D);},
                0x6B => {self.ld_r8_r8(REG_L, REG_E);},
                0x6C => {self.ld_r8_r8(REG_L, REG_H);},
                0x6D => {self.ld_r8_r8(REG_L, REG_L);},
                0x6E => {self.ld_r8_r16a(ram, REG_L, REG_H, REG_L);},
                0x6F => {self.ld_r8_r8(REG_L, REG_A);},
                0x70 => {self.ld_r16a_r8(ram, REG_H, REG_L, REG_B);},
                0x71 => {self.ld_r16a_r8(ram, REG_H, REG_L, REG_C);},
                0x72 => {self.ld_r16a_r8(ram, REG_H, REG_L, REG_D);},
                0x73 => {self.ld_r16a_r8(ram, REG_H, REG_L, REG_E);},
                0x74 => {self.ld_r16a_r8(ram, REG_H, REG_L, REG_H);},
                0x75 => {self.ld_r16a_r8(ram, REG_H, REG_L, REG_L);},
                0x76 => {self.halt();},
                0x77 => {self.ld_r16a_r8(ram, REG_H, REG_L, REG_A);},
                0x78 => {self.ld_r8_r8(REG_A, REG_B);},
                0x79 => {self.ld_r8_r8(REG_A, REG_C);},
                0x7A => {self.ld_r8_r8(REG_A, REG_D);},
                0x7B => {self.ld_r8_r8(REG_A, REG_E);},
                0x7C => {self.ld_r8_r8(REG_A, REG_H);},
                0x7D => {self.ld_r8_r8(REG_A, REG_L);},
                0x7E => {self.ld_r8_r16a(ram, REG_A, REG_H, REG_L);},
                0x7F => {self.ld_r8_r8(REG_A, REG_A);},
                0x80 => {self.add_r8_r8(REG_A, REG_B);},
                0x81 => {self.add_r8_r8(REG_A, REG_C);},
                0x82 => {self.add_r8_r8(REG_A, REG_D);},
                0x83 => {self.add_r8_r8(REG_A, REG_E);},
                0x84 => {self.add_r8_r8(REG_A, REG_H);},
                0x85 => {self.add_r8_r8(REG_A, REG_L);},
                0x86 => {self.add_r8_r16a(ram, REG_A, REG_H, REG_L);},
                0x87 => {self.add_r8_r8(REG_A, REG_A);},
                0x88 => {self.adc_r8_r8(REG_A, REG_B);},
                0x89 => {self.adc_r8_r8(REG_A, REG_C);},
                0x8A => {self.adc_r8_r8(REG_A, REG_D);},
                0x8B => {self.adc_r8_r8(REG_A, REG_E);},
                0x8C => {self.adc_r8_r8(REG_A, REG_H);},
                0x8D => {self.adc_r8_r8(REG_A, REG_L);},
                0x8E => {self.adc_r8_r16a(ram, REG_A, REG_H, REG_L);},
                0x8F => {self.adc_r8_r8(REG_A, REG_A);},
                0x90 => {self.sub_r8_r8(REG_A, REG_B);},
                0x91 => {self.sub_r8_r8(REG_A, REG_C);},
                0x92 => {self.sub_r8_r8(REG_A, REG_D);},
                0x93 => {self.sub_r8_r8(REG_A, REG_E);},
                0x94 => {self.sub_r8_r8(REG_A, REG_H);},
                0x95 => {self.sub_r8_r8(REG_A, REG_L);},
                0x96 => {self.sub_r8_r16a(ram, REG_A, REG_H, REG_L);},
                0x97 => {self.sub_r8_r8(REG_A, REG_A);},
                0x98 => {self.sbc_r8_r8(REG_A, REG_B);},
                0x99 => {self.sbc_r8_r8(REG_A, REG_C);},
                0x9A => {self.sbc_r8_r8(REG_A, REG_D);},
                0x9B => {self.sbc_r8_r8(REG_A, REG_E);},
                0x9C => {self.sbc_r8_r8(REG_A, REG_H);},
                0x9D => {self.sbc_r8_r8(REG_A, REG_L);},
                0x9E => {self.sbc_r8_r16a(ram, REG_A, REG_H, REG_L);},
                0x9F => {self.sbc_r8_r8(REG_A, REG_A);},
                0xA0 => {self.and_r8_r8(REG_A, REG_B);},
                0xA1 => {self.and_r8_r8(REG_A, REG_C);},
                0xA2 => {self.and_r8_r8(REG_A, REG_D);},
                0xA3 => {self.and_r8_r8(REG_A, REG_E);},
                0xA4 => {self.and_r8_r8(REG_A, REG_H);},
                0xA5 => {self.and_r8_r8(REG_A, REG_L);},
                0xA6 => {self.and_r8_r16a(ram, REG_A, REG_H, REG_L);},
                0xA7 => {self.and_r8_r8(REG_A, REG_A);},
                0xA8 => {self.xor_r8_r8(REG_A, REG_B);},
                0xA9 => {self.xor_r8_r8(REG_A, REG_C);},
                0xAA => {self.xor_r8_r8(REG_A, REG_D);},
                0xAB => {self.xor_r8_r8(REG_A, REG_E);},
                0xAC => {self.xor_r8_r8(REG_A, REG_H);},
                0xAD => {self.xor_r8_r8(REG_A, REG_L);},
                0xAE => {self.xor_r8_r16a(ram, REG_A, REG_H, REG_L);},
                0xAF => {self.xor_r8_r8(REG_A, REG_A);},
                0xB0 => {self.or_r8_r8(REG_A, REG_B);},
                0xB1 => {self.or_r8_r8(REG_A, REG_C);},
                0xB2 => {self.or_r8_r8(REG_A, REG_D);},
                0xB3 => {self.or_r8_r8(REG_A, REG_E);},
                0xB4 => {self.or_r8_r8(REG_A, REG_H);},
                0xB5 => {self.or_r8_r8(REG_A, REG_L);},
                0xB6 => {self.or_r8_r16a(ram, REG_A, REG_H, REG_L);},
                0xB7 => {self.or_r8_r8(REG_A, REG_A);},
                0xB8 => {self.cp_r8_r8(REG_A, REG_B);},
                0xB9 => {self.cp_r8_r8(REG_A, REG_C);},
                0xBA => {self.cp_r8_r8(REG_A, REG_D);},
                0xBB => {self.cp_r8_r8(REG_A, REG_E);},
                0xBC => {self.cp_r8_r8(REG_A, REG_H);},
                0xBD => {self.cp_r8_r8(REG_A, REG_L);},
                0xBE => {self.cp_r8_r16a(ram, REG_A, REG_H, REG_L);},
                0xBF => {self.cp_r8_r8(REG_A, REG_A);},
                0xC0 => {self.ret_nflag(ram, Flag::FLAG_Z);},
                0xC1 => {self.pop_r16(ram, REG_B, REG_C);},
                0xC2 => {
                    let lsh = self.aux_read_immediate_data(ram);
                    let msh = self.aux_read_immediate_data(ram);
                    self.jp_nflag_pc_16(Flag::FLAG_Z, msh, lsh);
                },
                0xC3 => {
                    let lsh = self.aux_read_immediate_data(ram);
                    let msh = self.aux_read_immediate_data(ram);
                    self.jp_pc_16(msh, lsh);
                },
                0xC4 => {
                    let lsh = self.aux_read_immediate_data(ram);
                    let msh = self.aux_read_immediate_data(ram);
                    self.call_nflag_16(ram, Flag::FLAG_Z, msh, lsh);
                },
                0xC5 => {self.push_r16(ram, REG_B, REG_C);},
                0xC6 => {
                    let num = self.aux_read_immediate_data(ram);
                    self.add_r8_8(REG_A, num);
                },
                0xC7 => {self.rst(ram, 0x00);},
                0xC8 => {self.ret_flag(ram, Flag::FLAG_Z);},
                0xC9 => {self.ret(ram);},
                0xCA => {
                    let lsh = self.aux_read_immediate_data(ram);
                    let msh = self.aux_read_immediate_data(ram);
                    self.jp_flag_pc_16(Flag::FLAG_Z, msh, lsh);
                },
                0xCB => {/*Prefix for the next instruction, handled earlier*/},
                0xCC => {
                    let lsh = self.aux_read_immediate_data(ram);
                    let msh = self.aux_read_immediate_data(ram);
                    self.call_flag_16(ram, Flag::FLAG_Z, msh, lsh);
                },
                0xCD => {let lsh = self.aux_read_immediate_data(ram);
                    let msh = self.aux_read_immediate_data(ram);
                    self.call_16(ram, msh, lsh);},
                0xCE => {
                    let num = self.aux_read_immediate_data(ram);
                    self.adc_r8_8(REG_A, num);
                },
                0xCF => {self.rst(ram, 0x08);},
                0xD0 => {self.ret_nflag(ram, Flag::FLAG_C);},
                0xD1 => {self.pop_r16(ram, REG_D, REG_E);},
                0xD2 => {
                    let lsh = self.aux_read_immediate_data(ram);
                    let msh = self.aux_read_immediate_data(ram);
                    self.jp_nflag_pc_16(Flag::FLAG_C, msh, lsh);
                },
                0xD3 => {self.invalid_instruction(0xD3);},
                0xD4 => {
                    let lsh = self.aux_read_immediate_data(ram);
                    let msh = self.aux_read_immediate_data(ram);
                    self.call_nflag_16(ram, Flag::FLAG_C, msh, lsh);
                },
                0xD5 => {self.push_r16(ram, REG_D, REG_E);},
                0xD6 => {
                    let num = self.aux_read_immediate_data(ram);
                    self.sub_r8_8(REG_A, num);
                },
                0xD7 => {self.rst(ram, 0x10);},
                0xD8 => {self.ret_flag(ram, Flag::FLAG_C);},
                0xD9 => {self.reti(ram);},
                0xDA => {
                    let lsh = self.aux_read_immediate_data(ram);
                    let msh = self.aux_read_immediate_data(ram);
                    self.jp_flag_pc_16(Flag::FLAG_C, msh, lsh);
                },
                0xDB => {self.invalid_instruction(0xDB);},
                0xDC => {
                    let lsh = self.aux_read_immediate_data(ram);
                    let msh = self.aux_read_immediate_data(ram);
                    self.call_flag_16(ram, Flag::FLAG_C, msh, lsh);
                },
                0xDD => {self.invalid_instruction(0xDD);},
                0xDE => {
                    let num = self.aux_read_immediate_data(ram);
                    self.sbc_r8_8(REG_A, num);
                },
                0xDF => {self.rst(ram, 0x18);},
                0xE0 => {
                    let lsh = self.aux_read_immediate_data(ram);
                    self.ld_16a_r8(ram, 0xFF, lsh, REG_A);
                },
                0xE1 => {self.pop_r16(ram, REG_H, REG_L);},
                0xE2 => {self.ld_16a_r8(ram, 0xFF, self.regs[REG_C], REG_A);},
                0xE3 => {self.invalid_instruction(0xE3);},
                0xE4 => {self.invalid_instruction(0xE4);},
                0xE5 => {self.push_r16(ram, REG_H, REG_L);},
                0xE6 => {
                    let num = self.aux_read_immediate_data(ram);
                    self.and_r8_8(REG_A, num);
                },
                0xE7 => {self.rst(ram, 0x20);},
                0xE8 => {
                    let immediate = self.aux_read_immediate_data(ram) as i8;
                    self.add_sp_i8(immediate);
                },
                0xE9 => {self.jp_pc_16(self.regs[REG_H], self.regs[REG_L]);},
                0xEA => {
                    let lsh = self.aux_read_immediate_data(ram);
                    let msh = self.aux_read_immediate_data(ram);
                    self.ld_16a_r8(ram, msh, lsh, REG_A);
                },
                0xEB => {self.invalid_instruction(0xEB);},
                0xEC => {self.invalid_instruction(0xEC);},
                0xED => {self.invalid_instruction(0xED);},
                0xEE => {
                    let num = self.aux_read_immediate_data(ram);
                    self.xor_r8_8(REG_A, num);
                },
                0xEF => {self.rst(ram, 0x28);},
                0xF0 => {
                    let lsh = self.aux_read_immediate_data(ram);
                    self.ld_r8_16a(ram, REG_A, 0xFF, lsh)
                },
                0xF1 => {self.pop_r16(ram, REG_A, REG_F);},
                0xF2 => {self.ld_r8_16a(ram, REG_A, 0xFF, self.regs[REG_C]);},
                0xF3 => {self.di();},
                0xF4 => {self.invalid_instruction(0xF4);},
                0xF5 => {self.push_r16(ram, REG_A, REG_F);},
                0xF6 => {
                    let num = self.aux_read_immediate_data(ram);
                    self.or_r8_8(REG_A, num);
                },
                0xF7 => {self.rst(ram, 0x30);},
                0xF8 => {
                    let immediate = self.aux_read_immediate_data(ram) as i8;
                    self.ld_hl_sp_plus(immediate);
                },
                0xF9 => {self.ld_sp_r16( REG_H, REG_L);},
                0xFA => {
                    let lsh = self.aux_read_immediate_data(ram);
                    let msh = self.aux_read_immediate_data(ram);
                    self.ld_r8_16a(ram, REG_A, msh, lsh);
                },
                0xFB => {self.ei();},
                0xFC => {self.invalid_instruction(0xFC);},
                0xFD => {self.invalid_instruction(0xFD);},
                0xFE => {
                    let num = self.aux_read_immediate_data(ram);
                    self.cp_r8_8(REG_A, num);
                },
                0xFF => {self.rst(ram, 0x38);}
            }

            self.pc.current_instruction_cycles += ZERO_INSTRUCTION_TIME_TABLE[instruction as usize];
        }
        else
        {
            let cb_instruction = self.aux_read_immediate_data(ram);

            // println!("Instruction: 0x{:02X?}, Program Counter: 0x{:02X?}", cb_instruction, self.pc.reg);

            match cb_instruction //CB Prefix
            {
                0x00 => {self.rlc_r8(REG_B);},
                0x01 => {self.rlc_r8(REG_C);},
                0x02 => {self.rlc_r8(REG_D);},
                0x03 => {self.rlc_r8(REG_E);},
                0x04 => {self.rlc_r8(REG_H);},
                0x05 => {self.rlc_r8(REG_L);},
                0x06 => {self.rlc_r16a(ram, REG_H, REG_L);},
                0x07 => {self.rlc_r8(REG_A);},
                0x08 => {self.rrc_r8(REG_B);},
                0x09 => {self.rrc_r8(REG_C);},
                0x0A => {self.rrc_r8(REG_D);},
                0x0B => {self.rrc_r8(REG_E);},
                0x0C => {self.rrc_r8(REG_H);},
                0x0D => {self.rrc_r8(REG_L);},
                0x0E => {self.rrc_r16a(ram, REG_H, REG_L);},
                0x0F => {self.rrc_r8(REG_A);},
                0x10 => {self.rl_r8(REG_B);},
                0x11 => {self.rl_r8(REG_C);},
                0x12 => {self.rl_r8(REG_D);},
                0x13 => {self.rl_r8(REG_E);},
                0x14 => {self.rl_r8(REG_H);},
                0x15 => {self.rl_r8(REG_L);},
                0x16 => {self.rl_r16a(ram, REG_H, REG_L);},
                0x17 => {self.rl_r8(REG_A);},
                0x18 => {self.rr_r8(REG_B);},
                0x19 => {self.rr_r8(REG_C);},
                0x1A => {self.rr_r8(REG_D);},
                0x1B => {self.rr_r8(REG_E);},
                0x1C => {self.rr_r8(REG_H);},
                0x1D => {self.rr_r8(REG_L);},
                0x1E => {self.rr_r16a(ram, REG_H, REG_L);},
                0x1F => {self.rr_r8(REG_A);},
                0x20 => {self.sla_r8(REG_B);},
                0x21 => {self.sla_r8(REG_C);},
                0x22 => {self.sla_r8(REG_D);},
                0x23 => {self.sla_r8(REG_E);},
                0x24 => {self.sla_r8(REG_H);},
                0x25 => {self.sla_r8(REG_L);},
                0x26 => {self.sla_r16a(ram, REG_H, REG_L);},
                0x27 => {self.sla_r8(REG_A);},
                0x28 => {self.sra_r8(REG_B);},
                0x29 => {self.sra_r8(REG_C);},
                0x2A => {self.sra_r8(REG_D);},
                0x2B => {self.sra_r8(REG_E);},
                0x2C => {self.sra_r8(REG_H);},
                0x2D => {self.sra_r8(REG_L);},
                0x2E => {self.sra_r16a(ram, REG_H, REG_L);},
                0x2F => {self.sra_r8(REG_A);},
                0x30 => {self.swap_r8(REG_B);},
                0x31 => {self.swap_r8(REG_C);},
                0x32 => {self.swap_r8(REG_D);},
                0x33 => {self.swap_r8(REG_E);},
                0x34 => {self.swap_r8(REG_H);},
                0x35 => {self.swap_r8(REG_L);},
                0x36 => {self.swap_r16a(ram, REG_H, REG_L);},
                0x37 => {self.swap_r8(REG_A);},
                0x38 => {self.srl_r8(REG_B);},
                0x39 => {self.srl_r8(REG_C);},
                0x3A => {self.srl_r8(REG_D);},
                0x3B => {self.srl_r8(REG_E);},
                0x3C => {self.srl_r8(REG_H);},
                0x3D => {self.srl_r8(REG_L);},
                0x3E => {self.srl_r16a(ram, REG_H, REG_L);},
                0x3F => {self.srl_r8(REG_A);},
                0x40 => {self.bit_r8(0, REG_B);},
                0x41 => {self.bit_r8(0, REG_C);},
                0x42 => {self.bit_r8(0, REG_D);},
                0x43 => {self.bit_r8(0, REG_E);},
                0x44 => {self.bit_r8(0, REG_H);},
                0x45 => {self.bit_r8(0, REG_L);},
                0x46 => {self.bit_r16a(ram, 0, REG_H, REG_L);},
                0x47 => {self.bit_r8(0, REG_A);},
                0x48 => {self.bit_r8(1, REG_B);},
                0x49 => {self.bit_r8(1, REG_C);},
                0x4A => {self.bit_r8(1, REG_D);},
                0x4B => {self.bit_r8(1, REG_E);},
                0x4C => {self.bit_r8(1, REG_H);},
                0x4D => {self.bit_r8(1, REG_L);},
                0x4E => {self.bit_r16a(ram, 1, REG_H, REG_L);},
                0x4F => {self.bit_r8(1, REG_A);},
                0x50 => {self.bit_r8(2, REG_B);},
                0x51 => {self.bit_r8(2, REG_C);},
                0x52 => {self.bit_r8(2, REG_D);},
                0x53 => {self.bit_r8(2, REG_E);},
                0x54 => {self.bit_r8(2, REG_H);},
                0x55 => {self.bit_r8(2, REG_L);},
                0x56 => {self.bit_r16a(ram, 2, REG_H, REG_L);},
                0x57 => {self.bit_r8(2, REG_A);},
                0x58 => {self.bit_r8(3, REG_B);},
                0x59 => {self.bit_r8(3, REG_C);},
                0x5A => {self.bit_r8(3, REG_D);},
                0x5B => {self.bit_r8(3, REG_E);},
                0x5C => {self.bit_r8(3, REG_H);},
                0x5D => {self.bit_r8(3, REG_L);},
                0x5E => {self.bit_r16a(ram, 3, REG_H, REG_L);},
                0x5F => {self.bit_r8(3, REG_A);},
                0x60 => {self.bit_r8(4, REG_B);},
                0x61 => {self.bit_r8(4, REG_C);},
                0x62 => {self.bit_r8(4, REG_D);},
                0x63 => {self.bit_r8(4, REG_E);},
                0x64 => {self.bit_r8(4, REG_H);},
                0x65 => {self.bit_r8(4, REG_L);},
                0x66 => {self.bit_r16a(ram, 4, REG_H, REG_L);},
                0x67 => {self.bit_r8(4, REG_A);},
                0x68 => {self.bit_r8(5, REG_B);},
                0x69 => {self.bit_r8(5, REG_C);},
                0x6A => {self.bit_r8(5, REG_D);},
                0x6B => {self.bit_r8(5, REG_E);},
                0x6C => {self.bit_r8(5, REG_H);},
                0x6D => {self.bit_r8(5, REG_L);},
                0x6E => {self.bit_r16a(ram, 5, REG_H, REG_L);},
                0x6F => {self.bit_r8(5, REG_A);},
                0x70 => {self.bit_r8(6, REG_B);},
                0x71 => {self.bit_r8(6, REG_C);},
                0x72 => {self.bit_r8(6, REG_D);},
                0x73 => {self.bit_r8(6, REG_E);},
                0x74 => {self.bit_r8(6, REG_H);},
                0x75 => {self.bit_r8(6, REG_L);},
                0x76 => {self.bit_r16a(ram, 6, REG_H, REG_L);},
                0x77 => {self.bit_r8(6, REG_A);},
                0x78 => {self.bit_r8(7, REG_B);},
                0x79 => {self.bit_r8(7, REG_C);},
                0x7A => {self.bit_r8(7, REG_D);},
                0x7B => {self.bit_r8(7, REG_E);},
                0x7C => {self.bit_r8(7, REG_H);},
                0x7D => {self.bit_r8(7, REG_L);},
                0x7E => {self.bit_r16a(ram, 7, REG_H, REG_L);},
                0x7F => {self.bit_r8(7, REG_A);},
                0x80 => {self.res_r8(0, REG_B);},
                0x81 => {self.res_r8(0, REG_C);},
                0x82 => {self.res_r8(0, REG_D);},
                0x83 => {self.res_r8(0, REG_E);},
                0x84 => {self.res_r8(0, REG_H);},
                0x85 => {self.res_r8(0, REG_L);},
                0x86 => {self.res_r16a(ram, 0, REG_H, REG_L);},
                0x87 => {self.res_r8(0, REG_A);},
                0x88 => {self.res_r8(1, REG_B);},
                0x89 => {self.res_r8(1, REG_C);},
                0x8A => {self.res_r8(1, REG_D);},
                0x8B => {self.res_r8(1, REG_E);},
                0x8C => {self.res_r8(1, REG_H);},
                0x8D => {self.res_r8(1, REG_L);},
                0x8E => {self.res_r16a(ram, 1, REG_H, REG_L);},
                0x8F => {self.res_r8(1, REG_A);},
                0x90 => {self.res_r8(2, REG_B);},
                0x91 => {self.res_r8(2, REG_C);},
                0x92 => {self.res_r8(2, REG_D);},
                0x93 => {self.res_r8(2, REG_E);},
                0x94 => {self.res_r8(2, REG_H);},
                0x95 => {self.res_r8(2, REG_L);},
                0x96 => {self.res_r16a(ram, 2, REG_H, REG_L);},
                0x97 => {self.res_r8(2, REG_A);},
                0x98 => {self.res_r8(3, REG_B);},
                0x99 => {self.res_r8(3, REG_C);},
                0x9A => {self.res_r8(3, REG_D);},
                0x9B => {self.res_r8(3, REG_E);},
                0x9C => {self.res_r8(3, REG_H);},
                0x9D => {self.res_r8(3, REG_L);},
                0x9E => {self.res_r16a(ram, 3, REG_H, REG_L);},
                0x9F => {self.res_r8(3, REG_A);},
                0xA0 => {self.res_r8(4, REG_B);},
                0xA1 => {self.res_r8(4, REG_C);},
                0xA2 => {self.res_r8(4, REG_D);},
                0xA3 => {self.res_r8(4, REG_E);},
                0xA4 => {self.res_r8(4, REG_H);},
                0xA5 => {self.res_r8(4, REG_L);},
                0xA6 => {self.res_r16a(ram, 4, REG_H, REG_L);},
                0xA7 => {self.res_r8(4, REG_A);},
                0xA8 => {self.res_r8(5, REG_B);},
                0xA9 => {self.res_r8(5, REG_C);},
                0xAA => {self.res_r8(5, REG_D);},
                0xAB => {self.res_r8(5, REG_E);},
                0xAC => {self.res_r8(5, REG_H);},
                0xAD => {self.res_r8(5, REG_L);},
                0xAE => {self.res_r16a(ram, 5, REG_H, REG_L);},
                0xAF => {self.res_r8(5, REG_A);},
                0xB0 => {self.res_r8(6, REG_B);},
                0xB1 => {self.res_r8(6, REG_C);},
                0xB2 => {self.res_r8(6, REG_D);},
                0xB3 => {self.res_r8(6, REG_E);},
                0xB4 => {self.res_r8(6, REG_H);},
                0xB5 => {self.res_r8(6, REG_L);},
                0xB6 => {self.res_r16a(ram, 6, REG_H, REG_L);},
                0xB7 => {self.res_r8(6, REG_A);},
                0xB8 => {self.res_r8(7, REG_B);},
                0xB9 => {self.res_r8(7, REG_C);},
                0xBA => {self.res_r8(7, REG_D);},
                0xBB => {self.res_r8(7, REG_E);},
                0xBC => {self.res_r8(7, REG_H);},
                0xBD => {self.res_r8(7, REG_L);},
                0xBE => {self.res_r16a(ram, 7, REG_H, REG_L);},
                0xBF => {self.res_r8(7, REG_A);},
                0xC0 => {self.set_r8(0, REG_B);},
                0xC1 => {self.set_r8(0, REG_C);},
                0xC2 => {self.set_r8(0, REG_D);},
                0xC3 => {self.set_r8(0, REG_E);},
                0xC4 => {self.set_r8(0, REG_H);},
                0xC5 => {self.set_r8(0, REG_L);},
                0xC6 => {self.set_r16a(ram, 0, REG_H, REG_L);},
                0xC7 => {self.set_r8(0, REG_A);},
                0xC8 => {self.set_r8(1, REG_B);},
                0xC9 => {self.set_r8(1, REG_C);},
                0xCA => {self.set_r8(1, REG_D);},
                0xCB => {self.set_r8(1, REG_E);},
                0xCC => {self.set_r8(1, REG_H);},
                0xCD => {self.set_r8(1, REG_L);},
                0xCE => {self.set_r16a(ram, 1, REG_H, REG_L);},
                0xCF => {self.set_r8(1, REG_A);},
                0xD0 => {self.set_r8(2, REG_B);},
                0xD1 => {self.set_r8(2, REG_C);},
                0xD2 => {self.set_r8(2, REG_D);},
                0xD3 => {self.set_r8(2, REG_E);},
                0xD4 => {self.set_r8(2, REG_H);},
                0xD5 => {self.set_r8(2, REG_L);},
                0xD6 => {self.set_r16a(ram, 2, REG_H, REG_L);},
                0xD7 => {self.set_r8(2, REG_A);},
                0xD8 => {self.set_r8(3, REG_B);},
                0xD9 => {self.set_r8(3, REG_C);},
                0xDA => {self.set_r8(3, REG_D);},
                0xDB => {self.set_r8(3, REG_E);},
                0xDC => {self.set_r8(3, REG_H);},
                0xDD => {self.set_r8(3, REG_L);},
                0xDE => {self.set_r16a(ram, 3, REG_H, REG_L);},
                0xDF => {self.set_r8(3, REG_A);},
                0xE0 => {self.set_r8(4, REG_B);},
                0xE1 => {self.set_r8(4, REG_C);},
                0xE2 => {self.set_r8(4, REG_D);},
                0xE3 => {self.set_r8(4, REG_E);},
                0xE4 => {self.set_r8(4, REG_H);},
                0xE5 => {self.set_r8(4, REG_L);},
                0xE6 => {self.set_r16a(ram, 4, REG_H, REG_L);},
                0xE7 => {self.set_r8(4, REG_A);},
                0xE8 => {self.set_r8(5, REG_B);},
                0xE9 => {self.set_r8(5, REG_C);},
                0xEA => {self.set_r8(5, REG_D);},
                0xEB => {self.set_r8(5, REG_E);},
                0xEC => {self.set_r8(5, REG_H);},
                0xED => {self.set_r8(5, REG_L);},
                0xEE => {self.set_r16a(ram, 5, REG_H, REG_L);},
                0xEF => {self.set_r8(5, REG_A);},
                0xF0 => {self.set_r8(6, REG_B);},
                0xF1 => {self.set_r8(6, REG_C);},
                0xF2 => {self.set_r8(6, REG_D);},
                0xF3 => {self.set_r8(6, REG_E);},
                0xF4 => {self.set_r8(6, REG_H);},
                0xF5 => {self.set_r8(6, REG_L);},
                0xF6 => {self.set_r16a(ram, 6, REG_H, REG_L);},
                0xF7 => {self.set_r8(6, REG_A);},
                0xF8 => {self.set_r8(7, REG_B);},
                0xF9 => {self.set_r8(7, REG_C);},
                0xFA => {self.set_r8(7, REG_D);},
                0xFB => {self.set_r8(7, REG_E);},
                0xFC => {self.set_r8(7, REG_H);},
                0xFD => {self.set_r8(7, REG_L);},
                0xFE => {self.set_r16a(ram, 7, REG_H, REG_L);},
                0xFF => {self.set_r8(7, REG_A);}
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
    #[allow(dead_code)]
    fn aux_get_reg(&self, regnum: usize) -> u8 { self.regs[regnum] }
    #[allow(dead_code)]
    fn aux_get_sp(&self) -> u16 { self.sp }
    #[allow(dead_code)]
    fn aux_get_pc(&self) -> ProgramCounter { self.pc }

}