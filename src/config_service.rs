use log::debug;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use toml_edit::{DocumentMut, Item, Table};

use crate::config::{Config, Layer, Ordering, TransitionType};

static WALLPAPER_PATH: std::sync::OnceLock<Arc<Mutex<Option<String>>>> = std::sync::OnceLock::new();
static WALLPAPER_DIR: std::sync::OnceLock<Arc<Mutex<Option<PathBuf>>>> = std::sync::OnceLock::new();
static REFRESH_INTERVAL: std::sync::OnceLock<Arc<Mutex<Option<u64>>>> = std::sync::OnceLock::new();
static ORDERING: std::sync::OnceLock<Arc<Mutex<Option<Ordering>>>> = std::sync::OnceLock::new();
static TRANSITION_TYPE: std::sync::OnceLock<Arc<Mutex<Option<TransitionType>>>> =
    std::sync::OnceLock::new();
static LAYER: std::sync::OnceLock<Arc<Mutex<Option<Layer>>>> = std::sync::OnceLock::new();
static WALLPAPER_PATH_CONFIG: std::sync::OnceLock<Arc<Mutex<Option<String>>>> =
    std::sync::OnceLock::new();

pub fn get_wallpaper_path() -> Arc<Mutex<Option<String>>> {
    WALLPAPER_PATH
        .get_or_init(|| Arc::new(Mutex::new(None)))
        .clone()
}

pub fn get_wallpaper_dir() -> Arc<Mutex<Option<PathBuf>>> {
    WALLPAPER_DIR
        .get_or_init(|| Arc::new(Mutex::new(None)))
        .clone()
}

pub fn get_refresh_interval() -> Arc<Mutex<Option<u64>>> {
    REFRESH_INTERVAL
        .get_or_init(|| Arc::new(Mutex::new(None)))
        .clone()
}

pub fn get_ordering() -> Arc<Mutex<Option<Ordering>>> {
    ORDERING.get_or_init(|| Arc::new(Mutex::new(None))).clone()
}

pub fn get_transition_type() -> Arc<Mutex<Option<TransitionType>>> {
    TRANSITION_TYPE
        .get_or_init(|| Arc::new(Mutex::new(None)))
        .clone()
}

pub fn get_layer() -> Arc<Mutex<Option<Layer>>> {
    LAYER.get_or_init(|| Arc::new(Mutex::new(None))).clone()
}

fn get_config_path() -> Option<PathBuf> {
    let config_dir = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|_| std::env::var("HOME").map(|h| PathBuf::from(h).join(".config")))
        .ok()?;

    Some(
        config_dir
            .join("org.drewrm.wallpaperd")
            .join("wallpaperd.toml"),
    )
}

pub fn load_config() -> Config {
    let config_path = get_config_path();

    let doc: Option<DocumentMut> = config_path
        .as_ref()
        .filter(|p| p.exists())
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|c| c.parse().ok());

    let wallpaper_path = doc
        .as_ref()
        .and_then(|d| d.get("defaults"))
        .and_then(|i| i.as_table())
        .and_then(|t| t.get("wallpaper_path"))
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(PathBuf::from);

    let refresh_interval = doc
        .as_ref()
        .and_then(|d| d.get("defaults"))
        .and_then(|i| i.as_table())
        .and_then(|t| t.get("refresh_interval"))
        .and_then(|v| v.as_integer())
        .unwrap_or(30) as u64;

    let ordering = doc
        .as_ref()
        .and_then(|d| d.get("defaults"))
        .and_then(|i| i.as_table())
        .and_then(|t| t.get("ordering"))
        .and_then(|v| v.as_str())
        .map(Ordering::from)
        .unwrap_or_default();

    let transition_type = doc
        .as_ref()
        .and_then(|d| d.get("defaults"))
        .and_then(|i| i.as_table())
        .and_then(|t| t.get("transition_type"))
        .and_then(|v| v.as_str())
        .map(TransitionType::from)
        .unwrap_or_default();

    let layer = doc
        .as_ref()
        .and_then(|d| d.get("defaults"))
        .and_then(|i| i.as_table())
        .and_then(|t| t.get("layer"))
        .and_then(|v| v.as_str())
        .map(Layer::from)
        .unwrap_or_default();

    if let Some(wallpaper_path) = &wallpaper_path && wallpaper_path.is_dir() {
        let dir_store = get_wallpaper_dir();
        let mut guard = dir_store.blocking_lock();
        *guard = Some(wallpaper_path.clone());
    }

    {
        let interval_store = get_refresh_interval();
        let mut guard = interval_store.blocking_lock();
        *guard = Some(refresh_interval);
    }

    {
        let ordering_store = get_ordering();
        let mut guard = ordering_store.blocking_lock();
        *guard = Some(ordering);
    }

    {
        let transition_store = get_transition_type();
        let mut guard = transition_store.blocking_lock();
        *guard = Some(transition_type);
    }

    {
        let layer_store = get_layer();
        let mut guard = layer_store.blocking_lock();
        *guard = Some(layer);
    }

    {
        let config_path_store = WALLPAPER_PATH_CONFIG
            .get_or_init(|| Arc::new(Mutex::new(None)))
            .clone();
        let mut guard = config_path_store.blocking_lock();
        *guard = wallpaper_path
            .clone()
            .map(|p| p.to_string_lossy().to_string());
    }

    Config {
        wallpaper_path,
        refresh_interval,
        ordering,
        transition_type,
        layer,
    }
}

