use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use rand::Rng;

use crate::{
    BUFFER_SIZE, HEIGHT, LIGHTNING_SPAWN_RATE, PIXEL_COUNT, SIM_SPEED, TREE_SPAWN_RATE,
    WIDTH_SUBPIXEL,
};

fn is_tree(index: usize, buffer: &[u8]) -> bool {
    buffer[index] == 0 && buffer[index + 1] == 255 && buffer[index + 2] == 0
}

fn clear_pixel(buffer: &mut [u8], index: usize) {
    buffer[index] = 0;
    buffer[index + 1] = 0;
    buffer[index + 2] = 0;
}

fn burn_trees(buffer: &mut [u8], fire_buf: &mut Vec<usize>) {
    let mut fire_next: Vec<usize> = Vec::new();
    for index in fire_buf.iter() {
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
        for position in positions.iter() {
            if is_tree(*position, buffer) {
                clear_pixel(buffer, *position);
                fire_next.push(*position);
            }
        }
    }
    fire_buf.clear();
    fire_buf.append(&mut fire_next);
}

pub struct Simulation {
    buffer: Arc<Mutex<Vec<u8>>>,
}

impl Simulation {
    pub fn new() -> Simulation {
        Simulation {
            buffer: Arc::new(Mutex::new(vec![0; BUFFER_SIZE])),
        }
    }

    pub fn get_buffer(&self) -> Arc<Mutex<Vec<u8>>> {
        Arc::clone(&self.buffer)
    }

    fn set_tree(&mut self, index: usize) {
        let buffer = &mut self.buffer.lock().unwrap();
        buffer[index] = 0;
        buffer[index + 1] = 255;
        buffer[index + 2] = 0;
    }

    // fill every edge pixel with black
    fn clear_edges(&self) {
        let mut buffer = self.buffer.lock().unwrap();
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

    fn is_empty(&self, index: usize) -> bool {
        let buffer = &self.buffer.lock().unwrap();
        buffer[index] == 0 && buffer[index + 1] == 0 && buffer[index + 2] == 0
    }

    // // leave only edge fire and extinguish every other
    // fn clear_fire(&self, prev_frame: &[u8], fire_buf: &mut Vec<usize>) {
    //     for n in 0..fire_buf.len() {
    //         self.clear_pixel(fire_buf[n]);
    //     }
    // }

    pub fn run(self, quit: Arc<AtomicBool>) -> JoinHandle<()> {
        thread::spawn(move || {
            let mut simulation = self;
            let mut rng = rand::rng();
            let mut frame_count = 0;

            let mut fire_buf: Vec<usize> = Vec::new();

            for _ in 0..(PIXEL_COUNT as f32 * 0.5).round() as usize {
                simulation.set_tree(rng.random_range(0..PIXEL_COUNT) * 3);
            }
            simulation.clear_edges();

            'running: loop {
                if quit.load(Ordering::Relaxed) {
                    break 'running;
                }
                // continuously spawn trees
                for _ in 0..TREE_SPAWN_RATE {
                    let spawn_point = rng.random_range(0..PIXEL_COUNT) * 3;
                    if simulation.is_empty(spawn_point) {
                        simulation.set_tree(spawn_point);
                    }
                }
                // every other frame spawn a lightning on rng location
                let buffer = simulation.buffer.lock().unwrap();
                if frame_count >= LIGHTNING_SPAWN_RATE {
                    let spawn_point = rng.random_range(0..PIXEL_COUNT) * 3;
                    if is_tree(spawn_point, &buffer) {
                        fire_buf.push(spawn_point);
                        // set_fire(spawn_point, &mut buffer);
                        frame_count = 0;
                    }
                }
                drop(buffer);
                // clears edge pixel so the burn_trees function doesnt check out of range pixel locations
                simulation.clear_edges();
                // spread the fire
                let mut buffer = simulation.buffer.lock().unwrap();
                burn_trees(&mut buffer, &mut fire_buf);
                drop(buffer);

                // increase frame_count for frame depending lightning spawn
                frame_count += 1;
                ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / SIM_SPEED));
            }
        })
    }
}
