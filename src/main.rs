mod simulation;
mod window;

use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use crate::simulation::Simulation;
use crate::window::Viewport;

const WIDTH: u32 = 1920;
const WIDTH_SUBPIXEL: usize = WIDTH as usize * 3;
const HEIGHT: u32 = 1080;
const PIXEL_COUNT: usize = (WIDTH * HEIGHT) as usize;
const BUFFER_SIZE: usize = PIXEL_COUNT * 3;

// Change these to alter the simulation
const TREE_SPAWN_RATE: u32 = 10;
const LIGHTNING_SPAWN_RATE: u32 = 150;
const SIM_SPEED: u32 = 200;

fn main() {
    let quit = Arc::new(AtomicBool::new(false));

    let simulation = Simulation::new();
    let simulation_buffer = simulation.get_buffer();
    let sim_thread = simulation.run(Arc::clone(&quit));

    let mut viewport = Viewport::new();
    viewport.draw(&simulation_buffer, Arc::clone(&quit));

    sim_thread.join().unwrap();
}
