# Wallpaper

A GTK4 desktop wallpaper manager for Linux with D-Bus integration and slideshow support.

![Screenshot of the CLI](https://github.com/drewrm/lincrusta/blob/main/screenshot.png?raw=true)

This has been built/tested on Fedora 43 with Niri Window Manager.

## Commands

### wallpaperd

The wallpaper daemon runs as a GTK4 application using [gtk4-layer-shell](https://github.com/wmww/gtk4-layer-shell) to display wallpapers on the background layer. It provides a D-Bus interface for runtime configuration.

**Features:**
- Slideshow mode with configurable directory
- Configurable refresh interval (default: 30 seconds)
- Two ordering modes: `random` or `sequential`
- 19 transition effects between wallpaper changes
- Configurable layer shell layer

**Configuration:**

The daemon reads default settings from `~/.config/org.drewrm.wallpaperd/wallpaperd.toml`:
```toml
[defaults]
wallpaper_path = ""
refresh_interval = 30
ordering = "sequential"
transition_type = "crossfade"
layer = "background"
```

**Layer options:**
- `background` - Background layer (behind normal windows)
- `bottom` - Bottom layer
- `top` - Top layer (above normal windows)
- `overlay` - Overlay layer (always on top)

**Transition types available:**
- `none` - Instant switch
- `crossfade` - Fade between images
- `slide_right`, `slide_left`, `slide_up`, `slide_down` - Slide animations
- `slide_left_right`, `slide_up_down` - Bidirectional slides
- `over_up`, `over_down`, `over_left`, `over_right` - Overlay animations
- `under_up`, `under_down`, `under_left`, `under_right` - Underlay animations
- `rotate_left`, `rotate_right`, `rotate_left_right` - Rotation animations

### wallpaper-cli

A command-line tool to control the wallpaper daemon via D-Bus.

**Usage:**
```bash
wallpaper-cli <command> [options]
```

**Commands:**

| Command | Description |
|---------|-------------|
| `path <path>` | Set wallpaper path (image file or directory for slideshow) |
| `refresh-interval <seconds>` | Set interval between wallpaper changes (minimum 1 second) |
| `ordering <mode>` | Set slideshow ordering: `random` or `sequential` |
| `transition-type <effect>` | Set transition effect (see list below) |
| `layer <layer>` | Set layer shell layer: `background`, `bottom`, `top`, or `overlay` |

**Examples:**
```bash
# Set a static wallpaper
wallpaper-cli path /path/to/image.jpg

# Set a slideshow directory with 30-second interval
wallpaper-cli path /path/to/wallpapers
wallpaper-cli refresh-interval 30
wallpaper-cli ordering random
wallpaper-cli transition-type slide_left
wallpaper-cli layer overlay
```

## Building

```bash
cargo build --release
```

## Installation

Install the binaries to `~/cargo/bin`:

```bash
cargo install --path .
```

To run as a systemd service add the following unit file to `~/.config/systemd/user/wallpaperd.service` 

```
[Unit]
Description=Wallpaper Daemon for Wayland
PartOf=graphical-session.target
Requires=graphical-session.target
After=graphical-session.target
ConditionEnvironment=WAYLAND_DISPLAY

[Service]
Type=simple
ExecStart=%h/.cargo/bin/wallpaperd
Slice=session.slice
Restart=on-failure

[Install]
WantedBy=greaphical-session.target
```

Enable the service to start on login:

```bash
systemctl --user enable wallpaperd.service
systemctl --user start wallpaperd.service
```

*Note* - Only a single instance of the daemon can run at any one time.
