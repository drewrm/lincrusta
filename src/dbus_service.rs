use zbus::{Result, connection::Builder, interface};

use crate::config_service::{
    set_allow_animated, set_layer, set_ordering, set_refresh_interval, set_transition_type, set_wallpaper,
};

struct WallpaperService;

#[interface(name = "org.drewrm.wallpaperd")]
impl WallpaperService {
    async fn set_wallpaper(&self, path: String) -> String {
        set_wallpaper(path).await
    }

    async fn set_refresh_interval(&self, interval: u32) -> String {
        set_refresh_interval(interval).await
    }

    async fn set_ordering(&self, ordering: String) -> String {
        set_ordering(ordering).await
    }

    async fn set_transition_type(&self, transition_type: String) -> String {
        set_transition_type(transition_type).await
    }

    async fn set_layer(&self, layer: String) -> String {
        set_layer(layer).await
    }

    async fn set_allow_animated(&self, allow: bool) -> String {
        set_allow_animated(allow).await
    }
}

pub async fn run_dbus_server() -> Result<()> {
    let _connection = Builder::session()?
        .name("org.drewrm.wallpaperd")?
        .serve_at("/org/drewrm/wallpaperd", WallpaperService)?
        .build()
        .await?;

    std::future::pending::<()>().await;

    Ok(())
}
