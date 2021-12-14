use std::ops::RemAssign;

use super::ram::Ram;

use usize as Reg;
const REG_A:Reg = 0;
const REG_B:Reg = 1;
const REG_C:Reg = 2;
const REG_D:Reg = 3;
const REG_E:Reg = 4;
const REG_F:Reg = 5;
const REG_H:Reg = 6;
const REG_L:Reg = 7;

const REG_SP:Reg = 0;
const REG_PC:Reg = 1;
//in an AF situation, A is msh, F is lsh, little endian

use u8 as Flag;
const FLAG_Z:Flag = 7;
const FLAG_N:Flag = 6;
const FLAG_H:Flag = 5;
const FLAG_C:Flag = 4;

pub struct Cpu
{
    regs: [u8;8],
    pcsp_regs: [u16;2],
    ram: Ram
}

impl Cpu
{
    pub fn new() -> Cpu
    {
        return Cpu { regs: [0;8], pcsp_regs: [0;2], ram: Ram::new() }
    }
    //Format [name]_[param1]_[param2]
    //r is a register
    //pcsp is a special register like sp which is a 16 bit variant register
    //a means parameter is an address (dereference)

    fn aux_read_flag(&self, param: Flag) -> bool
    {
        (self.regs[REG_F].to_le() & u8::to_le(1 << param)) > 0
    }

    fn aux_write_flag(&mut self, param: Flag, data: bool)
    {
        let x = data as u8;
        assert!(x == 0 || x == 1);
        self.regs[REG_F] = self.regs[REG_F].to_le() & u8::to_le(!(!x << param))
    }

    fn ld_r16_16(&mut self, msh: Reg, lsh: Reg, p2: u16)
    {
        let hl = p2.to_le_bytes();
        self.regs[msh] = hl[0];
        self.regs[lsh] = hl[1];
    }

    fn ld_r16a_8(&mut self, msh: Reg, lsh: Reg, p2: u8)
    {
        self.ram.write_to_address(u16::from_le_bytes([self.regs[msh], self.regs[lsh]]) as usize, p2);
    }

    fn ld_r8_8(&mut self, p1: Reg, p2: u8)
    {
        self.regs[p1] = p2;
    }

    fn ld_r8_r8(&mut self, p1: Reg, p2: Reg)
    {
        self.regs[p1] = self.regs[p2];
    }

    fn ld_r8_r16a(&mut self, p1: Reg, msh: Reg, lsh: Reg)
    {
        let x = self.ram.read_from_address_rp(self.regs[msh], self.regs[lsh]);
        self.regs[p1] = x;
    }

    fn ld_r16a_r8(&mut self, msh: Reg, lsh: Reg, p2: Reg)
    {
        self.ram.write_to_address_rp(self.regs[msh], self.regs[lsh], self.regs[p2]);
    }

    // TODO: See if the flags are modified
    fn inc_r16(&mut self, msh: Reg, lsh: Reg)
    {
        let lsh_result = u8::overflowing_add(self.regs[lsh], 1);
        self.regs[lsh] = lsh_result.0;
        let msh_result = u8::overflowing_add(self.regs[msh], lsh_result.1 as u8);
        self.regs[msh] = msh_result.0;
    }

    fn inc_r16pcsp(&mut self, reg: Reg)
    {
        let result = u16::overflowing_add(self.pcsp_regs[reg], 1);
        self.pcsp_regs[reg] = result.0;
    }

    fn inc_r8(&mut self, reg: Reg)
    {
        let result = u8::overflowing_add(self.regs[reg], 1);
        self.regs[reg] = result.0;
        
        self.aux_write_flag(FLAG_Z, result.0 == 0);
        self.aux_write_flag(FLAG_N, false);
        self.aux_write_flag(FLAG_H, result.1);
    }

    fn inc_r8a(&mut self, msh: Reg, lsh: Reg)
    {
        let result = self.ram.read_from_address_rp(
            self.regs[msh], self.regs[lsh]).overflowing_add(1);
        self.ram.write_to_address_rp(self.regs[msh], self.regs[lsh], result.0);
        
        self.aux_write_flag(FLAG_Z, result.0 == 0);
        self.aux_write_flag(FLAG_N, false);
        self.aux_write_flag(FLAG_H, result.1);
    }

    fn dec_r8(&mut self, reg: Reg)
    {
        let result = u8::overflowing_sub(self.regs[reg], 1);
        self.regs[reg] = result.0;
        
        self.aux_write_flag(FLAG_Z, result.0 == 0);
        self.aux_write_flag(FLAG_N, true);
        self.aux_write_flag(FLAG_H, result.1);
    }

    fn dec_r8a(&mut self, msh: Reg, lsh: Reg)
    {
        let result = self.ram.read_from_address_rp(
            self.regs[msh], self.regs[lsh]).overflowing_sub(1);
        self.ram.write_to_address_rp(self.regs[msh], self.regs[lsh], result.0);
        
        self.aux_write_flag(FLAG_Z, result.0 == 0);
        self.aux_write_flag(FLAG_N, true);
        self.aux_write_flag(FLAG_H, result.1);
    }

    fn dec_r16(&mut self, msh: Reg, lsh: Reg)
    {
        let lsh_result = u8::overflowing_sub(self.regs[lsh], 1);
        self.regs[lsh] = lsh_result.0;
        let msh_result = u8::overflowing_sub(self.regs[msh], lsh_result.1 as u8);
        self.regs[msh] = msh_result.0;
    }

    fn dec_r16pcsp(&mut self, reg: Reg)
    {
        let result = u16::overflowing_sub(self.pcsp_regs[reg], 1);
        self.pcsp_regs[reg] = result.0;
    }

    fn add_r8_r8(&mut self, p1: Reg, p2: Reg)
    {
        let half_carry_pre = ((self.regs[p1] ^ self.regs[p2]) >> 4) & 1;
        let result = self.regs[p1].overflowing_add(self.regs[p2]);
        self.regs[p1] = result.0;
        let half_carry_post = (result.0 >> 4) & 1;
        
        self.aux_write_flag(FLAG_Z, result.0 == 0);
        self.aux_write_flag(FLAG_H, half_carry_pre != half_carry_post);
        self.aux_write_flag(FLAG_N, false);
        self.aux_write_flag(FLAG_C, result.1);
    }

    fn add_r8_r8a(&mut self, p1: Reg, msh: Reg, lsh: Reg)
    {
        let p2 = self.ram.read_from_address_rp(self.regs[msh], self.regs[lsh]);
        let half_carry_pre = ((self.regs[p1] ^ p2) >> 4) & 1;
        let result = self.regs[p1].overflowing_add(p2);
        self.regs[p1] = result.0;
        let half_carry_post = (result.0 >> 4) & 1;
        
        self.aux_write_flag(FLAG_Z, result.0 == 0);
        self.aux_write_flag(FLAG_H, half_carry_pre == half_carry_post);
        self.aux_write_flag(FLAG_N, false);
        self.aux_write_flag(FLAG_C, result.1);
    }

