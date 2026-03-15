use clap::{Parser, Subcommand, ValueEnum};
use log::error;
use std::process::ExitCode;
use zbus::blocking::Connection;

#[derive(Parser)]
#[command(
    name = "wallpaper-cli",
    about = "Control the wallpaper daemon via D-Bus",
    long_about = "Control the wallpaper daemon via D-Bus

Examples:
  wallpaper-cli path /path/to/image.jpg
  wallpaper-cli refresh-interval 30
  wallpaper-cli ordering random
  wallpaper-cli transition-type slide_left
  wallpaper-cli layer overlay"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Set wallpaper path (image file or directory for slideshow)")]
    Path {
        #[arg(help = "Path to wallpaper image or directory")]
        path: String,
    },
    #[command(about = "Set refresh interval between wallpaper changes")]
    RefreshInterval {
        #[arg(help = "Interval in seconds", value_parser = clap::value_parser!(u32).range(1..))]
        seconds: u32,
    },
    #[command(about = "Set slideshow ordering")]
    Ordering {
        #[arg(help = "Ordering: random or sequential", value_enum)]
        order: Ordering,
    },
    #[command(about = "Set transition effect")]
    TransitionType {
        #[arg(help = "Transition effect", value_enum)]
        effect: TransitionTypeArg,
    },
    #[command(about = "Set layer shell layer")]
    Layer {
        #[arg(help = "Layer: background, bottom, top, or overlay", value_enum)]
        layer: LayerArg,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum Ordering {
    Random,
    Sequential,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum TransitionTypeArg {
    #[clap(name = "none")]
    None,
    #[clap(name = "crossfade")]
    Crossfade,
    #[clap(name = "slide_right")]
    SlideRight,
    #[clap(name = "slide_left")]
    SlideLeft,
    #[clap(name = "slide_up")]
    SlideUp,
    #[clap(name = "slide_down")]
    SlideDown,
    #[clap(name = "slide_left_right")]
    SlideLeftRight,
    #[clap(name = "slide_up_down")]
    SlideUpDown,
    #[clap(name = "over_up")]
    OverUp,
    #[clap(name = "over_down")]
    OverDown,
    #[clap(name = "over_left")]
    OverLeft,
    #[clap(name = "over_right")]
    OverRight,
    #[clap(name = "under_up")]
    UnderUp,
    #[clap(name = "under_down")]
    UnderDown,
    #[clap(name = "under_left")]
    UnderLeft,
    #[clap(name = "under_right")]
    UnderRight,
    #[clap(name = "rotate_left")]
    RotateLeft,
    #[clap(name = "rotate_right")]
    RotateRight,
    #[clap(name = "rotate_left_right")]
    RotateLeftRight,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum LayerArg {
    Background,
    Bottom,
    Top,
    Overlay,
}

const DBUS_BUS_NAME: &str = "org.drewrm.wallpaperd";
const DBUS_OBJECT_PATH: &str = "/org/drewrm/wallpaperd";
const DBUS_INTERFACE: &str = "org.drewrm.wallpaperd";

fn call_method(conn: &Connection, method: &str, value: &str) -> Result<String, String> {
    let proxy = zbus::blocking::Proxy::new(conn, DBUS_BUS_NAME, DBUS_OBJECT_PATH, DBUS_INTERFACE)
        .map_err(|e| e.to_string())?;

    proxy
        .call(method, &(value.to_string()))
        .map_err(|e| e.to_string())
}

fn call_method_u32(conn: &Connection, method: &str, value: u32) -> Result<String, String> {
    let proxy = zbus::blocking::Proxy::new(conn, DBUS_BUS_NAME, DBUS_OBJECT_PATH, DBUS_INTERFACE)
        .map_err(|e| e.to_string())?;

    proxy.call(method, &(value)).map_err(|e| e.to_string())
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    env_logger::init();

    let conn = match Connection::session() {
        Ok(c) => c,
        Err(e) => {
            error!("Error connecting to D-Bus: {}", e);
            return ExitCode::FAILURE;
        }
    };

    match cli.command {
        Commands::Path { path } => match call_method(&conn, "SetWallpaper", &path) {
            Ok(result) => println!("{}", result),
            Err(e) => {
                error!("Error: {}", e);
                return ExitCode::FAILURE;
            }
        },
        Commands::RefreshInterval { seconds } => {
            match call_method_u32(&conn, "SetRefreshInterval", seconds) {
                Ok(result) => println!("{}", result),
                Err(e) => {
                    error!("Error: {}", e);
                    return ExitCode::FAILURE;
                }
            }
        }
        Commands::Ordering { order } => {
            let value = match order {
                Ordering::Random => "random",
                Ordering::Sequential => "sequential",
            };
            match call_method(&conn, "SetOrdering", value) {
                Ok(result) => println!("{}", result),
                Err(e) => {
                    error!("Error: {}", e);
                    return ExitCode::FAILURE;
                }
            }
        }
        Commands::TransitionType { effect } => {
            let value = match effect {
                TransitionTypeArg::None => "none",
                TransitionTypeArg::Crossfade => "crossfade",
                TransitionTypeArg::SlideRight => "slide_right",
                TransitionTypeArg::SlideLeft => "slide_left",
                TransitionTypeArg::SlideUp => "slide_up",
                TransitionTypeArg::SlideDown => "slide_down",
                TransitionTypeArg::SlideLeftRight => "slide_left_right",
                TransitionTypeArg::SlideUpDown => "slide_up_down",
                TransitionTypeArg::OverUp => "over_up",
                TransitionTypeArg::OverDown => "over_down",
                TransitionTypeArg::OverLeft => "over_left",
                TransitionTypeArg::OverRight => "over_right",
                TransitionTypeArg::UnderUp => "under_up",
                TransitionTypeArg::UnderDown => "under_down",
                TransitionTypeArg::UnderLeft => "under_left",
                TransitionTypeArg::UnderRight => "under_right",
                TransitionTypeArg::RotateLeft => "rotate_left",
                TransitionTypeArg::RotateRight => "rotate_right",
                TransitionTypeArg::RotateLeftRight => "rotate_left_right",
            };
            match call_method(&conn, "SetTransitionType", value) {
                Ok(result) => println!("{}", result),
                Err(e) => {
                    error!("Error: {}", e);
                    return ExitCode::FAILURE;
                }
            }
        }
        Commands::Layer { layer } => {
            let value = match layer {
                LayerArg::Background => "background",
                LayerArg::Bottom => "bottom",
                LayerArg::Top => "top",
                LayerArg::Overlay => "overlay",
            };
            match call_method(&conn, "SetLayer", value) {
                Ok(result) => println!("{}", result),
                Err(e) => {
                    error!("Error: {}", e);
                    return ExitCode::FAILURE;
                }
            }
        }
    }

    ExitCode::SUCCESS
}
