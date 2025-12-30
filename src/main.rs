use std::time::Duration;

use rand::prelude::*;

use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use sdl3::pixels::{Color, PixelFormat};
use sdl3::render::Texture;
use sdl3::surface::Surface;

const WIDTH: u32 = 500;
const HEIGHT: u32 = 500;

// define fixed colors
const TREE: u32 = 65280;
const FIRE: u32 = 16711680;

// Change these to alter the simulation
const TREE_SPAWN_RATE: u32 = 10;
const LIGHTNING_SPAWN_RATE: u32 = 150;
const SIM_SPEED: usize = 200; // that sets the target FPS of the sim
// but it's limited by the PC speed

struct Point {
    x: u32,
    y: u32,
}

impl Point {
    fn new(x: u32, y: u32) -> Point {
        Point { x, y }
    }
}

fn is_tree(index: usize, buffer: &[u8]) -> bool {
    buffer[index + 1] == 255
}

fn set_tree(index: usize, buffer: &mut [u8]) {
    buffer[index + 1] = 255;
}

fn is_fire(index: usize, buffer: &[u8]) -> bool {
    buffer[index] == 255
}

fn set_fire(index: usize, buffer: &mut [u8]) {
    buffer[index] = 255;
}

fn xy_to_one_d(point: Point) -> usize {
    point.x as usize + point.y as usize * WIDTH as usize
}

fn one_d_to_xy(index: usize) -> Point {
    let x = index as u32 % WIDTH * 3;
    let y = index as u32 / WIDTH * 3;
    Point::new(x, y)
}

fn _from_u8_rgb(r: u8, g: u8, b: u8) -> u32 {
    let (r, g, b) = (r as u32, g as u32, b as u32);
    (r << 16) | (g << 8) | b
}

fn burn_trees(buffer: &mut [u8]) {
    // tmp_buffer is needed to not change the original buffer while checking it
    // otherwise errors occure
    let mut tmp_buffer = vec![0; (WIDTH * HEIGHT) as usize * 3];
    tmp_buffer.copy_from_slice(buffer); // copy buffer into tmp_buffer
    for i in 0..buffer.len() / 3 {
        if is_fire(i, buffer) {
            // check if a pixel is on fire
            // check surrounding pixel if its a tree and...
            let start_point = one_d_to_xy(i);
            let positions = [
                // top_left
                xy_to_one_d(Point::new(start_point.x - 1, start_point.y - 1)),
                // top
                xy_to_one_d(Point::new(start_point.x, start_point.y - 1)),
                // top_right
                xy_to_one_d(Point::new(start_point.x + 1, start_point.y - 1)),
                // right
                xy_to_one_d(Point::new(start_point.x + 1, start_point.y)),
                // down_right
                xy_to_one_d(Point::new(start_point.x + 1, start_point.y + 1)),
                // down
                xy_to_one_d(Point::new(start_point.x, start_point.y + 1)),
                // down_left
                xy_to_one_d(Point::new(start_point.x - 1, start_point.y + 1)),
                // left
                xy_to_one_d(Point::new(start_point.x - 1, start_point.y)),
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
fn delete_fire(buffer: &mut [u8], prev_frame: &[u8]) {
    for n in 0..buffer.len() / 3 {
        if is_fire(n, prev_frame) {
            buffer[n] = 0;
            buffer[n + 1] = 0;
            buffer[n + 2] = 0;
        }
    }
}

// fill every edge pixel with "0"
fn delete_edge(buffer: &mut [u8]) {
    for x in 0..WIDTH {
        buffer[xy_to_one_d(Point::new(x, 0))] = 0;
        buffer[xy_to_one_d(Point::new(x, 0)) + 1] = 0;
        buffer[xy_to_one_d(Point::new(x, 0)) + 2] = 0;
        buffer[xy_to_one_d(Point::new(x, HEIGHT - 1))] = 0;
        buffer[xy_to_one_d(Point::new(x, HEIGHT - 1)) + 1] = 0;
        buffer[xy_to_one_d(Point::new(x, HEIGHT - 1)) + 2] = 0;
    }
    for y in 0..HEIGHT {
        buffer[xy_to_one_d(Point::new(0, y))] = 0;
        buffer[xy_to_one_d(Point::new(0, y)) + 1] = 0;
        buffer[xy_to_one_d(Point::new(0, y)) + 2] = 0;
        buffer[xy_to_one_d(Point::new(WIDTH - 1, y))] = 0;
        buffer[xy_to_one_d(Point::new(WIDTH - 1, y)) + 1] = 0;
        buffer[xy_to_one_d(Point::new(WIDTH - 1, y)) + 2] = 0;
    }
}

fn main() {
    // store the current frame
    let mut buffer: Vec<u8> = vec![0; (WIDTH * HEIGHT * 3) as usize];
    // keep the previous frame
    let mut prev_buffer: Vec<u8> = vec![0; (WIDTH * HEIGHT * 3) as usize];
    let mut rng = rand::rng();

    let sdl_context = sdl3::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Forest Fire Sim", WIDTH, HEIGHT)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas();

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    let texture_creator = canvas.texture_creator();
    let surface = Surface::new(WIDTH, HEIGHT, PixelFormat::RGB24).unwrap();
    let mut texture = Texture::from_surface(&surface, &texture_creator).unwrap();

    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut frame_count = 0;
    let mut run = false;

    for _ in 0..((HEIGHT as f32 * WIDTH as f32) * 0.1) as usize {
        set_tree(rng.random_range(0..(HEIGHT * WIDTH) as usize), &mut buffer);
    }

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
                let spawn_point = rng.random_range(0..(WIDTH * HEIGHT) as usize);
                if buffer[spawn_point] == 0 {
                    set_tree(spawn_point, &mut buffer);
                }
            }

            // every other frame spawn a lightning on rng location
            if frame_count >= LIGHTNING_SPAWN_RATE {
                let spawn_point = rng.random_range(0..(WIDTH * HEIGHT) as usize);
                if is_tree(spawn_point, &buffer) {
                    set_fire(spawn_point, &mut buffer);
                    frame_count = 0;
                }
            }

            // set edge pixel to "0" so the burn_trees function doesnt check out of range pixel
            // locations
            delete_edge(&mut buffer);
            // spread the fire
            burn_trees(&mut buffer);
            // clean fire so only edge fire remains
            delete_fire(&mut buffer, &prev_buffer);

            texture.update(None, &buffer, WIDTH as usize * 3).unwrap();
            canvas.copy(&texture, None, None).unwrap();
            canvas.present();
            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));

            // copy current buffer into prev_buffer
            prev_buffer.copy_from_slice(&buffer);
            // increase frame_count for frame depending lightning spawn
            frame_count += 1;
        } else {
            texture.update(None, &buffer, WIDTH as usize * 3).unwrap();
            canvas.copy(&texture, None, None).unwrap();
            canvas.present();
            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));
        }
    }
}
