use std::time::Duration;

use rand::prelude::*;

use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use sdl3::pixels::PixelFormat;

const WIDTH: u32 = 500;
const WIDTH_SUBPIXEL: usize = WIDTH as usize * 3;
const HEIGHT: u32 = 500;
const PIXEL_COUNT: usize = (WIDTH * HEIGHT) as usize;
const BUFFER_SIZE: usize = PIXEL_COUNT * 3;

// Change these to alter the simulation
const TREE_SPAWN_RATE: u32 = 10;
const LIGHTNING_SPAWN_RATE: u32 = 150;
const SIM_SPEED: u32 = 200; // that sets the target FPS of the sim
// but it's limited by the PC speed

fn is_tree(index: usize, buffer: &[u8]) -> bool {
    buffer[index] == 0 && buffer[index + 1] == 255 && buffer[index + 2] == 0
}

fn set_tree(index: usize, buffer: &mut [u8]) {
    buffer[index] = 0;
    buffer[index + 1] = 255;
    buffer[index + 2] = 0;
}

fn is_fire(index: usize, buffer: &[u8]) -> bool {
    buffer[index] == 255 && buffer[index + 1] == 0 && buffer[index + 2] == 0
}

fn set_fire(index: usize, buffer: &mut [u8]) {
    buffer[index] = 255;
    buffer[index + 1] = 0;
    buffer[index + 2] = 0;
}

fn is_empty(index: usize, buffer: &[u8]) -> bool {
    buffer[index] == 0 && buffer[index + 1] == 0 && buffer[index + 2] == 0
}

fn clear_pixel(index: usize, buffer: &mut [u8]) {
    buffer[index] = 0;
    buffer[index + 1] = 0;
    buffer[index + 2] = 0;
}

fn burn_trees(buffer: &mut [u8]) {
    // tmp_buffer is needed to not change the original buffer while checking it
    // otherwise errors occure
    let mut tmp_buffer = vec![0; BUFFER_SIZE];
    tmp_buffer.copy_from_slice(buffer); // copy buffer into tmp_buffer
    for i in 0..PIXEL_COUNT {
        if is_fire(i * 3, buffer) {
            // check if a pixel is on fire
            // check surrounding pixel if its a tree and...
            let index = i * 3;
            let positions = [
                // top_left
                index - WIDTH_SUBPIXEL - 3,
                // top
                index - WIDTH_SUBPIXEL,
                // top_right
                index - WIDTH_SUBPIXEL + 3,
                // right
                index + 3,
                // down_right
                index + WIDTH_SUBPIXEL + 3,
                // down
                index + WIDTH_SUBPIXEL,
                // down_left
                index + WIDTH_SUBPIXEL - 3,
                // left
                index - 3,
            ];

            for index in positions.iter() {
                if is_tree(*index, buffer) {
                    set_fire(*index, &mut tmp_buffer); // ...burn it
                }
            }
        }
    }
    buffer.copy_from_slice(&tmp_buffer);
}

// leave only edge fire and extinguish every other
fn clear_fire(buffer: &mut [u8], prev_frame: &[u8]) {
    for n in 0..PIXEL_COUNT {
        if is_fire(n * 3, prev_frame) {
            clear_pixel(n * 3, buffer);
        }
    }
}

// fill every edge pixel with black
fn clear_edges(buffer: &mut [u8]) {
    for x in 0..WIDTH_SUBPIXEL {
        // Top row
        buffer[x] = 0;
        // Down row
        buffer[WIDTH_SUBPIXEL * (HEIGHT as usize - 1) + x] = 0;
    }
    for y in 0..HEIGHT as usize {
        // Left column
        let index = WIDTH_SUBPIXEL * y;
        buffer[index] = 0;
        buffer[index + 1] = 0;
        buffer[index + 2] = 0;
        // Right column
        buffer[index + WIDTH_SUBPIXEL - 3] = 0;
        buffer[index + WIDTH_SUBPIXEL - 2] = 0;
        buffer[index + WIDTH_SUBPIXEL - 1] = 0;
    }
}

fn main() {
    // store the current frame
    let mut buffer: Vec<u8> = vec![0; BUFFER_SIZE];
    // keep the previous frame
    let mut prev_buffer: Vec<u8> = vec![0; BUFFER_SIZE];
    let mut rng = rand::rng();

    let sdl_context = sdl3::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Forest Fire Sim", WIDTH, HEIGHT)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas();

    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(PixelFormat::RGB24, WIDTH, HEIGHT)
        .unwrap();

    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut frame_count = 0;
    let mut run = false;

    for _ in 0..(PIXEL_COUNT as f32 * 0.5).round() as usize {
        set_tree(rng.random_range(0..PIXEL_COUNT) * 3, &mut buffer);
    }
    clear_edges(&mut buffer);

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    ..
                } => run = true,
                _ => {}
            }
        }
        if run {
            // continuously spawn trees
            for _ in 0..TREE_SPAWN_RATE {
                let spawn_point = rng.random_range(0..PIXEL_COUNT) * 3;
                if is_empty(spawn_point, &buffer) {
                    set_tree(spawn_point, &mut buffer);
                }
            }

            // every other frame spawn a lightning on rng location
            if frame_count >= LIGHTNING_SPAWN_RATE {
                let spawn_point = rng.random_range(0..PIXEL_COUNT) * 3;
                if is_tree(spawn_point, &buffer) {
                    set_fire(spawn_point, &mut buffer);
                    frame_count = 0;
                }
            }

            // clears edge pixel so the burn_trees function doesnt check out of range pixel locations
            clear_edges(&mut buffer);
            // spread the fire
            burn_trees(&mut buffer);
            // clean fire so only edge fire remains
            clear_fire(&mut buffer, &prev_buffer);

            texture.update(None, &buffer, WIDTH_SUBPIXEL).unwrap();
            canvas.copy(&texture, None, None).unwrap();
            canvas.present();
            // ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / SIM_SPEED));

            // copy current buffer into prev_buffer
            prev_buffer.copy_from_slice(&buffer);
            // increase frame_count for frame depending lightning spawn
            frame_count += 1;
        } else {
            texture.update(None, &buffer, WIDTH_SUBPIXEL).unwrap();
            canvas.copy(&texture, None, None).unwrap();
            canvas.present();
            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));
        }
    }
}
