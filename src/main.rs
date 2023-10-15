mod emu;
use emu::*;

use raylib::prelude::*;

const KEY_MAP: [KeyboardKey; 16] = [
    KeyboardKey::KEY_X,
    KeyboardKey::KEY_ONE,
    KeyboardKey::KEY_TWO,
    KeyboardKey::KEY_THREE,
    KeyboardKey::KEY_Q,
    KeyboardKey::KEY_W,
    KeyboardKey::KEY_E,
    KeyboardKey::KEY_A,
    KeyboardKey::KEY_S,
    KeyboardKey::KEY_D,
    KeyboardKey::KEY_Z,
    KeyboardKey::KEY_C,
    KeyboardKey::KEY_FOUR,
    KeyboardKey::KEY_R,
    KeyboardKey::KEY_F,
    KeyboardKey::KEY_V,
];

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(640, 320)
        .title("CHIP-8 emulator")
        .build();

    rl.set_target_fps(60);

    let mut canvas = rl.load_render_texture(&thread, 64, 32).unwrap();
    // let mut rl_aud = RaylibAudio::init_audio_device();

    let rom = std::fs::read("rom.ch8").unwrap();
    let mut screen = Screen::new();
    let mut cpu = CPU::new(&mut screen, rom);

    while !rl.window_should_close() {
        let mut pressed = [false; 16];
        for i in 0..16 {
            pressed[i] = rl.is_key_down(KEY_MAP[i]);
        }
        for _ in 0..7_000_000 / 60 {
            cpu.cycle(CpuInput { key_pressed: pressed });
        }
        cpu.delay = cpu.delay.saturating_sub(1);
        cpu.sound = cpu.sound.saturating_sub(1);

        let scr = cpu.screen.export().clone();
        canvas.update_texture(&scr);

        let mut d = rl.begin_drawing(&thread);
        d.draw_texture_ex(&canvas, Vector2::zero(), 0.0, 10.0, Color::WHITE);
    }
}
