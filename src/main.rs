use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;
use std::thread;

mod game_boy_hardware;
fn main()
{
//     let x: CPU;
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("SDL2 Test", 800, 600)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(255, 0, 0));
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
        canvas.clear();
        canvas.present();
        thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

}
