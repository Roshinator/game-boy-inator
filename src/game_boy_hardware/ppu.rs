use sdl2::{render::*, video::*, EventPump, event::*, keyboard::*, pixels::*, rect::*};
use super::ram::{self, Ram};

const SCREEN_WIDTH:usize = 160;
const SCREEN_HEIGHT:usize = 144;
const SCREEN_WIDTH_U8:u8 = SCREEN_WIDTH as u8;
const SCREEN_HEIGHT_U8:u8 = SCREEN_HEIGHT as u8;
const CYCLES_PER_SCANLINE:u64 = 456;
const VBLANK_LINES:u64 = 10;
const CYCLES_PER_FRAME:u64 = CYCLES_PER_SCANLINE * (SCREEN_HEIGHT as u64 + VBLANK_LINES);

const COLORS:[Color; 4] =
[
    Color::RGB(0xFF, 0xFF, 0xFF),
    Color::RGB(0xAA, 0xAA, 0xAA),
    Color::RGB(0x55, 0x55, 0x55),
    Color::RGB(0x00, 0x00, 0x00),
];

const LCDC_BG_WIN_ENABLE:u8 = 1 << 0;
const LCDC_OBJ_ON:u8 = 1 << 1;
const LCDC_OBJ_SIZE_SELECT:u8 = 1 << 2; //OBJ_BLOCK_COMP 0=8x8, 1=8x16
const LCDC_BG_CODE_AREA_SELET:u8 = 1 << 3;
const LCDC_CHAR_DATA_SELECT:u8 = 1 << 4;
const LCDC_WINDOWING_ON:u8 = 1 << 5;
const LCDC_WINDOW_CODE_AREA_SELECT:u8 = 1 << 6;
const LCDC_LCD_CONTROLLER_OPERATION_ON:u8 = 1 << 7;

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
    palette: bool,
    x_flip: bool,
    y_flip: bool,
    priority: bool
}
pub struct Ppu
{
    canvas: Canvas<Window>,
    event_pump: EventPump,
    frame_progress:u64,
    buffer: [[u8;SCREEN_HEIGHT];SCREEN_WIDTH],
    new_frame: bool
}

impl Ppu
{
    pub fn new() -> Ppu
    {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem.window("Game Boy Inator", 810, 730)
            .position_centered()
            .resizable()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().accelerated().build().unwrap();
        canvas.set_logical_size(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32).unwrap();
        canvas.set_integer_scale(true).unwrap();
        let event_pump = sdl_context.event_pump().unwrap();

        Ppu
        {
            canvas: canvas,
            event_pump: event_pump,
            frame_progress: 0,
            buffer: [[0; SCREEN_HEIGHT]; SCREEN_WIDTH],
            new_frame: false
        }
    }

    pub fn execute(&mut self, ram: &mut Ram)
    {
        let scan_line = (self.frame_progress / (CYCLES_PER_SCANLINE as u64)) as u8;
        //4 pixels per cycle
        for _ in 0..4
        {
            self.pixel_update(ram, scan_line);
        }

        if self.new_frame
        {
            println!("Drawing screen");
            self.draw_to_screen(ram);
            self.new_frame = false;
        }
    }

    fn pixel_update(&mut self, ram: &mut Ram, scan_line: u8)
    {
        let lcd_on = ram.read(ram::LCDC) & LCDC_LCD_CONTROLLER_OPERATION_ON != 0;
        ram.write(ram::LY, scan_line);
        let status = ram.read(ram::STAT);

        if lcd_on
        {
            let y_compare_match = ram.read(ram::LYC) == scan_line;
            if y_compare_match && status & 0x04 == 0 && status & 0x40 != 0 //If match, fresh match, and interrupt mode set to compare match, fire interrupt
            {
                ram.write(ram::IF, ram.read(ram::IF) | ram::INTERRUPT_LCDC);
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
                ram.write(ram::IF, ram.read(ram::IF) | ram::INTERRUPT_VB);
                //Set mode to 1
                ram.write(ram::STAT, ram.read(ram::STAT) & 0b11111101);

            }
        }
        else
        {
            let scan_progress = self.frame_progress % CYCLES_PER_SCANLINE;
            match scan_progress
            {
                0..=91 if mode != 0 => //Mode 0
                {
                    //Set mode to 0
                    ram.write(ram::STAT, ram.read(ram::STAT) & 0b11111100);
                },
                92..=251 if mode != 2 => //Mode 2
                {
                    //Set mode to 2
                    ram.write(ram::STAT, ram.read(ram::STAT) & 0b11111110);
                },
                252..=455 if mode != 3 => //Mode 3
                {
                    //Set mode to 3
                    ram.write(ram::STAT, ram.read(ram::STAT) & 0b11111111);
                }
                _ => {}
            }
        }

        //Final progress update
        let next_frame = (self.frame_progress + 1) % CYCLES_PER_FRAME;
        if  next_frame < self.frame_progress
        {
            self.new_frame = true;
        }
        self.frame_progress = next_frame;
    }

