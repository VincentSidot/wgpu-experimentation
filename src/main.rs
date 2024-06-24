// use test_wgpu::hello;

use test_wgpu::run;

fn main() {
    pollster::block_on(run()).expect("Failed to run the application")
}