use game_boy_hardware::mainboard::Mainboard;


fn main()
{
    let mut motherboard = Mainboard::new(
        std::fs::File::open("Snake.gb").unwrap());
    
    motherboard.begin_execution();
}
