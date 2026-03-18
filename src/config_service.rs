use log::debug;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use toml_edit::{DocumentMut, Item, Table};

use crate::config::{Config, Layer, Ordering, TransitionType};

static CONFIG_TX: std::sync::OnceLock<mpsc::Sender<Config>> = std::sync::OnceLock::new();
static CONFIG: std::sync::OnceLock<Arc<Mutex<Option<Config>>>> = std::sync::OnceLock::new();

pub fn get_config() -> Arc<Mutex<Option<Config>>> {
    CONFIG.get_or_init(|| Arc::new(Mutex::new(None))).clone()
}

pub fn init_config_channel() -> mpsc::Receiver<Config> {
    let (tx, rx) = mpsc::channel(100);
    let _ = CONFIG_TX.set(tx);
    rx
}

fn send_config_update(config: &Config) {
    if let Some(tx) = CONFIG_TX.get() {
        let _ = tx.try_send(config.clone());
    }
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

    let defaults = doc.as_ref().and_then(|d| d.get("defaults")).and_then(|i| i.as_table());

    let wallpaper_path = defaults
        .and_then(|table| table.get("wallpaper_path"))
        .and_then(|value| value.as_str())
        .filter(|s| !s.is_empty())
        .map(PathBuf::from);

    let refresh_interval = defaults
        .and_then(|table| table.get("refresh_interval"))
        .and_then(|value| value.as_integer())
        .unwrap_or(30) as u64;

    let ordering = defaults
        .and_then(|table| table.get("ordering"))
        .and_then(|value| value.as_str())
        .map(Ordering::from)
        .unwrap_or_default();

    let transition_type = defaults
        .and_then(|table| table.get("transition_type"))
        .and_then(|value| value.as_str())
        .map(TransitionType::from)
        .unwrap_or_default();

    let layer = defaults
        .and_then(|table| table.get("layer"))
        .and_then(|value| value.as_str())
        .map(Layer::from)
        .unwrap_or_default();
    
    let config = Config {
        wallpaper_path,
        refresh_interval,
        ordering,
        transition_type,
        layer,
    }; 

    let config_store = get_config();
    let mut guard = config_store.blocking_lock();
    *guard = Some(config.clone());

    config
}

fn write_config() {
    let config_path = match get_config_path() {
        Some(p) => p,
        None => return,
    };

    let config = {
        let store = get_config();
        let guard = store.blocking_lock();
        guard.clone()
    };

    let mut doc = DocumentMut::new();
    let defaults = doc.entry("defaults").or_insert(Item::Table(Table::new()));
    let defaults_table = defaults.as_table_mut().unwrap();

    if let Some(config) = config {
        defaults_table.insert("wallpaper_path", config.wallpaper_path.as_ref().unwrap().to_string_lossy().as_ref().into());
        defaults_table.insert("refresh_interval", (config.refresh_interval as i64).into());
        defaults_table.insert("ordering", config.ordering.as_ref().into());
        defaults_table.insert("transition_type", config.transition_type.as_ref().into());
        defaults_table.insert("layer", config.layer.as_ref().into());
        send_config_update(&config);
        drop(config);
    }

    if let Some(parent) = config_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let content = doc.to_string();
    let _ = std::fs::write(&config_path, content);

}

fn update_config_and_write<F>(update: F)
where
    F: FnOnce(&mut Config),
{
    let config_store = get_config();
    let mut guard = config_store.blocking_lock();
    update(guard.as_mut().unwrap());
    drop(guard);
    write_config();
}

pub async fn set_wallpaper(path: String) -> String {
    let path_buf = PathBuf::from(&path);

    update_config_and_write(|c| c.wallpaper_path = Some(path_buf));

    debug!("Wallpaper directory set to {}", path);
    format!("Wallpaper directory set to {}", path)
}

pub async fn set_refresh_interval(interval: u32) -> String {
    update_config_and_write(|c| c.refresh_interval = interval as u64);

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

    update_config_and_write(|c| c.ordering = ordering.as_str().into());

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

    update_config_and_write(|c| c.transition_type = TransitionType::from(lower.as_str()));

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

    update_config_and_write(|c| c.layer = Layer::from(lower.as_str()));

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
