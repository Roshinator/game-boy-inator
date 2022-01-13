

struct ppu
{
    frame_progress:u64,
    buffer: [[u8;144];160],
    new_frame: bool
}