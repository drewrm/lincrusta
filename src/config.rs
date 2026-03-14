use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum Layer {
    #[default]
    Background,
    Bottom,
    Top,
    Overlay,
}

impl From<&str> for Layer {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "bottom" => Layer::Bottom,
            "top" => Layer::Top,
            "overlay" => Layer::Overlay,
            _ => Layer::Background,
        }
    }
}

impl AsRef<str> for Layer {
    fn as_ref(&self) -> &'static str {
        match self {
            Layer::Background => "background",
            Layer::Bottom => "bottom",
            Layer::Top => "top",
            Layer::Overlay => "overlay",
        }
    }
}

impl Layer {
    pub fn is_valid(s: &str) -> bool {
        matches!(
            s.to_lowercase().as_str(),
            "background" | "bottom" | "top" | "overlay"
        )
    }

    pub fn valid_options() -> Vec<&'static str> {
        vec!["background", "bottom", "top", "overlay"]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum Ordering {
    Random,
    #[default]
    Sequential,
}

impl From<&str> for Ordering {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "sequential" => Ordering::Sequential,
            _ => Ordering::Random,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum TransitionType {
    #[default]
    Crossfade,
    None,
    SlideRight,
    SlideLeft,
    SlideUp,
    SlideDown,
    SlideLeftRight,
    SlideUpDown,
    OverUp,
    OverDown,
    OverLeft,
    OverRight,
    UnderUp,
    UnderDown,
    UnderLeft,
    UnderRight,
    RotateLeft,
    RotateRight,
    RotateLeftRight,
}

impl From<&str> for TransitionType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "none" => TransitionType::None,
            "slide_right" => TransitionType::SlideRight,
            "slide_left" => TransitionType::SlideLeft,
            "slide_up" => TransitionType::SlideUp,
            "slide_down" => TransitionType::SlideDown,
            "slide_left_right" => TransitionType::SlideLeftRight,
            "slide_up_down" => TransitionType::SlideUpDown,
            "over_up" => TransitionType::OverUp,
            "over_down" => TransitionType::OverDown,
            "over_left" => TransitionType::OverLeft,
            "over_right" => TransitionType::OverRight,
            "under_up" => TransitionType::UnderUp,
            "under_down" => TransitionType::UnderDown,
            "under_left" => TransitionType::UnderLeft,
            "under_right" => TransitionType::UnderRight,
            "rotate_left" => TransitionType::RotateLeft,
            "rotate_right" => TransitionType::RotateRight,
            "rotate_left_right" => TransitionType::RotateLeftRight,
            _ => TransitionType::Crossfade,
        }
    }
}

impl AsRef<str> for TransitionType {
    fn as_ref(&self) -> &'static str {
        match self {
            TransitionType::Crossfade => "crossfade",
            TransitionType::None => "none",
            TransitionType::SlideRight => "slide_right",
            TransitionType::SlideLeft => "slide_left",
            TransitionType::SlideUp => "slide_up",
            TransitionType::SlideDown => "slide_down",
            TransitionType::SlideLeftRight => "slide_left_right",
            TransitionType::SlideUpDown => "slide_up_down",
            TransitionType::OverUp => "over_up",
            TransitionType::OverDown => "over_down",
            TransitionType::OverLeft => "over_left",
            TransitionType::OverRight => "over_right",
            TransitionType::UnderUp => "under_up",
            TransitionType::UnderDown => "under_down",
            TransitionType::UnderLeft => "under_left",
            TransitionType::UnderRight => "under_right",
            TransitionType::RotateLeft => "rotate_left",
            TransitionType::RotateRight => "rotate_right",
            TransitionType::RotateLeftRight => "rotate_left_right",
        }
    }
}

impl TransitionType {
    pub fn is_valid(s: &str) -> bool {
        matches!(
            s.to_lowercase().as_str(),
            "none"
                | "crossfade"
                | "slide_right"
                | "slide_left"
                | "slide_up"
                | "slide_down"
                | "slide_left_right"
                | "slide_up_down"
                | "over_up"
                | "over_down"
                | "over_left"
                | "over_right"
                | "under_up"
                | "under_down"
                | "under_left"
                | "under_right"
                | "rotate_left"
                | "rotate_right"
                | "rotate_left_right"
        )
    }

    pub fn valid_options() -> Vec<&'static str> {
        vec![
            "none",
            "crossfade",
            "slide_right",
            "slide_left",
            "slide_up",
            "slide_down",
            "slide_left_right",
            "slide_up_down",
            "over_up",
            "over_down",
            "over_left",
            "over_right",
            "under_up",
            "under_down",
            "under_left",
            "under_right",
            "rotate_left",
            "rotate_right",
            "rotate_left_right",
        ]
    }
}

#[derive(Clone)]
pub struct Config {
    pub wallpaper_path: Option<PathBuf>,
    pub refresh_interval: u64,
    pub ordering: Ordering,
    pub transition_type: TransitionType,
    pub layer: Layer,
}
