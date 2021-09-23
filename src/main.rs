use emulator::State8080;

mod disassembler;
mod emulator;

fn main() {
    let buf_h = std::fs::read("/Users/prezi/Developer/emu-8080/resources/invaders.h")
        .expect("Failed to read file");
    let buf_g = std::fs::read("/Users/prezi/Developer/emu-8080/resources/invaders.g")
        .expect("Failed to read file");
    let buf_f = std::fs::read("/Users/prezi/Developer/emu-8080/resources/invaders.f")
        .expect("Failed to read file");
    let buf_e = std::fs::read("/Users/prezi/Developer/emu-8080/resources/invaders.e")
        .expect("Failed to read file");

    let state = State8080::new()
        .loading_buffer_into_memory_at(buf_h, 0)
        .loading_buffer_into_memory_at(buf_g, 0x800)
        .loading_buffer_into_memory_at(buf_f, 0x1000)
        .loading_buffer_into_memory_at(buf_e, 0x1800);

    println!("Start...");

    state.run();
}