    fn adc_r8_r8(&mut self, p1: Reg, p2: Reg)
    {
        let carry = self.aux_read_flag(FLAG_C) as u8;
        let half_carry_pre1 = ((self.regs[p1] ^ self.regs[p2]) >> 4) & 1;
        let result1 = self.regs[p1].overflowing_add(self.regs[p2]);
        let half_carry_post1 = (result1.0 >> 4) & 1;
        let half_carry_pre2 = ((result1.0 ^ carry) >> 4) & 1;
        let result2 = result1.0.overflowing_add(carry);
        self.regs[p1] = result2.0;
        let half_carry_post2 = (result2.0 >> 4) & 1;
        
        self.aux_write_flag(FLAG_Z, result2.0 == 0);
        self.aux_write_flag(FLAG_H, half_carry_pre1 != half_carry_post1 || half_carry_pre2 != half_carry_post2);
        self.aux_write_flag(FLAG_N, false);
        self.aux_write_flag(FLAG_C, result1.1 || result2.1);
    }

    fn adc_r8_r8a(&mut self, p1: Reg, msh: Reg, lsh: Reg)
    {
        let carry = self.aux_read_flag(FLAG_C) as u8;
        let p2 = self.ram.read_from_address_rp(self.regs[msh], self.regs[lsh]);
        let half_carry_pre1 = ((self.regs[p1] ^ p2) >> 4) & 1;
        let result1 = self.regs[p1].overflowing_add(p2);
        let half_carry_post1 = (result1.0 >> 4) & 1;
        let half_carry_pre2 = ((result1.0 ^ carry) >> 4) & 1;
        let result2 = result1.0.overflowing_add(carry);
        self.regs[p1] = result2.0;
        let half_carry_post2 = (result2.0 >> 4) & 1;
        
        self.aux_write_flag(FLAG_Z, result2.0 == 0);
        self.aux_write_flag(FLAG_H, half_carry_pre1 != half_carry_post1 || half_carry_pre2 != half_carry_post2);
        self.aux_write_flag(FLAG_N, false);
        self.aux_write_flag(FLAG_C, result1.1 || result2.1);
    }

    //TODO: Check subtraction half carry calculations
    fn sub_r8_r8(&mut self, p1: Reg, p2: Reg)
    {
        let half_carry_pre = ((self.regs[p1] ^ self.regs[p2]) >> 4) & 1;
        let result = self.regs[p1].overflowing_sub(self.regs[p2]);
        self.regs[p1] = result.0;
        let half_carry_post = (result.0 >> 4) & 1;
        
        self.aux_write_flag(FLAG_Z, result.0 == 0);
        self.aux_write_flag(FLAG_H, half_carry_pre != half_carry_post);
        self.aux_write_flag(FLAG_N, true);
        self.aux_write_flag(FLAG_C, result.1);
    }

    fn sub_r8_r8a(&mut self, p1: Reg, msh: Reg, lsh: Reg)
    {
        let p2 = self.ram.read_from_address_rp(self.regs[msh], self.regs[lsh]);
        let half_carry_pre = ((self.regs[p1] ^ p2) >> 4) & 1;
        let result = self.regs[p1].overflowing_add(p2);
        self.regs[p1] = result.0;
        let half_carry_post = (result.0 >> 4) & 1;
        
        self.aux_write_flag(FLAG_Z, result.0 == 0);
        self.aux_write_flag(FLAG_H, half_carry_pre != half_carry_post);
        self.aux_write_flag(FLAG_N, true);
        self.aux_write_flag(FLAG_C, result.1);
    }

    fn sbc_r8_r8(&mut self, p1: Reg, p2: Reg)
    {
        let carry = self.aux_read_flag(FLAG_C) as u8;
        let half_carry_pre1 = ((self.regs[p1] ^ self.regs[p2]) >> 4) & 1;
        let result1 = self.regs[p1].overflowing_sub(self.regs[p2]);
        let half_carry_post1 = (result1.0 >> 4) & 1;
        let half_carry_pre2 = ((result1.0 ^ carry) >> 4) & 1;
        let result2 = result1.0.overflowing_sub(carry);
        self.regs[p1] = result2.0;
        let half_carry_post2 = (result2.0 >> 4) & 1;
        
        self.aux_write_flag(FLAG_Z, result2.0 == 0);
        self.aux_write_flag(FLAG_H, half_carry_pre1 != half_carry_post1 || half_carry_pre2 != half_carry_post2);
        self.aux_write_flag(FLAG_N, true);
        self.aux_write_flag(FLAG_C, result1.1 || result2.1);
    }

    fn sbc_r8_r8a(&mut self, p1: Reg, msh: Reg, lsh: Reg)
    {
        let carry = self.aux_read_flag(FLAG_C) as u8;
        let p2 = self.ram.read_from_address_rp(self.regs[msh], self.regs[lsh]);
        let half_carry_pre1 = ((self.regs[p1] ^ p2) >> 4) & 1;
        let result1 = self.regs[p1].overflowing_sub(p2);
        let half_carry_post1 = (result1.0 >> 4) & 1;
        let half_carry_pre2 = ((result1.0 ^ carry) >> 4) & 1;
        let result2 = result1.0.overflowing_sub(carry);
        self.regs[p1] = result2.0;
        let half_carry_post2 = (result2.0 >> 4) & 1;
        
        self.aux_write_flag(FLAG_Z, result2.0 == 0);
        self.aux_write_flag(FLAG_H, half_carry_pre1 != half_carry_post1 || half_carry_pre2 != half_carry_post2);
        self.aux_write_flag(FLAG_N, true);
        self.aux_write_flag(FLAG_C, result1.1 || result2.1);
    }

    fn and_r8_r8(&mut self, p1: Reg, p2: Reg)
    {
        self.regs[p1] = self.regs[p1] & self.regs[p2];

        self.aux_write_flag(FLAG_Z, self.regs[p1] == 0);
        self.aux_write_flag(FLAG_H, true);
        self.aux_write_flag(FLAG_N, false);
        self.aux_write_flag(FLAG_C, false);
    }

    fn and_r8_r8a(&mut self, p1: Reg, msh: Reg, lsh: Reg)
    {
        self.regs[p1] = self.regs[p1] & self.ram.read_from_address_rp(self.regs[msh], self.regs[lsh]);

        self.aux_write_flag(FLAG_Z, self.regs[p1] == 0);
        self.aux_write_flag(FLAG_H, true);
        self.aux_write_flag(FLAG_N, false);
        self.aux_write_flag(FLAG_C, false);
    }

    fn xor_r8_r8(&mut self, p1: Reg, p2: Reg)
    {
        self.regs[p1] = self.regs[p1] ^ self.regs[p2];

        self.aux_write_flag(FLAG_Z, self.regs[p1] == 0);
        self.aux_write_flag(FLAG_H, false);
        self.aux_write_flag(FLAG_N, false);
        self.aux_write_flag(FLAG_C, false);
    }

