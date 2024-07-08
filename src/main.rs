// use test_wgpu::hello;

use test_wgpu::run;
use regex::Regex;

use test_wgpu::WindowSize;

fn main() {
    let size_regex = Regex::new(r"^(\d+).(\d+)$").expect("Failed to create regex");

    let args = std::env::args().collect::<Vec<_>>();

    let size = if args.len() == 3 {
        let width = args[1].parse().expect("Failed to parse width");
        let height = args[2].parse().expect("Failed to parse height");
        WindowSize::Windowed(width, height)
    } else if args.len() == 2 {
        let size = args[1].as_str();
        
        if size == "fullscreen" {
            WindowSize::FullScreen
        } else if size == "borderless" {
            WindowSize::FullScreenBorderless
        } else if size_regex.is_match(size) {
            let captures = size_regex.captures(size).unwrap();
            let width = captures.get(1).unwrap().as_str().parse().expect("Failed to parse width");
            let height = captures.get(2).unwrap().as_str().parse().expect("Failed to parse height");
            WindowSize::Windowed(width, height)
        
        } else {
            WindowSize::Default
        }
    } else {
        WindowSize::Default
    };

    println!("Size: {:?}", size);

    pollster::block_on(run(&size)).expect("Failed to run the application");
}