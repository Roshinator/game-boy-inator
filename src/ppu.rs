use crate::ram::{self, Ram};

pub const SCREEN_WIDTH:usize = 160;
pub const SCREEN_HEIGHT:usize = 144;
const CYCLES_PER_SCANLINE:u64 = 456;
const VBLANK_LINES:u64 = 10;
pub const CYCLES_PER_FRAME:u64 = CYCLES_PER_SCANLINE * (SCREEN_HEIGHT as u64 + VBLANK_LINES);

bitflags::bitflags!
{
    struct LcdcFlag: u8
    {
        const BG_ENABLE = 1 << 0;
        const OBJ_ON = 1 << 1;
        const OBJ_SIZE_SELECT = 1 << 2; //OBJ_BLOCK_COMP 0=8x8, 1=8x16
        const BG_CODE_AREA_SELECT = 1 << 3;
        const CHAR_DATA_SELECT = 1 << 4;
        const WINDOWING_ON = 1 << 5;
        const WINDOW_CODE_AREA_SELECT = 1 << 6;
        const LCD_CONTROLLER_OPERATION_ON = 1 << 7;
    }
}

const STAT_MODE:u8 = 0b00000011;
const STAT_MATCH:u8 = 1 << 2;

const PALETTE_DOT_00:u8 = 0b00000011;
const PALETTE_DOT_01:u8 = 0b00001100;
const PALETTE_DOT_10:u8 = 0b00110000;
const PALETTE_DOT_11:u8 = 0b11000000;

const OBJ_LCD_Y_RAM_OFFSET:u16 = 0;
const OBJ_LCD_X_RAM_OFFSET:u16 = 1;
const OBJ_CHR_CODE_OFFSET:u16 = 2;
const OBJ_ATTRIBUTE_OFFSET:u16 = 3;
const OBJ_ATTRIBUTE_PALETTE:u8 = 1 << 4;
const OBJ_ATTRIBUTE_H_FLIP:u8 = 1 << 5;
const OBJ_ATTRIBUTE_V_FLIP:u8 = 1 << 6;
const OBJ_ATTRIBUTE_PRIORITY:u8 = 1 << 7;

#[derive(Default, Clone, Copy)]
pub struct Sprite
{
    x_coord: u8,
    y_coord: u8,
    tile_index: u8,
    use_palette_1: bool,
    x_flip: bool,
    y_flip: bool,
    priority: bool
}
pub struct Ppu
{
    frame_progress: u64,
    buffer: [[u8;SCREEN_HEIGHT];SCREEN_WIDTH],
    new_frame: bool,
    sprite_buffer: Vec<Sprite>,
    current_x: u8,
    frame_count: u64
}

impl Ppu
{
    pub fn new() -> Ppu
    {
        Ppu
        {
            frame_progress: 0,
            buffer: [[0; SCREEN_HEIGHT]; SCREEN_WIDTH],
            new_frame: false,
            sprite_buffer: Default::default(),
            current_x: 0,
            frame_count: 0
        }
    }

    pub fn execute(&mut self, ram: &mut Ram, hardware_handle: crate::HardwareHandle)
    {
        let scan_line = (self.frame_progress / (CYCLES_PER_SCANLINE as u64)) as u8;
        //4 pixels per cycle
        for _ in 0..4
        {
            self.pixel_update(ram, scan_line);
        }
        let next_scan_line = (self.frame_progress / (CYCLES_PER_SCANLINE as u64)) as u8;

        // println!("{}", scan_line);
        if next_scan_line == 0 && scan_line != 0
        {
            #[cfg(feature = "ppu-debug")]
            println!("Drawing screen");
            hardware_handle.borrow_mut().video_update(&self.buffer, self.frame_count);
            self.frame_count += 1;
        }
    }

