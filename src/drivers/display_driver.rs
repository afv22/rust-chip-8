// A display driver for chip-8 using SDL2
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

pub struct DisplayDriver {
    canvas: Canvas<Window>,
}

impl DisplayDriver {
    pub fn new(context: &sdl2::Sdl) -> Self {
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
        Self { canvas }
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
}
