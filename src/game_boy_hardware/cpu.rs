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
        let x = self.ram.read_from_address_reg_pair(self.regs[msh], self.regs[lsh]);
        self.regs[p1] = x;
    }

    fn ld_r16a_r8(&mut self, msh: Reg, lsh: Reg, p2: Reg)
    {
        self.ram.write_to_address_reg_pair(self.regs[msh], self.regs[lsh], self.regs[p2]);
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
        let result = self.ram.read_from_address_reg_pair(
            self.regs[msh], self.regs[lsh]).overflowing_add(1);
        self.ram.write_to_address_reg_pair(self.regs[msh], self.regs[lsh], result.0);
        
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
        let result = self.ram.read_from_address_reg_pair(
            self.regs[msh], self.regs[lsh]).overflowing_sub(1);
        self.ram.write_to_address_reg_pair(self.regs[msh], self.regs[lsh], result.0);
        
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
        let p2 = self.ram.read_from_address_reg_pair(self.regs[msh], self.regs[lsh]);
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
        let p2 = self.ram.read_from_address_reg_pair(self.regs[msh], self.regs[lsh]);
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
        self.aux_write_flag(FLAG_N, false);
        self.aux_write_flag(FLAG_C, result.1);
    }

    fn sub_r8_r8a(&mut self, p1: Reg, msh: Reg, lsh: Reg)
    {
        let p2 = self.ram.read_from_address_reg_pair(self.regs[msh], self.regs[lsh]);
        let half_carry_pre = ((self.regs[p1] ^ p2) >> 4) & 1;
        let result = self.regs[p1].overflowing_add(p2);
        self.regs[p1] = result.0;
        let half_carry_post = (result.0 >> 4) & 1;
        
        self.aux_write_flag(FLAG_Z, result.0 == 0);
        self.aux_write_flag(FLAG_H, half_carry_pre != half_carry_post);
        self.aux_write_flag(FLAG_N, false);
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
        self.aux_write_flag(FLAG_N, false);
        self.aux_write_flag(FLAG_C, result1.1 || result2.1);
    }

    fn sbc_r8_r8a(&mut self, p1: Reg, msh: Reg, lsh: Reg)
    {
        let carry = self.aux_read_flag(FLAG_C) as u8;
        let p2 = self.ram.read_from_address_reg_pair(self.regs[msh], self.regs[lsh]);
        let half_carry_pre1 = ((self.regs[p1] ^ p2) >> 4) & 1;
        let result1 = self.regs[p1].overflowing_sub(p2);
        let half_carry_post1 = (result1.0 >> 4) & 1;
        let half_carry_pre2 = ((result1.0 ^ carry) >> 4) & 1;
        let result2 = result1.0.overflowing_sub(carry);
        self.regs[p1] = result2.0;
        let half_carry_post2 = (result2.0 >> 4) & 1;
        
        self.aux_write_flag(FLAG_Z, result2.0 == 0);
        self.aux_write_flag(FLAG_H, half_carry_pre1 != half_carry_post1 || half_carry_pre2 != half_carry_post2);
        self.aux_write_flag(FLAG_N, false);
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
        self.regs[p1] = self.regs[p1] & self.ram.read_from_address_reg_pair(self.regs[msh], self.regs[lsh]);

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
        self.regs[p1] = self.regs[p1] ^ self.ram.read_from_address_reg_pair(self.regs[msh], self.regs[lsh]);

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
        self.regs[p1] = self.regs[p1] | self.ram.read_from_address_reg_pair(self.regs[msh], self.regs[lsh]);

        self.aux_write_flag(FLAG_Z, self.regs[p1] == 0);
        self.aux_write_flag(FLAG_H, false);
        self.aux_write_flag(FLAG_N, false);
        self.aux_write_flag(FLAG_C, false);
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
            0xB8 => {},
            0xB9 => {},
            0xBA => {},
            0xBB => {},
            0xBC => {},
            0xBD => {},
            0xBE => {},
            0xBF => {},
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
            0x00 => {},
            0x01 => {},
            0x02 => {},
            0x03 => {},
            0x04 => {},
            0x05 => {},
            0x06 => {},
            0x07 => {},
            0x08 => {},
            0x09 => {},
            0x0A => {},
            0x0B => {},
            0x0C => {},
            0x0D => {},
            0x0E => {},
            0x0F => {},
            0x10 => {},
            0x11 => {},
            0x12 => {},
            0x13 => {},
            0x14 => {},
            0x15 => {},
            0x16 => {},
            0x17 => {},
            0x18 => {},
            0x19 => {},
            0x1A => {},
            0x1B => {},
            0x1C => {},
            0x1D => {},
            0x1E => {},
            0x1F => {},
            0x20 => {},
            0x21 => {},
            0x22 => {},
            0x23 => {},
            0x24 => {},
            0x25 => {},
            0x26 => {},
            0x27 => {},
            0x28 => {},
            0x29 => {},
            0x2A => {},
            0x2B => {},
            0x2C => {},
            0x2D => {},
            0x2E => {},
            0x2F => {},
            0x30 => {},
            0x31 => {},
            0x32 => {},
            0x33 => {},
            0x34 => {},
            0x35 => {},
            0x36 => {},
            0x37 => {},
            0x38 => {},
            0x39 => {},
            0x3A => {},
            0x3B => {},
            0x3C => {},
            0x3D => {},
            0x3E => {},
            0x3F => {},
            0x40 => {},
            0x41 => {},
            0x42 => {},
            0x43 => {},
            0x44 => {},
            0x45 => {},
            0x46 => {},
            0x47 => {},
            0x48 => {},
            0x49 => {},
            0x4A => {},
            0x4B => {},
            0x4C => {},
            0x4D => {},
            0x4E => {},
            0x4F => {},
            0x50 => {},
            0x51 => {},
            0x52 => {},
            0x53 => {},
            0x54 => {},
            0x55 => {},
            0x56 => {},
            0x57 => {},
            0x58 => {},
            0x59 => {},
            0x5A => {},
            0x5B => {},
            0x5C => {},
            0x5D => {},
            0x5E => {},
            0x5F => {},
            0x60 => {},
            0x61 => {},
            0x62 => {},
            0x63 => {},
            0x64 => {},
            0x65 => {},
            0x66 => {},
            0x67 => {},
            0x68 => {},
            0x69 => {},
            0x6A => {},
            0x6B => {},
            0x6C => {},
            0x6D => {},
            0x6E => {},
            0x6F => {},
            0x70 => {},
            0x71 => {},
            0x72 => {},
            0x73 => {},
            0x74 => {},
            0x75 => {},
            0x76 => {},
            0x77 => {},
            0x78 => {},
            0x79 => {},
            0x7A => {},
            0x7B => {},
            0x7C => {},
            0x7D => {},
            0x7E => {},
            0x7F => {},
            0x80 => {},
            0x81 => {},
            0x82 => {},
            0x83 => {},
            0x84 => {},
            0x85 => {},
            0x86 => {},
            0x87 => {},
            0x88 => {},
            0x89 => {},
            0x8A => {},
            0x8B => {},
            0x8C => {},
            0x8D => {},
            0x8E => {},
            0x8F => {},
            0x90 => {},
            0x91 => {},
            0x92 => {},
            0x93 => {},
            0x94 => {},
            0x95 => {},
            0x96 => {},
            0x97 => {},
            0x98 => {},
            0x99 => {},
            0x9A => {},
            0x9B => {},
            0x9C => {},
            0x9D => {},
            0x9E => {},
            0x9F => {},
            0xA0 => {},
            0xA1 => {},
            0xA2 => {},
            0xA3 => {},
            0xA4 => {},
            0xA5 => {},
            0xA6 => {},
            0xA7 => {},
            0xA8 => {},
            0xA9 => {},
            0xAA => {},
            0xAB => {},
            0xAC => {},
            0xAD => {},
            0xAE => {},
            0xAF => {},
            0xB0 => {},
            0xB1 => {},
            0xB2 => {},
            0xB3 => {},
            0xB4 => {},
            0xB5 => {},
            0xB6 => {},
            0xB7 => {},
            0xB8 => {},
            0xB9 => {},
            0xBA => {},
            0xBB => {},
            0xBC => {},
            0xBD => {},
            0xBE => {},
            0xBF => {},
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
    }
}