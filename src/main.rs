// use test_wgpu::hello;

use test_wgpu::run;

use test_wgpu::Config;
use test_wgpu::Logger;

fn main() {
    // Setup logger
    Logger::setup().expect("Failed to setup logger");

    let config = Config::init();

    pollster::block_on(run(config)).expect("Failed to run the application");
}