    fn pixel_update(&mut self, ram: &mut Ram, scan_line: u8)
    {
        let lcd_on = ram.read(ram::LCDC) & LcdcFlag::LCD_CONTROLLER_OPERATION_ON.bits != 0;
        ram.write(ram::LY, scan_line);
        let status = ram.read(ram::STAT);

        if lcd_on
        {
            let y_compare_match = ram.read(ram::LYC) == scan_line;
            if y_compare_match && status & 0x04 == 0 && status & 0x40 != 0 //If match, fresh match, and interrupt mode set to compare match, fire interrupt
            {
                ram.set_interrupt(ram::InterruptFlag::LCDC)
            }
            ram.write(ram::STAT, status & !((!y_compare_match as u8) << 2));
        }

        //Begin pixel write
        //Mode 0: H-blank (92c), 1: vblank (), 2: vram in use, 3: vram transfer
        let mode = status & 0b00000011;
        if scan_line >= 144 //Handle V-blank
        {
            if mode != 1
            {
                ram.set_interrupt(ram::InterruptFlag::VB);
                //Set mode to 1
                ram.write(ram::STAT, ram.read(ram::STAT) & 0b11111101);

            }
        }
        else
        {
            let scan_progress = self.frame_progress % CYCLES_PER_SCANLINE;
            match scan_progress
            {
                0..=91 if mode != 2 => //Mode 2
                {
                    //Set mode to 2
                    ram.write(ram::STAT, (ram.read(ram::STAT) & 0b11111100) | 0b00000010);
                    if status & (1 << 5) != 0
                    {
                        ram.set_interrupt(ram::InterruptFlag::LCDC);
                    }
                    self.sprite_buffer = self.get_sprites_from_oam(ram, scan_line);
                    self.current_x = 0;

                },
                92..=251 if mode != 3 => //Mode 3
                {
                    //Set mode to 3
                    ram.write(ram::STAT, (ram.read(ram::STAT) & 0b11111100) | 0b00000011);
                    if lcd_on
                    {
                        let start = self.current_x;
                        let bound = (scan_progress - 92) as u8;
                        for x in start..bound
                        {
                            self.current_x = x;
                            self.draw_pixel(ram, scan_line, self.current_x);
                        }
                    }
                },
                252..=455 if mode != 0 => //Mode 0
                {
                    //Set mode to 0
                    ram.write(ram::STAT, (ram.read(ram::STAT) & 0b11111100) | 0b11111100);
                    if status & (1 << 3) != 0
                    {
                        ram.set_interrupt(ram::InterruptFlag::LCDC);
                    }
                    if lcd_on
                    {
                        let start = self.current_x;
                        for x in start..160
                        {
                            self.current_x = x;
                            self.draw_pixel(ram, scan_line, self.current_x);
                        }
                    }
                }
                _ => {}
            }
        }

        //Final progress update
        self.frame_progress = (self.frame_progress + 1) % CYCLES_PER_FRAME;
    }

    fn draw_pixel(&mut self, ram: &mut Ram, scan_line: u8, x_coord: u8)
    {
        let lcdc = LcdcFlag::from_bits(ram.read(ram::LCDC)).unwrap();
        let bg_enable = lcdc.contains(LcdcFlag::BG_ENABLE);
        let obj_on = lcdc.contains(LcdcFlag::OBJ_ON);
        let bg_tile_hi_map = lcdc.contains(LcdcFlag::BG_CODE_AREA_SELECT);
        let bg_char_lo_tiles = lcdc.contains(LcdcFlag::CHAR_DATA_SELECT);
        let windowing_on = lcdc.contains(LcdcFlag::WINDOWING_ON);
        let windowing_tile_hi_map = lcdc.contains(LcdcFlag::WINDOW_CODE_AREA_SELECT);
        let mut sprite_height = 8_u8;
        if lcdc.contains(LcdcFlag::OBJ_SIZE_SELECT)
        {
            sprite_height = 16;
        }

        let window_x = ram.read(ram::WX);
        let window_y = ram.read(ram::WY);
        let scroll_y = ram.read(ram::SCY);
        let scroll_x = ram.read(ram::SCX);

        let mut bg_pixel = 0_u8;
        if windowing_on && x_coord + 8 > window_x
        {
            bg_pixel = self.get_color_of_pixel(ram, windowing_tile_hi_map, bg_char_lo_tiles, x_coord + 7 - window_x, scan_line - window_y);
        }
        else if bg_enable
        {
            bg_pixel = self.get_color_of_pixel(ram, bg_tile_hi_map, bg_char_lo_tiles, ((x_coord + scroll_x) as u32 % 256) as u8, ((scan_line + scroll_y) as u32 % 256) as u8);
        }
        let mut output_color = self.color_palette_lookup(bg_pixel, ram.read(ram::BGP));

        if obj_on
        {
            let sprites = &self.sprite_buffer;
            if !sprites.is_empty()
            {
                let palette0 = ram.read(ram::OBP0);
                let palette1 = ram.read(ram::OBP1);

                for sprite in sprites
                {
                    if x_coord + 8 >= sprite.x_coord && x_coord + 8 < sprite.x_coord + 8
                    {
                        let mut tile_index = sprite.tile_index;
                        if sprite_height == 16
                        {
                            tile_index &= 0xFE; //???
                        }

                        let mut y_tile_px = scan_line + 16 - sprite.y_coord;
                        if sprite.y_flip
                        {
                            y_tile_px = sprite_height - 1 - y_tile_px;
                        }

                        let mut x_tile_px = x_coord + 8 - sprite.x_coord;
                        if sprite.x_flip
                        {
                            x_tile_px = 7 - x_tile_px;
                        }
                        let tile_address = self.get_tile_addr(tile_index, y_tile_px, true);
                        let pixels = [ram.read(tile_address), ram.read(tile_address + 1)];
                        let pixel = self.get_color_from_tilemap(&pixels, x_tile_px);
                        if pixel != 0
                        {
                            let priority = sprite.priority;
                            if output_color == 0 || priority
                            {
                                let mut palette = palette0;
                                if sprite.use_palette_1
                                {
                                    palette = palette1;
                                }
                                output_color = self.color_palette_lookup(pixel, palette);
                            }
                            break;
                        }
                    }
                }
            }
        }
        self.buffer[x_coord as usize][scan_line as usize] = output_color;
    }

