mod emu;
use emu::*;

use raylib::prelude::*;

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(640, 320)
        .title("CHIP-8 emulator")
        .build();

    rl.set_target_fps(60);

    let mut canvas = rl.load_render_texture(&thread, 64, 32).unwrap();

    let rom = std::fs::read("rom.ch8").unwrap();
    let mut screen = Screen::new();
    let mut cpu = CPU::new(&mut screen, rom);

    while !rl.window_should_close() {
        for _ in 0..7_000_000 / 60 {
            cpu.cycle(CpuInput { key_pressed: [false; 16] });
        }
        let scr = cpu.screen.export().clone();
        canvas.update_texture(&scr);

        let mut d = rl.begin_drawing(&thread);
        d.draw_texture_ex(&canvas, Vector2::zero(), 0.0, 10.0, Color::WHITE);
    }
}
