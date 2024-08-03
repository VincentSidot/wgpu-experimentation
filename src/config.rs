use clap::{Parser, ValueEnum};
use serde::Serialize;
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
pub struct Config {
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
    #[arg(short = 'p', long = "present-mode", default_value = "default")]
    present_mode: PresentModeConfig,
}

pub struct AppConfig {
    pub window_title: String,
    pub window_size: WindowSizeConfig,
    pub present_mode: PresentModeConfig,
}

/// Enum to hold the different window sizes
#[derive(Debug, Clone, ValueEnum, Serialize)]
#[serde(rename_all = "snake_case")]
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

#[derive(Debug, Default, Clone, ValueEnum, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PresentModeConfig {
    #[clap(help = "Chooses FifoRelaxed -> Fifo based on availability.")]
    AutoVsync = 0,
    #[clap(
        help = "Chooses Immediate -> Mailbox -> Fifo (on web) based on availability."
    )]
    AutoNoVsync = 1,
    #[clap(help = "Supported on all platforms.")]
    Fifo = 2,
    #[clap(help = "Supported on AMD on Vulkan.")]
    FifoRelaxed = 3,
    #[clap(
        help = "Supported on most platforms except older DX12 and Wayland."
    )]
    Immediate = 4,
    #[clap(
        help = "Supported on DX12 on Windows 10, NVidia on Vulkan and Wayland on Vulkan."
    )]
    Mailbox = 5,
    #[default]
    #[clap(help = "Uses the first available present mode.")]
    Default = 6,
}

impl PresentModeConfig {
    pub fn to_wgpu_present_mode(
        &self,
        surface_caps: &SurfaceCapabilities,
    ) -> Result<PresentMode, &'static str> {
        let fallback_present_mode = *surface_caps
            .present_modes
            .first()
            .ok_or("No present modes found")?;
        match self {
            PresentModeConfig::Default => {
                log::trace!(
                    "Using default present mode: {:?}",
                    fallback_present_mode
                );
                Ok(fallback_present_mode)
            }
            present_mode => {
                let wanted_present_mode = match present_mode {
                    PresentModeConfig::AutoVsync => PresentMode::Fifo,
                    PresentModeConfig::AutoNoVsync => PresentMode::Immediate,
                    PresentModeConfig::Fifo => PresentMode::Fifo,
                    PresentModeConfig::FifoRelaxed => PresentMode::FifoRelaxed,
                    PresentModeConfig::Immediate => PresentMode::Immediate,
                    PresentModeConfig::Mailbox => PresentMode::Mailbox,
                    PresentModeConfig::Default => unreachable!(),
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

impl Default for WindowSizeConfig {
    fn default() -> Self {
        Self {
            size: (800, 600),
            hint: WindowSizeHint::Windowed,
        }
    }
}

impl Config {
    pub fn compute(self) -> AppConfig {
        let window_size = WindowSizeConfig {
            hint: self.screen_mode,
            size: (self.window_width, self.window_height),
        };
        AppConfig {
            window_title: self.window_title,
            window_size,
            present_mode: self.present_mode,
        }
    }

    pub fn init() -> Self {
        Self::parse()
    }
}
