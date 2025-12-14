use minifb::{Key, Window, WindowOptions};
use rand::prelude::*;

const WIDTH: usize = 500;
const HEIGHT: usize = 500;

const TREE: u32 = 16711680;
const FIRE: u32 = 65280;

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

pub fn from_u8_rgb(r: u8, g: u8, b: u8) -> u32 {
    let (r, g, b) = (r as u32, g as u32, b as u32);
    (r << 16) | (g << 8) | b
}

fn burn_trees(buffer: &mut Vec<u32>) {
    for (n, i) in buffer.iter_mut().enumerate() {
        if *i == TREE {
            // check surrounding
            let start_point = one_d_to_xy(n);
            let positions = [
                // top_left
                xy_to_one_d(Point::new(start_point.x - 1, start_point.y + 1)),
                // top
                xy_to_one_d(Point::new(start_point.x, start_point.y + 1)),
                // top_right
                xy_to_one_d(Point::new(start_point.x + 1, start_point.y + 1)),
                // right
                xy_to_one_d(Point::new(start_point.x + 1, start_point.y)),
                // down_right
                xy_to_one_d(Point::new(start_point.x + 1, start_point.y - 1)),
                // down
                xy_to_one_d(Point::new(start_point.x, start_point.y - 1)),
                // down_left
                xy_to_one_d(Point::new(start_point.x - 1, start_point.y - 1)),
                // left
                xy_to_one_d(Point::new(start_point.x - 1, start_point.y)),
            ];

            let mut is_surrounded = false;
            for index in positions.iter() {
                if buffer[*index] == FIRE {
                    is_surrounded = true;
                }
            }
            if is_surrounded {
                *i = FIRE;
            }
        }
    }
}

fn main() {
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    let mut rng = rand::rng();

    let mut window = Window::new(
        "Forest Fire Simulator",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    window.set_target_fps(60);

    println!("red: {}", from_u8_rgb(255, 0, 0));
    println!("green: {}", from_u8_rgb(0, 255, 0));

    let mut frame_count = 0;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        if frame_count >= 100 {
            burn_trees(&mut buffer);
            frame_count = 0;
        }
        for _ in 0..10 {
            buffer[rng.random_range(0..WIDTH * HEIGHT)] = from_u8_rgb(0, 255, 0);
        }
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
        frame_count += 1;
    }
}
