use gtk4::glib::ControlFlow;
use gtk::{Application, Builder, Window, glib::source, prelude::*};
use gtk4 as gtk;
use gtk4::StackTransitionType;
use gtk4_layer_shell::LayerShell;
use log::error;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use wallpaper::config::Ordering as ConfigOrdering;
use wallpaper::config_service::{
    init_config_channel, load_config,
};
use wallpaper::ordering::{RandomOrdering, SequentialOrdering, get_next_image, is_video_file, WallpaperOrdering};

fn build_ui(application: &Application) {
    let mut config = load_config();
    let mut config_rx = init_config_channel();

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

    let video1 = gtk::Video::new();
    let video2 = gtk::Video::new();

    stack.add_child(&video1);
    stack.add_child(&video2);

    let current_slot: std::sync::Mutex<u32> = std::sync::Mutex::new(1);

    let sequential_ordering =
        Arc::new(SequentialOrdering::new()) as Arc<dyn WallpaperOrdering>;
    let random_ordering = Arc::new(RandomOrdering) as Arc<dyn WallpaperOrdering>;

    let stack_clone = stack.clone();
    let set_stack_transition = move |config_transition_type: StackTransitionType| {
        stack_clone.set_transition_type(config_transition_type);
    };

    let window_for_layer = window.clone();
    let set_layer = move |layer: gtk4_layer_shell::Layer| {
        window_for_layer.set_layer(layer);
    };

    let stack_for_image = stack.clone();
    let video1_clone = video1.clone();
    let video2_clone = video2.clone();
    let allow_animated = config.allow_animated;

    let set_wallpaper = move |path: &str| {
        let path_buf = std::path::PathBuf::from(path);
        let is_video = is_video_file(&path_buf, allow_animated);
        
        let mut current = current_slot.lock().unwrap();
        if *current == 1 {
            if is_video {
                picture1.set_filename(None::<&std::path::Path>);
                picture2.set_filename(None::<&std::path::Path>);
                video1_clone.set_filename(None::<&std::path::Path>);
                video2_clone.set_loop(true);
                video2_clone.set_autoplay(true);
                video2_clone.set_filename(Some(path));
                stack_for_image.set_visible_child(&video2_clone);
            } else {
                video1_clone.set_filename(None::<&std::path::Path>);
                video2_clone.set_filename(None::<&std::path::Path>);
                picture1.set_filename(None::<&std::path::Path>);
                picture2.set_filename(Some(path));
                stack_for_image.set_visible_child(&picture2);
            }
            *current = 2;
        } else {
            if is_video {
                picture1.set_filename(None::<&std::path::Path>);
                picture2.set_filename(None::<&std::path::Path>);
                video2_clone.set_filename(None::<&std::path::Path>);
                video1_clone.set_loop(true);
                video1_clone.set_autoplay(true);
                video1_clone.set_filename(Some(path));
                stack_for_image.set_visible_child(&video1_clone);
            } else {
                video1_clone.set_filename(None::<&std::path::Path>);
                video2_clone.set_filename(None::<&std::path::Path>);
                picture2.set_filename(None::<&std::path::Path>);
                picture1.set_filename(Some(path));
                stack_for_image.set_visible_child(&picture1);
            }
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
        if let Ok(updated_config) = config_rx.try_recv() {
            config = updated_config;
        }

        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        set_stack_transition(config.transition_type.into());
        set_layer(config.layer.into());

        let last_change = last_slideshow_change.load(Ordering::Relaxed);
        if current_time - last_change >= config.refresh_interval || first_load {
            first_load = false;
            last_slideshow_change.store(current_time, Ordering::Relaxed);

            let strategy = match config.ordering {
                ConfigOrdering::Sequential => sequential_ordering.clone(),
                _ => random_ordering.clone(),
            };

            if let Some(path) = get_next_image(config.wallpaper_path.as_ref().unwrap().as_path(), strategy.as_ref(), config.allow_animated) {
                set_wallpaper(&path);
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