    fn draw_to_screen(&mut self, ram: &mut Ram)
    {
        for event in self.event_pump.poll_iter()
        {
            match event
            {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } =>
                {
                    std::process::exit(0);
                },
                _ => {}
            }
        }

        self.canvas.set_draw_color(Color::BLACK);
        self.canvas.clear();
        for x_coord in 0..SCREEN_WIDTH
        {
            for y_coord in 0..SCREEN_HEIGHT
            {
                self.canvas.set_draw_color(COLORS[self.buffer[x_coord][y_coord] as usize]);
                self.canvas.draw_point(Point::new(x_coord as i32, y_coord as i32)).unwrap();
            }
        }
        self.canvas.present();

    }

    fn get_palette_addr(&self, ram: &Ram, idx: u8, y_coord: u8, lower_bank: bool)
    {
        let mut addr = ram::VRAM1.start();
        if !lower_bank
        {
            addr = ram::VRAM2.start();
        }
        //TODO
    }

    fn get_color_from_palette(&self, palette_line: &[u8;2], x_coord: u8) -> u8
    {
        ((palette_line[0] << x_coord) & 0x80) >> 7 | (((palette_line[1] << x_coord) & 0x80) >> 6)
    }

    fn get_sprites_from_oam(&mut self, ram: &mut Ram, scan_num: u8) -> Vec<Sprite>
    {
        let mut sprites = Vec::<Sprite>::new();
        sprites.reserve_exact(11);

        let sprite_height;
        if (ram.read(ram::LCDC) & LCDC_OBJ_SIZE_SELECT) == 0
        {
            sprite_height = 8_u8;
        }
        else
        {
            sprite_height = 16_u8;
        }

        for oam_slot in (ram::OAM).step_by(4)
        {
            let attributes = ram.read(oam_slot + OBJ_ATTRIBUTE_OFFSET);
            let sprite = Sprite
            {
                y_coord: ram.read(oam_slot + OBJ_LCD_Y_RAM_OFFSET),
                x_coord: ram.read(oam_slot + OBJ_LCD_X_RAM_OFFSET),
                tile_index: ram.read(oam_slot + OBJ_CHR_CODE_OFFSET),
                palette: attributes << OBJ_ATTRIBUTE_PALETTE != 0,
                x_flip: attributes << OBJ_ATTRIBUTE_H_FLIP != 0,
                y_flip: attributes << OBJ_ATTRIBUTE_V_FLIP != 0,
                priority: attributes << OBJ_ATTRIBUTE_PRIORITY != 0
            };

            if sprite.y_coord > 0 && sprite.y_coord < 160 && sprite.x_coord < SCREEN_WIDTH_U8 + 8 //On screen
                && scan_num + 16 >= sprite.y_coord && scan_num + 16 < sprite.y_coord + sprite_height //In Scanline
            {
                let mut priority = sprites.len();
                while priority > 0 && sprites[priority - 1].x_coord > sprite.x_coord //Sort by leftmost, then by oam (will naturally happen since we analyze by oam)
                {
                    priority -= 1;
                }
                sprites.insert(priority, sprite);
                sprites.remove(sprites.len() - 1);
            }

        }
        return sprites;
    }
}