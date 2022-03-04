use std::path::{Path, PathBuf};
use rfd::FileDialog;
use sdl2::{render::*, video::*, EventPump, event::*, keyboard::*, pixels::*, rect::*};
use game_boy_hardware::{mainboard::Mainboard, ppu};

fn main()
{
    let args: Vec<String> = std::env::args().collect();
    let filename = if args.len() < 2
    {
        let dialog = FileDialog::new()
            .set_directory(&std::env::current_dir().unwrap())
            .add_filter("Game Boy Roms", &["gb"]);
        match dialog.pick_file()
        {
            Some(path) => path,
            None => panic!("Bad file path")
        }
    }
    else
    {
        PathBuf::from(&args[1])
    };
    let frontend = PCHardware::new();
    let mut motherboard = Mainboard::new(frontend);
    motherboard.load_game(Path::new(filename.as_path())).unwrap();

    loop
    {
        motherboard.execute_frame();
    }
}

const COLORS:[Color; 4] =
[
    Color::RGB(0xFF, 0xFF, 0xFF),
    Color::RGB(0xAA, 0xAA, 0xAA),
    Color::RGB(0x55, 0x55, 0x55),
    Color::RGB(0x00, 0x00, 0x00),
];

struct PCHardware
{
    canvas: Canvas<Window>,
    event_pump: EventPump
}

impl PCHardware
{
    fn new() -> Self
    {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem.window("Game Boy Inator", 810, 730)
            .position_centered()
            .resizable()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().accelerated().build().unwrap();
        canvas.set_logical_size(ppu::SCREEN_WIDTH as u32, ppu::SCREEN_HEIGHT as u32).unwrap();
        canvas.set_integer_scale(true).unwrap();
        let event_pump = sdl_context.event_pump().unwrap();
        PCHardware
        {
            canvas: canvas,
            event_pump:event_pump
        }
    }
}

impl game_boy_hardware::Frontend for PCHardware
{
    fn event_poll(&mut self)
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
    }

    fn video_update(&mut self, buffer: &[[u8; ppu::SCREEN_HEIGHT];ppu::SCREEN_WIDTH])
    {
        self.canvas.set_draw_color(Color::BLACK);
        self.canvas.clear();
        for x_coord in 0..ppu::SCREEN_WIDTH
        {
            for y_coord in 0..ppu::SCREEN_HEIGHT
            {
                self.canvas.set_draw_color(COLORS[buffer[x_coord][y_coord] as usize]);
                self.canvas.draw_point(Point::new(x_coord as i32, y_coord as i32)).unwrap();
            }
        }
        self.canvas.present();
    }
}
