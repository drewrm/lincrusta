use glib::ControlFlow;
use gtk::{Application, Window, Builder, glib::source, prelude::*};
use gtk4 as gtk;
use gtk4::StackTransitionType;
use gtk4_layer_shell::{Layer, LayerShell};
use log::error;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use wallpaper::config::Layer as ConfigLayer;
use wallpaper::config::Ordering as ConfigOrdering;
use wallpaper::config::TransitionType;
use wallpaper::config_service::{
    get_layer, get_ordering, get_refresh_interval, get_transition_type, get_wallpaper_dir,
    get_wallpaper_path, load_config,
};
use wallpaper::ordering::{RandomOrdering, SequentialOrdering, get_next_image};

fn to_gtk_transition(t: TransitionType) -> StackTransitionType {
    use wallpaper::config::TransitionType as T;
    match t {
        T::None => StackTransitionType::None,
        T::Crossfade => StackTransitionType::Crossfade,
        T::SlideRight => StackTransitionType::SlideRight,
        T::SlideLeft => StackTransitionType::SlideLeft,
        T::SlideUp => StackTransitionType::SlideUp,
        T::SlideDown => StackTransitionType::SlideDown,
        T::SlideLeftRight => StackTransitionType::SlideLeftRight,
        T::SlideUpDown => StackTransitionType::SlideUpDown,
        T::OverUp => StackTransitionType::OverUp,
        T::OverDown => StackTransitionType::OverDown,
        T::OverLeft => StackTransitionType::OverLeft,
        T::OverRight => StackTransitionType::OverRight,
        T::UnderUp => StackTransitionType::UnderUp,
        T::UnderDown => StackTransitionType::UnderDown,
        T::UnderLeft => StackTransitionType::UnderLeft,
        T::UnderRight => StackTransitionType::UnderRight,
        T::RotateLeft => StackTransitionType::RotateLeft,
        T::RotateRight => StackTransitionType::RotateRight,
        T::RotateLeftRight => StackTransitionType::RotateLeftRight,
    }
}

fn to_gtk_layer(l: ConfigLayer) -> Layer {
    use wallpaper::config::Layer as L;
    match l {
        L::Bottom => Layer::Bottom,
        L::Top => Layer::Top,
        L::Overlay => Layer::Overlay,
        _ => Layer::Background,
    }
}