    fn xor_r8_r8a(&mut self, p1: Reg, msh: Reg, lsh: Reg)
    {
        self.regs[p1] = self.regs[p1] ^ self.ram.read_from_address_rp(self.regs[msh], self.regs[lsh]);

        self.aux_write_flag(FLAG_Z, self.regs[p1] == 0);
        self.aux_write_flag(FLAG_H, false);
        self.aux_write_flag(FLAG_N, false);
        self.aux_write_flag(FLAG_C, false);
    }

    fn or_r8_r8(&mut self, p1: Reg, p2: Reg)
    {
        self.regs[p1] = self.regs[p1] | self.regs[p2];

        self.aux_write_flag(FLAG_Z, self.regs[p1] == 0);
        self.aux_write_flag(FLAG_H, false);
        self.aux_write_flag(FLAG_N, false);
        self.aux_write_flag(FLAG_C, false);
    }

    fn or_r8_r8a(&mut self, p1: Reg, msh: Reg, lsh: Reg)
    {
        self.regs[p1] = self.regs[p1] | self.ram.read_from_address_rp(self.regs[msh], self.regs[lsh]);

        self.aux_write_flag(FLAG_Z, self.regs[p1] == 0);
        self.aux_write_flag(FLAG_H, false);
        self.aux_write_flag(FLAG_N, false);
        self.aux_write_flag(FLAG_C, false);
    }

    fn cp_r8_r8(&mut self, p1: Reg, p2: Reg)
    {
        let half_carry_pre = ((self.regs[p1] ^ self.regs[p2]) >> 4) & 1;
        let result = self.regs[p1].overflowing_sub(self.regs[p2]);
        self.regs[p1] = result.0;
        let half_carry_post = (result.0 >> 4) & 1;
        
        self.aux_write_flag(FLAG_Z, result.0 == 0);
        self.aux_write_flag(FLAG_H, half_carry_pre != half_carry_post);
        self.aux_write_flag(FLAG_N, true);
        self.aux_write_flag(FLAG_C, result.1);
    }

    fn cp_r8_r8a(&mut self, p1: Reg, msh: Reg, lsh: Reg)
    {
        let p2 = self.ram.read_from_address_rp(self.regs[msh], self.regs[lsh]);
        let half_carry_pre = ((self.regs[p1] ^ p2) >> 4) & 1;
        let result = self.regs[p1].overflowing_add(p2);
        self.regs[p1] = result.0;
        let half_carry_post = (result.0 >> 4) & 1;
        
        self.aux_write_flag(FLAG_Z, result.0 == 0);
        self.aux_write_flag(FLAG_H, half_carry_pre != half_carry_post);
        self.aux_write_flag(FLAG_N, true);
        self.aux_write_flag(FLAG_C, result.1);
    }

    //--------------------16 BIT OPCODES--------------------

    fn rlc_r8(&mut self, p1: Reg)
    {
        self.aux_write_flag(FLAG_C, (self.regs[p1] >> 7) & 1 != 0);
        self.regs[p1] = self.regs[p1].rotate_left(1);
        
        self.aux_write_flag(FLAG_Z, self.regs[p1] == 0);
        self.aux_write_flag(FLAG_N, false);
        self.aux_write_flag(FLAG_H, false);
    }

    fn rlc_r8a(&mut self, msh: Reg, lsh: Reg)
    {
        let p1 = self.ram.read_from_address_rp(self.regs[msh], self.regs[lsh]);
        self.aux_write_flag(FLAG_C, (p1 >> 7) & 1 != 0);
        let result = p1.rotate_left(1);
        self.ram.write_to_address_rp(self.regs[msh], self.regs[lsh], result);
        
        self.aux_write_flag(FLAG_Z, result == 0);
        self.aux_write_flag(FLAG_N, false);
        self.aux_write_flag(FLAG_H, false);
    }

    fn rrc_r8(&mut self, p1: Reg)
    {
        self.aux_write_flag(FLAG_C, self.regs[p1] & 1 != 0);
        self.regs[p1] = self.regs[p1].rotate_right(1);
        
        self.aux_write_flag(FLAG_Z, self.regs[p1] == 0);
        self.aux_write_flag(FLAG_N, false);
        self.aux_write_flag(FLAG_H, false);
    }

    fn rrc_r8a(&mut self, msh: Reg, lsh: Reg)
    {
        let p1 = self.ram.read_from_address_rp(self.regs[msh], self.regs[lsh]);
        self.aux_write_flag(FLAG_C, p1 & 1 != 0);
        let result = p1.rotate_right(1);
        self.ram.write_to_address_rp(self.regs[msh], self.regs[lsh], result);
        
        self.aux_write_flag(FLAG_Z, result == 0);
        self.aux_write_flag(FLAG_N, false);
        self.aux_write_flag(FLAG_H, false);
    }

    fn rl_r8(&mut self, p1: Reg)
    {
        let cin = self.aux_read_flag(FLAG_C) as u8;
        self.aux_write_flag(FLAG_C, (self.regs[p1] >> 7) & 1 != 0);
        self.regs[p1] = (self.regs[p1] << 1u8) | cin;
        
        self.aux_write_flag(FLAG_Z, self.regs[p1] == 0);
        self.aux_write_flag(FLAG_N, false);
        self.aux_write_flag(FLAG_H, false);
    }

    fn rl_r8a(&mut self, msh: Reg, lsh: Reg)
    {
        let cin = self.aux_read_flag(FLAG_C) as u8;
        let p1 = self.ram.read_from_address_rp(self.regs[msh], self.regs[lsh]);
        self.aux_write_flag(FLAG_C, (p1 >> 7) & 1 != 0);
        let result = (p1 << 1u8) | cin;
        self.ram.write_to_address_rp(self.regs[msh], self.regs[lsh], result);
        
        self.aux_write_flag(FLAG_Z, result == 0);
        self.aux_write_flag(FLAG_N, false);
        self.aux_write_flag(FLAG_H, false);
    }

    fn rr_r8(&mut self, p1: Reg)
    {
        let cin = self.aux_read_flag(FLAG_C) as u8;
        self.aux_write_flag(FLAG_C, self.regs[p1] & 1 != 0);
        self.regs[p1] = (self.regs[p1] >> 1u8) | (cin << 7u8);
        
        self.aux_write_flag(FLAG_Z, self.regs[p1] == 0);
        self.aux_write_flag(FLAG_N, false);
        self.aux_write_flag(FLAG_H, false);
    }

    fn rr_r8a(&mut self, msh: Reg, lsh: Reg)
    {
        let cin = self.aux_read_flag(FLAG_C) as u8;
        let p1 = self.ram.read_from_address_rp(self.regs[msh], self.regs[lsh]);
        self.aux_write_flag(FLAG_C, p1 & 1 != 0);
        let result = (p1 >> 1u8) | (cin << 7u8);
        self.ram.write_to_address_rp(self.regs[msh], self.regs[lsh], result);
        
        self.aux_write_flag(FLAG_Z, result == 0);
        self.aux_write_flag(FLAG_N, false);
        self.aux_write_flag(FLAG_H, false);
    }

