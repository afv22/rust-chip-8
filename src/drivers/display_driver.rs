// A display driver for chip-8 using SDL2
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

pub struct DisplayDriver {
    context: sdl2::Sdl,
    canvas: Canvas<Window>,
}

impl DisplayDriver {
    pub fn new(context: sdl2::Sdl) -> Self {
        let video_subsystem = context.video().unwrap();
        let window = video_subsystem
            .window("Chip-8 Emulator", 64 * 8, 32 * 8)
            .position_centered()
            .build()
            .unwrap();
        let mut canvas = window.into_canvas().build().unwrap();
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.present();
        Self { context, canvas }
    }

    pub fn draw(&mut self, display: &[[u8; 64]; 32]) {
        self.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.canvas.clear();
        for y in 0..32 {
            for x in 0..64 {
                if display[y][x] == 1 {
                    self.canvas.set_draw_color(Color::RGB(255, 255, 255));
                    self.canvas
                        .fill_rect(Rect::new(x as i32 * 8, y as i32 * 8, 8, 8))
                        .unwrap();
                }
            }
        }
        self.canvas.present();
    }

    pub fn handle_events(&mut self) {
        for event in self.context.event_pump().unwrap().poll_iter() {
            match event {
                Event::Quit { .. } => {
                    std::process::exit(0);
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    std::process::exit(0);
                }
                _ => {}
            }
        }
    }
}
