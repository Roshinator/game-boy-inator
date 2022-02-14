use std::path;

use game_boy_hardware::mainboard::Mainboard;


fn main()
{
    let args: Vec<String> = std::env::args().collect();
    let filename = &args[1];
    let mut motherboard = Mainboard::new();
    motherboard.load_game(path::Path::new(filename)).unwrap();
    
    motherboard.execute_frame();
}
