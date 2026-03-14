# Wallpaper

A GTK4 desktop wallpaper manager for Linux with D-Bus integration and slideshow support.

## Commands

### wallpaperd

The wallpaper daemon runs as a GTK4 application using [gtk4-layer-shell](https://github.com/wmww/gtk4-layer-shell) to display wallpapers on the background layer. It provides a D-Bus interface for runtime configuration.

**Features:**
- Single image wallpaper display
- Slideshow mode with configurable directory
- Configurable refresh interval (default: 30 seconds)
- Two ordering modes: `random` or `sequential`
- 19 transition effects between wallpaper changes
- Configurable layer shell layer

**Configuration:**

The daemon reads default settings from `config/default.toml` or `~/.config/org.drewrm.wallpaperd/wallpaperd.toml`:
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

## Running

Start the daemon first, then use the CLI to control it:

```bash
# Start the wallpaper daemon (in background or on startup)
./target/release/wallpaperd

# Control via CLI
./target/release/wallpaper-cli path ~/Pictures/wallpapers
```
