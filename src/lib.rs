mod app;
mod config;
mod debug;
mod graphics;
mod render;
mod scene;
mod utils;

pub use app::App;
pub use config::Config;
pub use debug::widget::Logger;
pub use graphics::shapes::Shape;
pub use graphics::types::Instance;
pub use render::Renderer;
pub use scene::Scene;

pub use debug::widget;

macro_rules! elapsed_handler {
    ($item:expr => $block:expr) => {{
        let now = std::time::Instant::now();
        let ret = $block;
        let elapsed = now.elapsed();
        $item = elapsed;
        ret
    }};
}

pub(crate) use elapsed_handler;