    fn sla_r8(&mut self, p1: Reg)
    {
        self.aux_write_flag(FLAG_C, (self.regs[p1] >> 7) & 1 != 0);
        self.regs[p1] = self.regs[p1] << 1u8;
        
        self.aux_write_flag(FLAG_Z, self.regs[p1] == 0);
        self.aux_write_flag(FLAG_N, false);
        self.aux_write_flag(FLAG_H, false);
    }

    fn sla_r8a(&mut self, msh: Reg, lsh: Reg)
    {
        let p1 = self.ram.read_from_address_rp(self.regs[msh], self.regs[lsh]);
        self.aux_write_flag(FLAG_C, (p1 >> 7) & 1 != 0);
        let result = p1 << 1u8;
        self.ram.write_to_address_rp(self.regs[msh], self.regs[lsh], result);
        
        self.aux_write_flag(FLAG_Z, result == 0);
        self.aux_write_flag(FLAG_N, false);
        self.aux_write_flag(FLAG_H, false);
    }

    fn sra_r8(&mut self, p1: Reg)
    {
        self.aux_write_flag(FLAG_C, self.regs[p1] & 1 != 0);
        self.regs[p1] = (self.regs[p1] >> 1u8) | (self.regs[p1] & 0b10000000u8); //fill with leftmost
        
        self.aux_write_flag(FLAG_Z, self.regs[p1] == 0);
        self.aux_write_flag(FLAG_N, false);
        self.aux_write_flag(FLAG_H, false);
    }

    fn sra_r8a(&mut self, msh: Reg, lsh: Reg)
    {
        let p1 = self.ram.read_from_address_rp(self.regs[msh], self.regs[lsh]);
        self.aux_write_flag(FLAG_C, p1 & 1 != 0);
        let result =( p1 >> 1u8) | (p1 | 0b10000000u8);
        self.ram.write_to_address_rp(self.regs[msh], self.regs[lsh], result);
        
        self.aux_write_flag(FLAG_Z, result == 0);
        self.aux_write_flag(FLAG_N, false);
        self.aux_write_flag(FLAG_H, false);
    }

    fn srl_r8(&mut self, p1: Reg)
    {
        self.aux_write_flag(FLAG_C, self.regs[p1] & 1 != 0);
        self.regs[p1] = self.regs[p1] >> 1u8; //fill with leftmost
        
        self.aux_write_flag(FLAG_Z, self.regs[p1] == 0);
        self.aux_write_flag(FLAG_N, false);
        self.aux_write_flag(FLAG_H, false);
    }

    fn srl_r8a(&mut self, msh: Reg, lsh: Reg)
    {
        let p1 = self.ram.read_from_address_rp(self.regs[msh], self.regs[lsh]);
        self.aux_write_flag(FLAG_C, p1 & 1 != 0);
        let result = p1 >> 1u8;
        self.ram.write_to_address_rp(self.regs[msh], self.regs[lsh], result);
        
        self.aux_write_flag(FLAG_Z, result == 0);
        self.aux_write_flag(FLAG_N, false);
        self.aux_write_flag(FLAG_H, false);
    }

    fn swap_r8(&mut self, p1: Reg)
    {
        let lower_to_upper_half = self.regs[p1] << 4u8;
        let upper_to_lower_half = self.regs[p1] >> 4u8;
        self.regs[p1] = lower_to_upper_half | upper_to_lower_half;
    }

    fn swap_r8a(&mut self, msh: Reg, lsh: Reg)
    {
        let p1 = self.ram.read_from_address_rp(self.regs[msh], self.regs[lsh]);
        let lower_to_upper_half = p1 << 4u8;
        let upper_to_lower_half = p1 >> 4u8;
        self.ram.write_to_address_rp(self.regs[msh], self.regs[lsh], lower_to_upper_half | upper_to_lower_half);
    }

    fn bit_r8(&mut self, p1: u8, p2: Reg)
    {
        self.aux_write_flag(FLAG_H, true);
        self.aux_write_flag(FLAG_N, false);
        self.aux_write_flag(FLAG_Z, (self.regs[p2] & (1u8 << p1)) == 0);
    }

    fn bit_r8a(&mut self, p1: u8, msh: Reg, lsh: Reg)
    {
        self.aux_write_flag(FLAG_H, true);
        self.aux_write_flag(FLAG_N, false);
        self.aux_write_flag(FLAG_Z, 
            (self.ram.read_from_address_rp(self.regs[msh], self.regs[lsh]) & (1u8 << p1)) == 0);
    }

    fn res_r8(&mut self, p1: u8, p2: Reg)
    {
        self.regs[p2] = self.regs[p2] & (!(1u8 << p1));
    }

    fn res_r8a(&mut self, p1: u8, msh: Reg, lsh: Reg)
    {
        self.ram.write_to_address_rp(self.regs[msh], self.regs[lsh],
            self.ram.read_from_address_rp(self.regs[msh], self.regs[lsh]) & (!(1u8 << p1)));
    }

    fn set_r8(&mut self, p1: u8, p2: Reg)
    {
        self.regs[p2] = self.regs[p2] | (1u8 << p1);
    }

    fn set_r8a(&mut self, p1: u8, msh: Reg, lsh: Reg)
    {
        self.ram.write_to_address_rp(self.regs[msh], self.regs[lsh],
            self.ram.read_from_address_rp(self.regs[msh], self.regs[lsh]) | (1u8 << p1));
    }