fn build_ui(application: &Application) {
    let config = load_config();

    let builder = Builder::from_string(include_str!("../../resources/wallpaper-window.xml"));
    let window: Window = builder
        .object("wallpaper-window")
        .expect("Couldn't load window");

    LayerShell::init_layer_shell(&window);
    window.set_application(Some(application));
    window.set_namespace(Some("wallpaperd"));
    window.set_layer(to_gtk_layer(config.layer));
    window.set_anchor(gtk4_layer_shell::Edge::Top, true);
    window.set_anchor(gtk4_layer_shell::Edge::Bottom, true);
    window.set_anchor(gtk4_layer_shell::Edge::Left, true);
    window.set_anchor(gtk4_layer_shell::Edge::Right, true);
    window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::None);
    window.set_exclusive_zone(-1);


    let stack: gtk::Stack = builder
        .object("wallpaper-stack")
        .expect("Couldn't load stack");

    let picture1: gtk::Picture = builder
        .object("wallpaper-picture-1")
        .expect("Couldn't load picture 1");

    let picture2: gtk::Picture = builder
        .object("wallpaper-picture-2")
        .expect("Couldn't load picture 2");

    let picture1_init = picture1.clone();

    let wallpaper_path = get_wallpaper_path();
    let wallpaper_dir = get_wallpaper_dir();
    let refresh_interval_store = get_refresh_interval();
    let ordering_store = get_ordering();
    let transition_type_store = get_transition_type();
    let layer_store = get_layer();
    let last_path: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);
    let current_picture: std::sync::Mutex<u32> = std::sync::Mutex::new(1);

    let slideshow_dir = config.wallpaper_path.clone();
    let config_refresh_interval = config.refresh_interval;
    let config_ordering = config.ordering;
    let config_transition_type = config.transition_type;
    let config_layer = config.layer;
    let sequential_ordering =
        Arc::new(SequentialOrdering::new()) as Arc<dyn wallpaper::ordering::ImageOrdering>;

    let stack_for_transition = stack.clone();
    let set_stack_transition = move |transition: &str| {
        let tt = match transition {
            "none" => StackTransitionType::None,
            "slide_right" => StackTransitionType::SlideRight,
            "slide_left" => StackTransitionType::SlideLeft,
            "slide_up" => StackTransitionType::SlideUp,
            "slide_down" => StackTransitionType::SlideDown,
            "slide_left_right" => StackTransitionType::SlideLeftRight,
            "slide_up_down" => StackTransitionType::SlideUpDown,
            "over_up" => StackTransitionType::OverUp,
            "over_down" => StackTransitionType::OverDown,
            "over_left" => StackTransitionType::OverLeft,
            "over_right" => StackTransitionType::OverRight,
            "under_up" => StackTransitionType::UnderUp,
            "under_down" => StackTransitionType::UnderDown,
            "under_left" => StackTransitionType::UnderLeft,
            "under_right" => StackTransitionType::UnderRight,
            "rotate_left" => StackTransitionType::RotateLeft,
            "rotate_right" => StackTransitionType::RotateRight,
            "rotate_left_right" => StackTransitionType::RotateLeftRight,
            _ => StackTransitionType::Crossfade,
        };
        stack_for_transition.set_transition_type(tt);
    };

    {
        let dbus_transition_guard = transition_type_store.blocking_lock();
        let transition_str = dbus_transition_guard
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or(config_transition_type.as_ref());
        set_stack_transition(transition_str);
    }

    let window_for_layer = window.clone();
    let set_layer = move |layer: &str| {
        let gtk_layer = match layer {
            "background" => Layer::Background,
            "bottom" => Layer::Bottom,
            "top" => Layer::Top,
            "overlay" => Layer::Overlay,
            _ => Layer::Background,
        };
        LayerShell::set_layer(&window_for_layer, gtk_layer);
    };

    {
        let dbus_layer_guard = layer_store.blocking_lock();
        if let Some(layer) = dbus_layer_guard.as_ref() {
            set_layer(layer);
        } else {
            set_layer(config_layer.as_ref());
        }
    }

    let gtk_config_transition = to_gtk_transition(config_transition_type);
    stack.set_transition_type(gtk_config_transition);

    let load_wallpaper = || -> Option<String> {
        let active_dir = {
            let dbus_dir_guard = wallpaper_dir.blocking_lock();
            if dbus_dir_guard.is_some() {
                dbus_dir_guard.clone()
            } else if slideshow_dir.is_some() {
                slideshow_dir.clone()
            } else {
                None
            }
        };

        if let Some(dir) = active_dir {
            let is_sequential = {
                let dbus_ordering_guard = ordering_store.blocking_lock();
                match dbus_ordering_guard.as_ref() {
                    Some(o) => o == "sequential",
                    None => config_ordering == ConfigOrdering::Sequential,
                }
            };

            let strategy: Arc<dyn wallpaper::ordering::ImageOrdering> = if is_sequential {
                sequential_ordering.clone()
            } else {
                Arc::new(RandomOrdering) as Arc<dyn wallpaper::ordering::ImageOrdering>
            };

            get_next_image(&dir, strategy.as_ref())
        } else {
            let guard = wallpaper_path.blocking_lock();
            guard.as_ref().cloned()
        }
    };

    let stack_for_image = stack.clone();

    let set_image = move |path: &str| {
        let mut current = current_picture.lock().unwrap();
        if *current == 1 {
            picture2.set_filename(Some(path));
            stack_for_image.set_visible_child(&picture2);
            *current = 2;
        } else {
            picture1.set_filename(Some(path));
            stack_for_image.set_visible_child(&picture1);
            *current = 1;
        }
    };

    if let Some(path) = load_wallpaper()
        && std::path::Path::new(&path).exists()
    {
        picture1_init.set_filename(Some(&path));
        *last_path.lock().unwrap() = Some(path);
    }

    let last_slideshow_change = Arc::new(AtomicU64::new(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    ));

    source::timeout_add_local(std::time::Duration::from_millis(500), move || {
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        {
            let dbus_transition_guard = transition_type_store.blocking_lock();
            if let Some(transition) = dbus_transition_guard.as_ref() {
                set_stack_transition(transition);
            }
        }

        {
            let dbus_layer_guard = layer_store.blocking_lock();
            if let Some(layer) = dbus_layer_guard.as_ref() {
                set_layer(layer);
            }
        }

        let active_dir = {
            let dbus_dir_guard = wallpaper_dir.blocking_lock();
            if dbus_dir_guard.is_some() {
                dbus_dir_guard.clone()
            } else if slideshow_dir.is_some() {
                slideshow_dir.clone()
            } else {
                None
            }
        };

        if let Some(dir) = active_dir {
            let refresh_interval = {
                let dbus_interval_guard = refresh_interval_store.blocking_lock();
                dbus_interval_guard.unwrap_or(config_refresh_interval)
            };
            let last_change = last_slideshow_change.load(Ordering::Relaxed);
            if current_time - last_change >= refresh_interval {
                last_slideshow_change.store(current_time, Ordering::Relaxed);

                let is_sequential = {
                    let dbus_ordering_guard = ordering_store.blocking_lock();
                    match dbus_ordering_guard.as_ref() {
                        Some(o) => o == "sequential",
                        None => config_ordering == ConfigOrdering::Sequential,
                    }
                };

                let strategy: Arc<dyn wallpaper::ordering::ImageOrdering> = if is_sequential {
                    sequential_ordering.clone()
                } else {
                    Arc::new(RandomOrdering) as Arc<dyn wallpaper::ordering::ImageOrdering>
                };

                if let Some(path) = get_next_image(&dir, strategy.as_ref()) {
                    let mut last = last_path.lock().unwrap();
                    if last.as_ref() != Some(&path) {
                        *last = Some(path.clone());
                        if std::path::Path::new(&path).exists() {
                            set_image(&path);
                        }
                    }
                }
            }
        }
        ControlFlow::Continue
    });

    window.present();
}

fn main() {
    env_logger::init();
    wallpaper::config_service::load_config();

    let app = Application::builder()
        .application_id("org.drewrm.wallpaperd-ui")
        .build();

    std::thread::spawn(|| {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async {
            if let Err(e) = wallpaper::dbus_service::run_dbus_server().await {
                error!("DBus error: {}", e);
            }
        });
    });

    app.connect_activate(build_ui);

    app.run();
}
