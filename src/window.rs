use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use sdl3::EventPump;
use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use sdl3::pixels::PixelFormat;
use sdl3::render::Canvas;
use sdl3::video::Window;

use crate::{HEIGHT, WIDTH, WIDTH_SUBPIXEL};

pub struct Viewport {
    canvas: Canvas<Window>,
    event_pump: EventPump,
}

impl Viewport {
    pub fn new() -> Self {
        let sdl_context = sdl3::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem
            .window("Pathfinder", WIDTH, HEIGHT)
            .position_centered()
            .build()
            .unwrap();

        let canvas = window.into_canvas();
        let event_pump = sdl_context.event_pump().unwrap();
        Self { canvas, event_pump }
    }

    pub fn get_input(&mut self) -> bool {
        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => return true,
                _ => {}
            }
        }
        false
    }

    pub fn draw(&mut self, buffer: &Arc<Mutex<Vec<u8>>>, quit: Arc<AtomicBool>) {
        let texture_creator = self.canvas.texture_creator();
        let mut texture = texture_creator
            .create_texture_streaming(PixelFormat::RGB24, WIDTH, HEIGHT)
            .unwrap();
        'running: loop {
            if self.get_input() {
                quit.store(true, Ordering::Relaxed);
                break 'running;
            }

            let buffer = buffer.lock().unwrap();
            texture.update(None, &buffer, WIDTH_SUBPIXEL).unwrap();
            drop(buffer);
            self.canvas.copy(&texture, None, None).unwrap();
            self.canvas.present();
            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));
        }
    }
}