    fn color_palette_lookup(&self, pixel:u8, palette:u8) -> u8
    {
        (palette >> (pixel * 2)) & 0b00000011
    }

    fn get_tile_addr(&self, idx: u8, y_coord: u8, lower_bank: bool) -> u16
    {
        let mut addr = ram::OBJ1.start() + (idx as u16) * 16;
        if !lower_bank
        {
            addr = (*ram::OBJ2.start() as i32 + 0x1000 + idx as i32 * 16) as u16;
        }
        addr + y_coord as u16 * 2
    }

    fn get_color_from_tilemap(&self, palette_line: &[u8;2], x_coord: u8) -> u8
    {
        ((palette_line[0] << x_coord) & 0x80) >> 7 | (((palette_line[1] << x_coord) & 0x80) >> 6)
    }

    fn get_color_of_pixel(&self, ram: &Ram, hi_tile_map: bool, low_bank: bool, x_coord: u8, y_coord: u8) -> u8
    {
        let mut start = ram::VRAM1.start();
        if hi_tile_map
        {
            start = ram::VRAM2.start();
        }
        let tile_index = ram.read(*start + ((y_coord as u16 / 8) * 32 + x_coord as u16 / 8));
        let tile_addr = self.get_tile_addr(tile_index, y_coord % 8, low_bank);
        self.get_color_from_tilemap(&[ram.read(tile_addr), ram.read(tile_addr + 1)], x_coord % 8)
    }

    fn get_sprites_from_oam(&mut self, ram: &mut Ram, scan_num: u8) -> Vec<Sprite>
    {
        let mut sprites = Vec::<Sprite>::new();
        sprites.reserve_exact(11);

        let sprite_height = if (ram.read(ram::LCDC) & LcdcFlag::OBJ_SIZE_SELECT.bits) == 0
        {
            8_u8
        }
        else
        {
            16_u8
        };

        for oam_slot in (ram::OAM).step_by(4)
        {
            let attributes = ram.read(oam_slot + OBJ_ATTRIBUTE_OFFSET);
            let sprite = Sprite
            {
                y_coord: ram.read(oam_slot + OBJ_LCD_Y_RAM_OFFSET),
                x_coord: ram.read(oam_slot + OBJ_LCD_X_RAM_OFFSET),
                tile_index: ram.read(oam_slot + OBJ_CHR_CODE_OFFSET),
                use_palette_1: attributes & OBJ_ATTRIBUTE_PALETTE != 0,
                x_flip: attributes & OBJ_ATTRIBUTE_H_FLIP != 0,
                y_flip: attributes & OBJ_ATTRIBUTE_V_FLIP != 0,
                priority: attributes & OBJ_ATTRIBUTE_PRIORITY == 0
            };

            if sprite.y_coord > 0 && sprite.y_coord < 160 && sprite.x_coord < (SCREEN_WIDTH as u8) + 8 //On screen
                && scan_num + 16 >= sprite.y_coord && scan_num + 16 < sprite.y_coord + sprite_height //In Scanline
            {
                let mut priority = sprites.len();
                while priority > 0 && sprites[priority - 1].x_coord > sprite.x_coord //Sort by leftmost, then by oam (will naturally happen since we analyze by oam)
                {
                    priority -= 1;
                }
                sprites.insert(priority, sprite);
                if sprites.len() > 10
                {
                    sprites.remove(sprites.len() - 1);
                }
            }

        }
        sprites
    }
}

impl Default for Ppu
{
    fn default() -> Self { Self::new() }
}