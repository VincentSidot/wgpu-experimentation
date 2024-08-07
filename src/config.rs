use clap::{Parser, ValueEnum};
use wgpu::{PresentMode, SurfaceCapabilities};

/// Config struct for the application
///
/// This struct will hold all the configuration for the application
/// and will be used to initialize the application
#[derive(Parser)]
#[command(
    version = "0.0.1",
    about = "Small Rust Program using WGPU",
    long_about = "Small Rust Program using WGPU, with an EGUI interface built on top of it to display simple debug information",
    author = "Vincent S."
)]
pub struct ClapConfig {
    /// Title of the window
    #[arg(short = 't', long = "title", default_value = "Test WGPU")]
    window_title: String,

    /// Window size configuration
    #[arg(short = 'm', long = "mode", default_value = "windowed")]
    screen_mode: WindowSizeHint,

    /// Window size value
    #[arg(short = 'e', long = "height", default_value = "600")]
    window_height: u32,

    /// Window size value
    #[arg(short = 'w', long = "width", default_value = "800")]
    window_width: u32,

    /// Present mode configuration
    #[arg(short = 'p', long = "present-mode")]
    present_mode: Option<PresentModeConfig>,

    /// Backend selection
    #[arg(short = 'b', long = "backend")]
    backend: Option<BackendSelection>,

    /// Disable EGUI Rendering
    #[arg(short = 'd', long = "disable-egui")]
    disable_egui: bool,
}

pub struct Config {
    pub backends: wgpu::Backends,
    pub disable_egui: bool,
    pub present_mode: Option<PresentModeConfig>,
    pub window_title: String,
    pub window_size: WindowSizeConfig,
}

/// Enum to hold the different window sizes
#[derive(Debug, Clone, ValueEnum)]
pub enum WindowSizeHint {
    #[clap(help = "Fullscreen mode with borders.")]
    Fullscreen,
    #[clap(help = "Fullscreen mode without borders.")]
    FullscreenBorderless,
    #[clap(help = "Windowed mode.")]
    Windowed,
}

#[derive(Debug)]
pub struct WindowSizeConfig {
    pub hint: WindowSizeHint,
    pub size: (u32, u32),
}

#[derive(Debug, Clone, ValueEnum)]
pub enum PresentModeConfig {
    #[clap(help = "Chooses FifoRelaxed -> Fifo based on availability.")]
    AutoVsync,
    #[clap(
        help = "Chooses Immediate -> Mailbox -> Fifo (on web) based on availability."
    )]
    AutoNoVsync,
    #[clap(help = "Supported on all platforms.")]
    Fifo,
    #[clap(help = "Supported on AMD on Vulkan.")]
    FifoRelaxed,
    #[clap(
        help = "Supported on most platforms except older DX12 and Wayland."
    )]
    Immediate,
    #[clap(
        help = "Supported on DX12 on Windows 10, NVidia on Vulkan and Wayland on Vulkan."
    )]
    Mailbox,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum BackendSelection {
    #[clap(
        help = "Use the Vulkan backend. (Windows, Linux, Android, MacOS via vulkan-portability/MoltenVK)"
    )]
    Vulkan,
    #[clap(help = "Use the Metal backend. (Apple platforms)")]
    Metal,
    #[clap(help = "Use the DX12 backend. (Windows)")]
    DirectX12,
    #[clap(
        help = "Use the OpenGL backend. (Windows, Linux, Android, MacOS via Angle) (Not recommended)"
    )]
    OpenGL,
}

fn to_wgpu_backend(value: Option<BackendSelection>) -> wgpu::Backends {
    match value {
        Some(value) => match value {
            BackendSelection::Vulkan => wgpu::Backends::VULKAN,
            BackendSelection::Metal => wgpu::Backends::METAL,
            BackendSelection::DirectX12 => wgpu::Backends::DX12,
            BackendSelection::OpenGL => wgpu::Backends::GL,
        },
        None => wgpu::Backends::all(),
    }
}

impl Default for WindowSizeConfig {
    fn default() -> Self {
        Self {
            size: (800, 600),
            hint: WindowSizeHint::Windowed,
        }
    }
}

impl Config {
    pub fn init() -> Self {
        ClapConfig::init().compute()
    }

    pub fn to_wgpu_present_mode(
        &self,
        surface_caps: &SurfaceCapabilities,
    ) -> Result<PresentMode, &'static str> {
        let fallback_present_mode = *surface_caps
            .present_modes
            .first()
            .ok_or("No present modes found")?;
        match &self.present_mode {
            None => {
                log::trace!(
                    "Using default present mode: {:?}",
                    fallback_present_mode
                );
                Ok(fallback_present_mode)
            }
            Some(present_mode) => {
                let wanted_present_mode = match present_mode {
                    PresentModeConfig::AutoVsync => PresentMode::Fifo,
                    PresentModeConfig::AutoNoVsync => PresentMode::Immediate,
                    PresentModeConfig::Fifo => PresentMode::Fifo,
                    PresentModeConfig::FifoRelaxed => PresentMode::FifoRelaxed,
                    PresentModeConfig::Immediate => PresentMode::Immediate,
                    PresentModeConfig::Mailbox => PresentMode::Mailbox,
                };
                // Ensure the requested present mode is supported
                if surface_caps.present_modes.contains(&wanted_present_mode) {
                    log::trace!("Using requested present mode");
                    Ok(wanted_present_mode)
                } else {
                    log::warn!(
                        "Requested present mode {:?} not supported, falling back to {:?}",
                        wanted_present_mode,
                        fallback_present_mode
                    );
                    Ok(fallback_present_mode)
                }
            }
        }
    }
}

impl ClapConfig {
    fn compute(self) -> Config {
        Config {
            backends: to_wgpu_backend(self.backend),
            disable_egui: self.disable_egui,
            present_mode: self.present_mode,
            window_title: self.window_title,
            window_size: WindowSizeConfig {
                hint: self.screen_mode,
                size: (self.window_width, self.window_height),
            },
        }
    }

    pub fn init() -> ClapConfig {
        Self::parse()
    }
}
