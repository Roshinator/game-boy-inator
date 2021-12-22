use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::{Point};
use std::time::Duration;
use std::thread;
use game_boy_hardware::mainboard::Mainboard;

const SCREEN_WIDTH:i32 = 160;
const SCREEN_HEIGHT:i32 = 144;
fn main()
{
    // let x: game_boy_hardware::cpu::CPU;
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
    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop
    {
        for event in event_pump.poll_iter()
        {
            match event
            {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } =>
                {
                    break 'running
                },
                _ => {}
            }
        }
        canvas.set_draw_color(Color::RGB(255, 0, 0));
        canvas.clear();
        for i in 0..SCREEN_WIDTH
        {
            for j in 0..SCREEN_HEIGHT
            {
                let color = ((i % 2 == j % 2) as u8) * 255;
                canvas.set_draw_color(Color::RGB(color, color, color));
                canvas.draw_point(Point::new(i, j)).unwrap();
                //canvas.fill_rect(Rect::new(i,j,1,1)).unwrap();
            }
        }
        canvas.present();
        thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    let motherboard = Mainboard::new();
}
