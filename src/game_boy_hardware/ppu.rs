use sdl2::{render::*, video::*, EventPump, event::*, keyboard::*, pixels::*, rect::*};
use super::ram::Ram;

const SCREEN_WIDTH:usize = 160;
const SCREEN_HEIGHT:usize = 144;
const COLORS:[Color; 4] = 
[
    Color::RGB(0xFF, 0xFF, 0xFF),
    Color::RGB(0xAA, 0xAA, 0xAA),
    Color::RGB(0x55, 0x55, 0x55),
    Color::RGB(0x00, 0x00, 0x00),    
];

const LCDC_OBJ_ON:u8 = 1 << 1;
const LCDC_OBJ_BLOCK_COMP_SELECT:u8 = 1 << 2;
const LCDC_BG_CODE_AREA_SELET:u8 = 1 << 3;
const LCDC_CHAR_DATA_SELECT:u8 = 1 << 4;
const LCDC_WINDOWING_ON:u8 = 1 << 5;
const LCDC_WINDOW_CODE_AREA_SELECT:u8 = 1 << 6;
const LCDC_LCD_CONTROLLER_OPERATION_STOP:u8 = 1 << 7;

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
pub struct Ppu
{
    canvas: Canvas<Window>,
    event_pump: EventPump,
    frame_progress:u64,
    buffer: [[u8;SCREEN_WIDTH];SCREEN_HEIGHT],
    new_frame: bool
}

impl Ppu
{
    pub fn new() -> Ppu
    {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem.window("Game Boy Inator", 800, 600)
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
            buffer: [[0; SCREEN_WIDTH]; SCREEN_HEIGHT],
            new_frame: false
        }
    }

    pub fn execute(&mut self, ram: &mut Ram)
    {
        self.draw_to_screen();
    }

    fn draw_to_screen(&mut self)
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
        for i in 0..SCREEN_WIDTH
        {
            for j in 0..SCREEN_HEIGHT
            {
                let color = COLORS[(i % 2 == j % 2) as usize];
                self.canvas.set_draw_color(color);
                self.canvas.draw_point(Point::new(i as i32, j as i32)).unwrap();
                //canvas.fill_rect(Rect::new(i,j,1,1)).unwrap();
            }
        }
        self.canvas.present();
    }
}