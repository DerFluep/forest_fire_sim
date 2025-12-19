use minifb::{Key, Window, WindowOptions};
use rand::prelude::*;

const WIDTH: usize = 1000;
const HEIGHT: usize = 1000;

const TREE: u32 = 65280;
const FIRE: u32 = 16711680;

struct Point {
    x: u32,
    y: u32,
}

impl Point {
    fn new(x: u32, y: u32) -> Point {
        Point { x, y }
    }
}

fn xy_to_one_d(point: Point) -> usize {
    point.x as usize + point.y as usize * WIDTH
}

fn one_d_to_xy(index: usize) -> Point {
    let x = index % WIDTH;
    let y = index / WIDTH;
    Point::new(x as u32, y as u32)
}

fn _from_u8_rgb(r: u8, g: u8, b: u8) -> u32 {
    let (r, g, b) = (r as u32, g as u32, b as u32);
    (r << 16) | (g << 8) | b
}

fn burn_trees(buffer: &mut [u32]) {
    let mut tmp_buffer = vec![0; WIDTH * HEIGHT];
    tmp_buffer.copy_from_slice(buffer);
    for i in 0..buffer.len() {
        if buffer[i] == FIRE {
            // check surrounding
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
                if buffer[*index] == TREE {
                    tmp_buffer[*index] = FIRE;
                }
            }
        }
    }
    buffer.copy_from_slice(&tmp_buffer);
}

fn delete_fire(buffer: &mut [u32], prev_frame: &[u32]) {
    for n in 0..buffer.len() {
        if prev_frame[n] == FIRE {
            buffer[n] = 0;
        }
    }
}

fn delete_edge(buffer: &mut [u32]) {
    for x in 0..WIDTH {
        buffer[xy_to_one_d(Point::new(x as u32, 0))] = 0;
        buffer[xy_to_one_d(Point::new(x as u32, HEIGHT as u32 - 1))] = 0;
    }
    for y in 0..HEIGHT {
        buffer[xy_to_one_d(Point::new(0, y as u32))] = 0;
        buffer[xy_to_one_d(Point::new(WIDTH as u32 - 1, y as u32))] = 0;
    }
}

fn main() {
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    let mut prev_buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    let mut rng = rand::rng();

    let tree_spawn = (WIDTH * HEIGHT) / 5000;
    let lightning_spawn = 6250000 / (WIDTH * HEIGHT);

    let mut window = Window::new(
        "Forest Fire Simulator",
        WIDTH,
        HEIGHT,
        WindowOptions {
            borderless: (true),
            title: (true),
            resize: (true),
            scale: (minifb::Scale::FitScreen),
            scale_mode: (minifb::ScaleMode::AspectRatioStretch),
            topmost: (true),
            transparency: (false),
            none: (false),
        },
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    window.set_target_fps(60);

    let mut frame_count = 0;
    let mut run = false;

    for _ in 0..((HEIGHT as f32 * WIDTH as f32) * 0.1) as usize {
        buffer[rng.random_range(0..HEIGHT * WIDTH)] = TREE;
    }

    while window.is_open() && !window.is_key_down(Key::Escape) {
        if window.is_key_down(Key::Space) {
            run = !run;
        }
        if run {
            for _ in 0..tree_spawn {
                let spawn_point = rng.random_range(0..WIDTH * HEIGHT);
                if buffer[spawn_point] == 0 {
                    buffer[spawn_point] = TREE;
                }
            }

            if frame_count >= lightning_spawn {
                buffer[rng.random_range(0..WIDTH * HEIGHT)] = FIRE;
                frame_count = 0;
            }

            delete_edge(&mut buffer);
            burn_trees(&mut buffer);
            delete_fire(&mut buffer, &prev_buffer);

            window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
            prev_buffer.copy_from_slice(&buffer);
            frame_count += 1;
        } else {
            window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
        }
    }
}