fn write_config() {
    let config_path = match get_config_path() {
        Some(p) => p,
        None => return,
    };

    let wallpaper_path = {
        let store = WALLPAPER_PATH_CONFIG.get_or_init(|| Arc::new(Mutex::new(None)));
        let guard = store.blocking_lock();
        guard.clone()
    };

    let refresh_interval = {
        let store = get_refresh_interval();
        let guard = store.blocking_lock();
        *guard
    };

    let ordering = {
        let store = get_ordering();
        let guard = store.blocking_lock();
        guard.clone()
    };

    let transition_type = {
        let store = get_transition_type();
        let guard = store.blocking_lock();
        guard.clone()
    };

    let layer = {
        let store = get_layer();
        let guard = store.blocking_lock();
        guard.clone()
    };

    let mut doc = DocumentMut::new();

    let defaults = doc.entry("defaults").or_insert(Item::Table(Table::new()));
    let defaults_table = defaults.as_table_mut().unwrap();

    if let Some(path) = wallpaper_path {
        defaults_table.insert("wallpaper_path", path.into());
    }

    if let Some(interval) = refresh_interval {
        defaults_table.insert("refresh_interval", (interval as i64).into());
    }

    if let Some(order) = ordering {
        defaults_table.insert("ordering", order.as_ref().into());
    }

    if let Some(transition) = transition_type {
        defaults_table.insert("transition_type", transition.as_ref().into());
    }

    if let Some(layer) = layer {
        defaults_table.insert("layer", layer.as_ref().into());
    }

    if let Some(parent) = config_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let content = doc.to_string();
    let _ = std::fs::write(&config_path, content);
}

pub async fn set_wallpaper(path: String) -> String {
    let path_buf = PathBuf::from(&path);

    let wallpaper_dir = get_wallpaper_dir();
    let mut guard = wallpaper_dir.lock().await;
    *guard = Some(path_buf.clone());

    let wallpaper_path = get_wallpaper_path();
    let mut path_guard = wallpaper_path.lock().await;
    *path_guard = None;

    {
        let store = WALLPAPER_PATH_CONFIG.get_or_init(|| Arc::new(Mutex::new(None)));
        let mut guard = store.lock().await;
        *guard = Some(path.clone());
    }

    write_config();

    debug!("Wallpaper directory set to {}", path);
    format!("Wallpaper directory set to {}", path)
}

pub async fn set_refresh_interval(interval: u32) -> String {
    let refresh_interval = get_refresh_interval();
    let mut guard = refresh_interval.lock().await;
    *guard = Some(interval as u64);
    drop(guard);

    write_config();

    debug!("Refresh interval set to {} seconds", interval);
    format!("Refresh interval set to {} seconds", interval)
}

pub async fn set_ordering(ordering: String) -> String {
    let valid_orderings = ["random", "sequential"];
    if !valid_orderings.contains(&ordering.to_lowercase().as_str()) {
        return format!(
            "Invalid ordering: {}. Valid options: random, sequential",
            ordering
        );
    }

    let ordering_store = get_ordering();
    let mut guard = ordering_store.lock().await;
    *guard = Some(ordering.as_str().into());
    drop(guard);

    write_config();

    debug!("Ordering set to {}", ordering);
    format!("Ordering set to {}", ordering)
}

pub async fn set_transition_type(transition_type: String) -> String {
    let lower = transition_type.to_lowercase();
    if !TransitionType::is_valid(&lower) {
        return format!(
            "Invalid transition_type: {}. Valid options: {:?}",
            transition_type,
            TransitionType::valid_options()
        );
    }

    let transition_store = get_transition_type();
    let mut guard = transition_store.lock().await;
    *guard = Some(TransitionType::from(lower.as_str()));
    drop(guard);

    write_config();

    debug!("Transition type set to {}", transition_type);
    format!("Transition type set to {}", transition_type)
}

pub async fn set_layer(layer: String) -> String {
    let lower = layer.to_lowercase();
    if !Layer::is_valid(&lower) {
        return format!(
            "Invalid layer: {}. Valid options: {:?}",
            layer,
            Layer::valid_options()
        );
    }

    let layer_store = get_layer();
    let mut guard = layer_store.lock().await;
    *guard = Some(Layer::from(lower.as_str()));
    drop(guard);

    write_config();

    debug!("Layer set to {}", layer);
    format!("Layer set to {}", layer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_config_with_defaults() {
        // Set up test config
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("org.drewrm.wallpaperd");
        std::fs::create_dir_all(&config_path).unwrap();

        let config_file = config_path.join("wallpaperd.toml");
        std::fs::write(
            &config_file,
            r#"
[defaults]
wallpaper_path = "/test/path"
refresh_interval = 60
ordering = "random"
"#,
        )
        .unwrap();

        let parent = temp_dir.path().to_path_buf();
        unsafe {
            std::env::set_var("XDG_CONFIG_HOME", &parent);
        }

        let config = load_config();

        assert_eq!(config.refresh_interval, 60);
        assert_eq!(config.ordering, Ordering::Random);

        unsafe {
            std::env::remove_var("XDG_CONFIG_HOME");
        }
    }

    #[test]
    fn test_load_config_with_empty_values() {
        // Set up test config with empty values
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("org.drewrm.wallpaperd");
        std::fs::create_dir_all(&config_path).unwrap();

        let config_file = config_path.join("wallpaperd.toml");
        std::fs::write(&config_file, "").unwrap();

        let parent = temp_dir.path().to_path_buf();
        unsafe {
            std::env::set_var("XDG_CONFIG_HOME", &parent);
        }

        let config = load_config();

        // Should have defaults
        assert_eq!(config.refresh_interval, 30);
        assert_eq!(config.ordering, Ordering::Sequential);

        unsafe {
            std::env::remove_var("XDG_CONFIG_HOME");
        }
    }
}