    pub fn execute(&mut self)
    {
        match self.pcsp_regs[REG_PC]
        {
            0x00 => {/* NOP */},
            0x01 => {},
            0x02 => {self.ld_r16a_r8(REG_B, REG_C, REG_A)},
            0x03 => {self.inc_r16(REG_B, REG_C)},
            0x04 => {self.inc_r8(REG_B)},
            0x05 => {self.dec_r8(REG_B)},
            0x06 => {},
            0x07 => {},
            0x08 => {},
            0x09 => {},
            0x0A => {self.ld_r8_r16a(REG_A, REG_B, REG_C)},
            0x0B => {self.dec_r16(REG_B, REG_C)},
            0x0C => {self.inc_r8(REG_C)},
            0x0D => {self.dec_r8(REG_C)},
            0x0E => {},
            0x0F => {},
            0x10 => {},
            0x11 => {},
            0x12 => {self.ld_r16a_r8(REG_D, REG_E, REG_A)},
            0x13 => {self.inc_r16(REG_D, REG_E)},
            0x14 => {self.inc_r8(REG_D)},
            0x15 => {self.dec_r8(REG_D)},
            0x16 => {},
            0x17 => {},
            0x18 => {},
            0x19 => {},
            0x1A => {self.ld_r8_r16a(REG_A, REG_D, REG_E)},
            0x1B => {self.dec_r16(REG_D, REG_E)},
            0x1C => {self.inc_r8(REG_E)},
            0x1D => {self.dec_r8(REG_E)},
            0x1E => {},
            0x1F => {},
            0x20 => {},
            0x21 => {},
            0x22 => {},
            0x23 => {self.inc_r16(REG_H, REG_L)},
            0x24 => {self.inc_r8(REG_H)},
            0x25 => {self.dec_r8(REG_H)},
            0x26 => {},
            0x27 => {},
            0x28 => {},
            0x29 => {},
            0x2A => {},
            0x2B => {self.dec_r16(REG_H, REG_L)},
            0x2C => {self.inc_r8(REG_L)},
            0x2D => {self.dec_r8(REG_L)},
            0x2E => {},
            0x2F => {},
            0x30 => {},
            0x31 => {},
            0x32 => {},
            0x33 => {self.inc_r16pcsp(REG_SP)},
            0x34 => {self.inc_r8a(REG_H, REG_L)},
            0x35 => {self.dec_r8a(REG_H, REG_L)},
            0x36 => {},
            0x37 => {},
            0x38 => {},
            0x39 => {},
            0x3A => {},
            0x3B => {self.dec_r16pcsp(REG_SP)},
            0x3C => {self.inc_r8(REG_A)},
            0x3D => {self.dec_r8(REG_A)},
            0x3E => {},
            0x3F => {},
            0x40 => {self.ld_r8_r8(REG_B, REG_B)},
            0x41 => {self.ld_r8_r8(REG_B, REG_C)},
            0x42 => {self.ld_r8_r8(REG_B, REG_D)},
            0x43 => {self.ld_r8_r8(REG_B, REG_E)},
            0x44 => {self.ld_r8_r8(REG_B, REG_H)},
            0x45 => {self.ld_r8_r8(REG_B, REG_L)},
            0x46 => {self.ld_r8_r16a(REG_B, REG_H, REG_L)},
            0x47 => {self.ld_r8_r8(REG_B, REG_A)},
            0x48 => {self.ld_r8_r8(REG_C, REG_B)},
            0x49 => {self.ld_r8_r8(REG_C, REG_C)},
            0x4A => {self.ld_r8_r8(REG_C, REG_D)},
            0x4B => {self.ld_r8_r8(REG_C, REG_E)},
            0x4C => {self.ld_r8_r8(REG_C, REG_H)},
            0x4D => {self.ld_r8_r8(REG_C, REG_L)},
            0x4E => {self.ld_r8_r16a(REG_C, REG_H, REG_L)},
            0x4F => {self.ld_r8_r8(REG_C, REG_A)},
            0x50 => {self.ld_r8_r8(REG_D, REG_B)},
            0x51 => {self.ld_r8_r8(REG_D, REG_C)},
            0x52 => {self.ld_r8_r8(REG_D, REG_D)},
            0x53 => {self.ld_r8_r8(REG_D, REG_E)},
            0x54 => {self.ld_r8_r8(REG_D, REG_H)},
            0x55 => {self.ld_r8_r8(REG_D, REG_L)},
            0x56 => {self.ld_r8_r16a(REG_D, REG_H, REG_L)},
            0x57 => {self.ld_r8_r8(REG_D, REG_A)},
            0x58 => {self.ld_r8_r8(REG_E, REG_B)},
            0x59 => {self.ld_r8_r8(REG_E, REG_C)},
            0x5A => {self.ld_r8_r8(REG_E, REG_D)},
            0x5B => {self.ld_r8_r8(REG_E, REG_E)},
            0x5C => {self.ld_r8_r8(REG_E, REG_H)},
            0x5D => {self.ld_r8_r8(REG_E, REG_L)},
            0x5E => {self.ld_r8_r16a(REG_E, REG_H, REG_L)},
            0x5F => {self.ld_r8_r8(REG_E, REG_A)},
            0x60 => {self.ld_r8_r8(REG_H, REG_B)},
            0x61 => {self.ld_r8_r8(REG_H, REG_C)},
            0x62 => {self.ld_r8_r8(REG_H, REG_D)},
            0x63 => {self.ld_r8_r8(REG_H, REG_E)},
            0x64 => {self.ld_r8_r8(REG_H, REG_H)},
            0x65 => {self.ld_r8_r8(REG_H, REG_L)},
            0x66 => {self.ld_r8_r16a(REG_H, REG_H, REG_L)},
            0x67 => {self.ld_r8_r8(REG_H, REG_A)},
            0x68 => {self.ld_r8_r8(REG_L, REG_B)},
            0x69 => {self.ld_r8_r8(REG_L, REG_C)},
            0x6A => {self.ld_r8_r8(REG_L, REG_D)},
            0x6B => {self.ld_r8_r8(REG_L, REG_E)},
            0x6C => {self.ld_r8_r8(REG_L, REG_H)},
            0x6D => {self.ld_r8_r8(REG_L, REG_L)},
            0x6E => {self.ld_r8_r16a(REG_L, REG_H, REG_L)},
            0x6F => {self.ld_r8_r8(REG_L, REG_A)},
            0x70 => {self.ld_r16a_r8(REG_H, REG_L, REG_B)},
            0x71 => {self.ld_r16a_r8(REG_H, REG_L, REG_C)},
            0x72 => {self.ld_r16a_r8(REG_H, REG_L, REG_D)},
            0x73 => {self.ld_r16a_r8(REG_H, REG_L, REG_E)},
            0x74 => {self.ld_r16a_r8(REG_H, REG_L, REG_H)},
            0x75 => {self.ld_r16a_r8(REG_H, REG_L, REG_L)},
            0x76 => {},
            0x77 => {self.ld_r16a_r8(REG_H, REG_L, REG_A)},
            0x78 => {self.ld_r8_r8(REG_A, REG_B)},
            0x79 => {self.ld_r8_r8(REG_A, REG_C)},
            0x7A => {self.ld_r8_r8(REG_A, REG_D)},
            0x7B => {self.ld_r8_r8(REG_A, REG_E)},
            0x7C => {self.ld_r8_r8(REG_A, REG_H)},
            0x7D => {self.ld_r8_r8(REG_A, REG_L)},
            0x7E => {self.ld_r8_r16a(REG_A, REG_H, REG_L)},
            0x7F => {self.ld_r8_r8(REG_A, REG_A)},
            0x80 => {self.add_r8_r8(REG_A, REG_B)},
            0x81 => {self.add_r8_r8(REG_A, REG_C)},
            0x82 => {self.add_r8_r8(REG_A, REG_D)},
            0x83 => {self.add_r8_r8(REG_A, REG_E)},
            0x84 => {self.add_r8_r8(REG_A, REG_H)},
            0x85 => {self.add_r8_r8(REG_A, REG_L)},
            0x86 => {self.add_r8_r8a(REG_A, REG_H, REG_L)},
            0x87 => {self.add_r8_r8(REG_A, REG_A)},
            0x88 => {self.adc_r8_r8(REG_A, REG_B)},
            0x89 => {self.adc_r8_r8(REG_A, REG_C)},
            0x8A => {self.adc_r8_r8(REG_A, REG_D)},
            0x8B => {self.adc_r8_r8(REG_A, REG_E)},
            0x8C => {self.adc_r8_r8(REG_A, REG_H)},
            0x8D => {self.adc_r8_r8(REG_A, REG_L)},
            0x8E => {self.adc_r8_r8a(REG_A, REG_H, REG_L)},
            0x8F => {self.adc_r8_r8(REG_A, REG_A)},
            0x90 => {self.sub_r8_r8(REG_A, REG_B)},
            0x91 => {self.sub_r8_r8(REG_A, REG_C)},
            0x92 => {self.sub_r8_r8(REG_A, REG_D)},
            0x93 => {self.sub_r8_r8(REG_A, REG_E)},
            0x94 => {self.sub_r8_r8(REG_A, REG_H)},
            0x95 => {self.sub_r8_r8(REG_A, REG_L)},
            0x96 => {self.sub_r8_r8a(REG_A, REG_H, REG_L)},
            0x97 => {self.sub_r8_r8(REG_A, REG_A)},
            0x98 => {self.sbc_r8_r8(REG_A, REG_B)},
            0x99 => {self.sbc_r8_r8(REG_A, REG_C)},
            0x9A => {self.sbc_r8_r8(REG_A, REG_D)},
            0x9B => {self.sbc_r8_r8(REG_A, REG_E)},
            0x9C => {self.sbc_r8_r8(REG_A, REG_H)},
            0x9D => {self.sbc_r8_r8(REG_A, REG_L)},
            0x9E => {self.sbc_r8_r8a(REG_A, REG_H, REG_L)},
            0x9F => {self.sbc_r8_r8(REG_A, REG_A)},
            0xA0 => {self.and_r8_r8(REG_A, REG_B)},
            0xA1 => {self.and_r8_r8(REG_A, REG_C)},
            0xA2 => {self.and_r8_r8(REG_A, REG_D)},
            0xA3 => {self.and_r8_r8(REG_A, REG_E)},
            0xA4 => {self.and_r8_r8(REG_A, REG_H)},
            0xA5 => {self.and_r8_r8(REG_A, REG_L)},
            0xA6 => {self.and_r8_r8a(REG_A, REG_H, REG_L)},
            0xA7 => {self.and_r8_r8(REG_A, REG_A)},
            0xA8 => {self.xor_r8_r8(REG_A, REG_B)},
            0xA9 => {self.xor_r8_r8(REG_A, REG_C)},
            0xAA => {self.xor_r8_r8(REG_A, REG_D)},
            0xAB => {self.xor_r8_r8(REG_A, REG_E)},
            0xAC => {self.xor_r8_r8(REG_A, REG_H)},
            0xAD => {self.xor_r8_r8(REG_A, REG_L)},
            0xAE => {self.xor_r8_r8a(REG_A, REG_H, REG_L)},
            0xAF => {self.xor_r8_r8(REG_A, REG_A)},
            0xB0 => {self.or_r8_r8(REG_A, REG_B)},
            0xB1 => {self.or_r8_r8(REG_A, REG_C)},
            0xB2 => {self.or_r8_r8(REG_A, REG_D)},
            0xB3 => {self.or_r8_r8(REG_A, REG_E)},
            0xB4 => {self.or_r8_r8(REG_A, REG_H)},
            0xB5 => {self.or_r8_r8(REG_A, REG_L)},
            0xB6 => {self.or_r8_r8a(REG_A, REG_H, REG_L)},
            0xB7 => {self.or_r8_r8(REG_A, REG_A)},
            0xB8 => {self.cp_r8_r8(REG_A, REG_B)},
            0xB9 => {self.cp_r8_r8(REG_A, REG_C)},
            0xBA => {self.cp_r8_r8(REG_A, REG_D)},
            0xBB => {self.cp_r8_r8(REG_A, REG_E)},
            0xBC => {self.cp_r8_r8(REG_A, REG_H)},
            0xBD => {self.cp_r8_r8(REG_A, REG_L)},
            0xBE => {self.cp_r8_r8a(REG_A, REG_H, REG_L)},
            0xBF => {self.cp_r8_r8(REG_A, REG_A)},
            0xC0 => {},
            0xC1 => {},
            0xC2 => {},
            0xC3 => {},
            0xC4 => {},
            0xC5 => {},
            0xC6 => {},
            0xC7 => {},
            0xC8 => {},
            0xC9 => {},
            0xCA => {},
            0xCB => {},
            0xCC => {},
            0xCD => {},
            0xCE => {},
            0xCF => {},
            0xD0 => {},
            0xD1 => {},
            0xD2 => {},
            0xD3 => {},
            0xD4 => {},
            0xD5 => {},
            0xD6 => {},
            0xD7 => {},
            0xD8 => {},
            0xD9 => {},
            0xDA => {},
            0xDB => {},
            0xDC => {},
            0xDD => {},
            0xDE => {},
            0xDF => {},
            0xE0 => {},
            0xE1 => {},
            0xE2 => {},
            0xE3 => {},
            0xE4 => {},
            0xE5 => {},
            0xE6 => {},
            0xE7 => {},
            0xE8 => {},
            0xE9 => {},
            0xEA => {},
            0xEB => {},
            0xEC => {},
            0xED => {},
            0xEE => {},
            0xEF => {},
            0xF0 => {},
            0xF1 => {},
            0xF2 => {},
            0xF3 => {},
            0xF4 => {},
            0xF5 => {},
            0xF6 => {},
            0xF7 => {},
            0xF8 => {},
            0xF9 => {},
            0xFA => {},
            0xFB => {},
            0xFC => {},
            0xFD => {},
            0xFE => {},
            0xFF => {},
            _ => panic!("Tried to execute invalid instruction")
        }

        match self.pcsp_regs[REG_PC] //CB Prefix
        {
            0x00 => {self.rlc_r8(REG_B)},
            0x01 => {self.rlc_r8(REG_C)},
            0x02 => {self.rlc_r8(REG_D)},
            0x03 => {self.rlc_r8(REG_E)},
            0x04 => {self.rlc_r8(REG_H)},
            0x05 => {self.rlc_r8(REG_L)},
            0x06 => {self.rlc_r8a(REG_H, REG_L)},
            0x07 => {self.rlc_r8(REG_A)},
            0x08 => {self.rrc_r8(REG_B)},
            0x09 => {self.rrc_r8(REG_C)},
            0x0A => {self.rrc_r8(REG_D)},
            0x0B => {self.rrc_r8(REG_E)},
            0x0C => {self.rrc_r8(REG_H)},
            0x0D => {self.rrc_r8(REG_L)},
            0x0E => {self.rrc_r8a(REG_H, REG_L)},
            0x0F => {self.rrc_r8(REG_A)},
            0x10 => {self.rl_r8(REG_B)},
            0x11 => {self.rl_r8(REG_C)},
            0x12 => {self.rl_r8(REG_D)},
            0x13 => {self.rl_r8(REG_E)},
            0x14 => {self.rl_r8(REG_H)},
            0x15 => {self.rl_r8(REG_L)},
            0x16 => {self.rl_r8a(REG_H, REG_L)},
            0x17 => {self.rl_r8(REG_A)},
            0x18 => {self.rr_r8(REG_B)},
            0x19 => {self.rr_r8(REG_C)},
            0x1A => {self.rr_r8(REG_D)},
            0x1B => {self.rr_r8(REG_E)},
            0x1C => {self.rr_r8(REG_H)},
            0x1D => {self.rr_r8(REG_L)},
            0x1E => {self.rr_r8a(REG_H, REG_L)},
            0x1F => {self.rr_r8(REG_A)},
            0x20 => {self.sla_r8(REG_B)},
            0x21 => {self.sla_r8(REG_C)},
            0x22 => {self.sla_r8(REG_D)},
            0x23 => {self.sla_r8(REG_E)},
            0x24 => {self.sla_r8(REG_H)},
            0x25 => {self.sla_r8(REG_L)},
            0x26 => {self.sla_r8a(REG_H, REG_L)},
            0x27 => {self.sla_r8(REG_A)},
            0x28 => {self.sra_r8(REG_B)},
            0x29 => {self.sra_r8(REG_C)},
            0x2A => {self.sra_r8(REG_D)},
            0x2B => {self.sra_r8(REG_E)},
            0x2C => {self.sra_r8(REG_H)},
            0x2D => {self.sra_r8(REG_L)},
            0x2E => {self.sra_r8a(REG_H, REG_L)},
            0x2F => {self.sra_r8(REG_A)},
            0x30 => {self.swap_r8(REG_B)},
            0x31 => {self.swap_r8(REG_C)},
            0x32 => {self.swap_r8(REG_D)},
            0x33 => {self.swap_r8(REG_E)},
            0x34 => {self.swap_r8(REG_H)},
            0x35 => {self.swap_r8(REG_L)},
            0x36 => {self.swap_r8a(REG_H, REG_L)},
            0x37 => {self.swap_r8(REG_A)},
            0x38 => {self.srl_r8(REG_B)},
            0x39 => {self.srl_r8(REG_C)},
            0x3A => {self.srl_r8(REG_D)},
            0x3B => {self.srl_r8(REG_E)},
            0x3C => {self.srl_r8(REG_H)},
            0x3D => {self.srl_r8(REG_L)},
            0x3E => {self.srl_r8a(REG_H, REG_L)},
            0x3F => {self.srl_r8(REG_A)},
            0x40 => {self.bit_r8(0, REG_B)},
            0x41 => {self.bit_r8(0, REG_C)},
            0x42 => {self.bit_r8(0, REG_D)},
            0x43 => {self.bit_r8(0, REG_E)},
            0x44 => {self.bit_r8(0, REG_H)},
            0x45 => {self.bit_r8(0, REG_L)},
            0x46 => {self.bit_r8a(0, REG_H, REG_L)},
            0x47 => {self.bit_r8(0, REG_A)},
            0x48 => {self.bit_r8(1, REG_B)},
            0x49 => {self.bit_r8(1, REG_C)},
            0x4A => {self.bit_r8(1, REG_D)},
            0x4B => {self.bit_r8(1, REG_E)},
            0x4C => {self.bit_r8(1, REG_H)},
            0x4D => {self.bit_r8(1, REG_L)},
            0x4E => {self.bit_r8a(1, REG_H, REG_L)},
            0x4F => {self.bit_r8(1, REG_A)},
            0x50 => {self.bit_r8(2, REG_B)},
            0x51 => {self.bit_r8(2, REG_C)},
            0x52 => {self.bit_r8(2, REG_D)},
            0x53 => {self.bit_r8(2, REG_E)},
            0x54 => {self.bit_r8(2, REG_H)},
            0x55 => {self.bit_r8(2, REG_L)},
            0x56 => {self.bit_r8a(2, REG_H, REG_L)},
            0x57 => {self.bit_r8(2, REG_A)},
            0x58 => {self.bit_r8(3, REG_B)},
            0x59 => {self.bit_r8(3, REG_C)},
            0x5A => {self.bit_r8(3, REG_D)},
            0x5B => {self.bit_r8(3, REG_E)},
            0x5C => {self.bit_r8(3, REG_H)},
            0x5D => {self.bit_r8(3, REG_L)},
            0x5E => {self.bit_r8a(3, REG_H, REG_L)},
            0x5F => {self.bit_r8(3, REG_A)},
            0x60 => {self.bit_r8(4, REG_B)},
            0x61 => {self.bit_r8(4, REG_C)},
            0x62 => {self.bit_r8(4, REG_D)},
            0x63 => {self.bit_r8(4, REG_E)},
            0x64 => {self.bit_r8(4, REG_H)},
            0x65 => {self.bit_r8(4, REG_L)},
            0x66 => {self.bit_r8a(4, REG_H, REG_L)},
            0x67 => {self.bit_r8(4, REG_A)},
            0x68 => {self.bit_r8(5, REG_B)},
            0x69 => {self.bit_r8(5, REG_C)},
            0x6A => {self.bit_r8(5, REG_D)},
            0x6B => {self.bit_r8(5, REG_E)},
            0x6C => {self.bit_r8(5, REG_H)},
            0x6D => {self.bit_r8(5, REG_L)},
            0x6E => {self.bit_r8a(5, REG_H, REG_L)},
            0x6F => {self.bit_r8(5, REG_A)},
            0x70 => {self.bit_r8(6, REG_B)},
            0x71 => {self.bit_r8(6, REG_C)},
            0x72 => {self.bit_r8(6, REG_D)},
            0x73 => {self.bit_r8(6, REG_E)},
            0x74 => {self.bit_r8(6, REG_H)},
            0x75 => {self.bit_r8(6, REG_L)},
            0x76 => {self.bit_r8a(6, REG_H, REG_L)},
            0x77 => {self.bit_r8(6, REG_A)},
            0x78 => {self.bit_r8(7, REG_B)},
            0x79 => {self.bit_r8(7, REG_C)},
            0x7A => {self.bit_r8(7, REG_D)},
            0x7B => {self.bit_r8(7, REG_E)},
            0x7C => {self.bit_r8(7, REG_H)},
            0x7D => {self.bit_r8(7, REG_L)},
            0x7E => {self.bit_r8a(7, REG_H, REG_L)},
            0x7F => {self.bit_r8(7, REG_A)},
            0x80 => {self.res_r8(0, REG_B)},
            0x81 => {self.res_r8(0, REG_C)},
            0x82 => {self.res_r8(0, REG_D)},
            0x83 => {self.res_r8(0, REG_E)},
            0x84 => {self.res_r8(0, REG_H)},
            0x85 => {self.res_r8(0, REG_L)},
            0x86 => {self.res_r8a(0, REG_H, REG_L)},
            0x87 => {self.res_r8(0, REG_A)},
            0x88 => {self.res_r8(1, REG_B)},
            0x89 => {self.res_r8(1, REG_C)},
            0x8A => {self.res_r8(1, REG_D)},
            0x8B => {self.res_r8(1, REG_E)},
            0x8C => {self.res_r8(1, REG_H)},
            0x8D => {self.res_r8(1, REG_L)},
            0x8E => {self.res_r8a(1, REG_H, REG_L)},
            0x8F => {self.res_r8(1, REG_A)},
            0x90 => {self.res_r8(2, REG_B)},
            0x91 => {self.res_r8(2, REG_C)},
            0x92 => {self.res_r8(2, REG_D)},
            0x93 => {self.res_r8(2, REG_E)},
            0x94 => {self.res_r8(2, REG_H)},
            0x95 => {self.res_r8(2, REG_L)},
            0x96 => {self.res_r8a(2, REG_H, REG_L)},
            0x97 => {self.res_r8(2, REG_A)},
            0x98 => {self.res_r8(3, REG_B)},
            0x99 => {self.res_r8(3, REG_C)},
            0x9A => {self.res_r8(3, REG_D)},
            0x9B => {self.res_r8(3, REG_E)},
            0x9C => {self.res_r8(3, REG_H)},
            0x9D => {self.res_r8(3, REG_L)},
            0x9E => {self.res_r8a(3, REG_H, REG_L)},
            0x9F => {self.res_r8(3, REG_A)},
            0xA0 => {self.res_r8(4, REG_B)},
            0xA1 => {self.res_r8(4, REG_C)},
            0xA2 => {self.res_r8(4, REG_D)},
            0xA3 => {self.res_r8(4, REG_E)},
            0xA4 => {self.res_r8(4, REG_H)},
            0xA5 => {self.res_r8(4, REG_L)},
            0xA6 => {self.res_r8a(4, REG_H, REG_L)},
            0xA7 => {self.res_r8(4, REG_A)},
            0xA8 => {self.res_r8(5, REG_B)},
            0xA9 => {self.res_r8(5, REG_C)},
            0xAA => {self.res_r8(5, REG_D)},
            0xAB => {self.res_r8(5, REG_E)},
            0xAC => {self.res_r8(5, REG_H)},
            0xAD => {self.res_r8(5, REG_L)},
            0xAE => {self.res_r8a(5, REG_H, REG_L)},
            0xAF => {self.res_r8(5, REG_A)},
            0xB0 => {self.res_r8(6, REG_B)},
            0xB1 => {self.res_r8(6, REG_C)},
            0xB2 => {self.res_r8(6, REG_D)},
            0xB3 => {self.res_r8(6, REG_E)},
            0xB4 => {self.res_r8(6, REG_H)},
            0xB5 => {self.res_r8(6, REG_L)},
            0xB6 => {self.res_r8a(6, REG_H, REG_L)},
            0xB7 => {self.res_r8(6, REG_A)},
            0xB8 => {self.res_r8(7, REG_B)},
            0xB9 => {self.res_r8(7, REG_C)},
            0xBA => {self.res_r8(7, REG_D)},
            0xBB => {self.res_r8(7, REG_E)},
            0xBC => {self.res_r8(7, REG_H)},
            0xBD => {self.res_r8(7, REG_L)},
            0xBE => {self.res_r8a(7, REG_H, REG_L)},
            0xBF => {self.res_r8(7, REG_A)},
            0xC0 => {self.set_r8(0, REG_B)},
            0xC1 => {self.set_r8(0, REG_C)},
            0xC2 => {self.set_r8(0, REG_D)},
            0xC3 => {self.set_r8(0, REG_E)},
            0xC4 => {self.set_r8(0, REG_H)},
            0xC5 => {self.set_r8(0, REG_L)},
            0xC6 => {self.set_r8a(0, REG_H, REG_L)},
            0xC7 => {self.set_r8(0, REG_A)},
            0xC8 => {self.set_r8(1, REG_B)},
            0xC9 => {self.set_r8(1, REG_C)},
            0xCA => {self.set_r8(1, REG_D)},
            0xCB => {self.set_r8(1, REG_E)},
            0xCC => {self.set_r8(1, REG_H)},
            0xCD => {self.set_r8(1, REG_L)},
            0xCE => {self.set_r8a(1, REG_H, REG_L)},
            0xCF => {self.set_r8(1, REG_A)},
            0xD0 => {self.set_r8(2, REG_B)},
            0xD1 => {self.set_r8(2, REG_C)},
            0xD2 => {self.set_r8(2, REG_D)},
            0xD3 => {self.set_r8(2, REG_E)},
            0xD4 => {self.set_r8(2, REG_H)},
            0xD5 => {self.set_r8(2, REG_L)},
            0xD6 => {self.set_r8a(2, REG_H, REG_L)},
            0xD7 => {self.set_r8(2, REG_A)},
            0xD8 => {self.set_r8(3, REG_B)},
            0xD9 => {self.set_r8(3, REG_C)},
            0xDA => {self.set_r8(3, REG_D)},
            0xDB => {self.set_r8(3, REG_E)},
            0xDC => {self.set_r8(3, REG_H)},
            0xDD => {self.set_r8(3, REG_L)},
            0xDE => {self.set_r8a(3, REG_H, REG_L)},
            0xDF => {self.set_r8(3, REG_A)},
            0xE0 => {self.set_r8(4, REG_B)},
            0xE1 => {self.set_r8(4, REG_C)},
            0xE2 => {self.set_r8(4, REG_D)},
            0xE3 => {self.set_r8(4, REG_E)},
            0xE4 => {self.set_r8(4, REG_H)},
            0xE5 => {self.set_r8(4, REG_L)},
            0xE6 => {self.set_r8a(4, REG_H, REG_L)},
            0xE7 => {self.set_r8(4, REG_A)},
            0xE8 => {self.set_r8(5, REG_B)},
            0xE9 => {self.set_r8(5, REG_C)},
            0xEA => {self.set_r8(5, REG_D)},
            0xEB => {self.set_r8(5, REG_E)},
            0xEC => {self.set_r8(5, REG_H)},
            0xED => {self.set_r8(5, REG_L)},
            0xEE => {self.set_r8a(5, REG_H, REG_L)},
            0xEF => {self.set_r8(5, REG_A)},
            0xF0 => {self.set_r8(6, REG_B)},
            0xF1 => {self.set_r8(6, REG_C)},
            0xF2 => {self.set_r8(6, REG_D)},
            0xF3 => {self.set_r8(6, REG_E)},
            0xF4 => {self.set_r8(6, REG_H)},
            0xF5 => {self.set_r8(6, REG_L)},
            0xF6 => {self.set_r8a(6, REG_H, REG_L)},
            0xF7 => {self.set_r8(6, REG_A)},
            0xF8 => {self.set_r8(7, REG_B)},
            0xF9 => {self.set_r8(7, REG_C)},
            0xFA => {self.set_r8(7, REG_D)},
            0xFB => {self.set_r8(7, REG_E)},
            0xFC => {self.set_r8(7, REG_H)},
            0xFD => {self.set_r8(7, REG_L)},
            0xFE => {self.set_r8a(7, REG_H, REG_L)},
            0xFF => {self.set_r8(7, REG_A)},
            _ => panic!("Tried to execute invalid instruction")
        }
    }
}