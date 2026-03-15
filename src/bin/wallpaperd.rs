use glib::ControlFlow;
use gtk::{Application, Builder, Window, glib::source, prelude::*};
use gtk4 as gtk;
use gtk4::StackTransitionType;
use gtk4_layer_shell::LayerShell;
use log::error;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use wallpaper::config::Ordering as ConfigOrdering;
use wallpaper::config_service::{
    get_layer, get_ordering, get_refresh_interval, get_transition_type, get_wallpaper_dir,
    load_config,
};
use wallpaper::ordering::{RandomOrdering, SequentialOrdering, get_next_image};

fn build_ui(application: &Application) {
    let config = load_config();

    let builder = Builder::from_string(include_str!("../../resources/wallpaper-window.xml"));
    let mut first_load = true;
    let window: Window = builder
        .object("wallpaper-window")
        .expect("Couldn't load window");

    LayerShell::init_layer_shell(&window);
    window.set_application(Some(application));
    window.set_namespace(Some("wallpaperd"));
    window.set_layer(config.layer.into());
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

    let wallpaper_dir = get_wallpaper_dir();
    let refresh_interval_store = get_refresh_interval();
    let ordering_store = get_ordering();
    let transition_type_store = get_transition_type();
    let layer_store = get_layer();
    let last_path: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);
    let current_picture: std::sync::Mutex<u32> = std::sync::Mutex::new(1);

    let sequential_ordering =
        Arc::new(SequentialOrdering::new()) as Arc<dyn wallpaper::ordering::ImageOrdering>;
    let random_ordering = Arc::new(RandomOrdering) as Arc<dyn wallpaper::ordering::ImageOrdering>;

    let slideshow_dir = config.wallpaper_path.clone();
    let config_refresh_interval = config.refresh_interval;

    let stack_clone = stack.clone();
    let set_stack_transition = move |config_transition_type: StackTransitionType| {
        stack_clone.set_transition_type(config_transition_type);
    };

    let window_for_layer = window.clone();
    let set_layer = move |layer: gtk4_layer_shell::Layer| {
        window_for_layer.set_layer(layer);
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
                set_stack_transition((*transition).into());
            }
        }

        {
            let dbus_layer_guard = layer_store.blocking_lock();
            if let Some(layer) = dbus_layer_guard.as_ref() {
                set_layer((*layer).into());
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
            if current_time - last_change >= refresh_interval || first_load {
                first_load = false;
                last_slideshow_change.store(current_time, Ordering::Relaxed);

                let strategy = match ordering_store.blocking_lock().as_ref().unwrap() {
                    ConfigOrdering::Sequential => sequential_ordering.clone(),
                    _ => random_ordering.clone(),
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
