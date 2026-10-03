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
- Support for animated/video wallpapers (mp4, webm, mov, avi, mkv)

**Configuration:**

The daemon reads default settings from `~/.config/org.drewrm.wallpaperd/wallpaperd.toml`:
```toml
[defaults]
wallpaper_path = ""
refresh_interval = 30
ordering = "sequential"
transition_type = "crossfade"
layer = "background"
allow_animated = false
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
| `allow-animated <bool>` | Enable or disable animated/video wallpapers: `true` or `false` |

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

# Enable animated/video wallpapers
wallpaper-cli allow-animated true
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

## Releases

This project uses [release-please](https://github.com/google-github-actions/release-please-action) for automated versioning and releases.

### How it works

1. **Conventional commits** on `main` trigger a release PR
2. **Merge the release PR** → Creates tag + updates `Cargo.toml`/`Cargo.lock`
3. **Tag push** → Triggers build workflow → Creates `.deb` + GitHub Release

### Required GitHub Setting

For release-please to create pull requests, the repository must allow GitHub Actions to create and approve PRs:

**Settings → Actions → General → Workflow permissions → ✅ "Allow GitHub Actions to create and approve pull requests"**

Without this setting, release-please will fail with: `GitHub Actions is not permitted to create or approve pull requests.`

### Commit format

Use [Conventional Commits](https://www.conventionalcommits.org/):

```bash
feat: add new feature
fix: bug fix
chore: maintenance task
docs: documentation changes
refactor: code refactoring
test: adding tests
```

| Type | Version bump |
|------|--------------|
| `feat:` | **minor** (0.1.x → 0.2.0) |
| `fix:`, `chore:`, `docs:`, `refactor:`, `test:` | **patch** (0.1.x → 0.1.x+1) |
| `BREAKING CHANGE:` in footer | **major** (0.x.x → 1.0.0) |

### Example workflow

```bash
# Make changes
git add .
git commit -m "feat: add new transition effect"
git push origin main

# → release-please creates/updates a release PR
# → Merge the PR when ready
# → Tag + GitHub Release created automatically
```

### Manual release (optional)

For manual control, tag directly:

```bash
cargo release patch --execute --no-confirm --no-publish --no-verify
git push origin main --tags
```

### Alternative: Auto-release on merge (no PR required)

If you prefer not to enable the GitHub Actions PR creation setting, add this workflow instead of release-please:

**`.github/workflows/auto-release.yml`**
```yaml
on:
  push:
    branches: [main]

permissions:
  contents: write

jobs:
  auto-release:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with: { token: ${{ secrets.GITHUB_TOKEN }} }
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo install cargo-release
      - run: cargo release patch --execute --no-confirm --no-publish --no-verify
      - run: git push origin main --tags
```

This bumps the patch version on every merge to main and pushes the tag directly (no PR).

### Installing releases

Download the `.deb` from [GitHub Releases](https://github.com/drewrm/wallpaper/releases) and install:

```bash
sudo dpkg -i lincrusta_0.1.4-1_amd64.deb
```

Or build from source:

```bash
cargo install --path .
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make changes with conventional commits
4. Run pre-commit checks: `prek run`
5. Open a pull request

Pre-commit hooks run:
- `cargo fmt`, `cargo clippy`, `cargo test`
- `actionlint` for GitHub Actions
- Standard checks (trailing whitespace, YAML/TOML/JSON syntax, etc.)
