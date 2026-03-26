# Quarkpad

Quarkpad is a modern, lightweight game launcher for Linux designed to simplify managing and launching games using **Proton** and **umu-launcher**. Built with Rust, GTK4, and Libadwaita, it provides a clean, native experience for Linux gamers who want a straightforward way to organize their library.

## Features

- **Proton Integration:** Easily launch Windows games using Proton and `umu-launcher`.
- **Game Management:** Organize your games with custom names, cover art, and wineprefixes.
- **Gamescope Support:** Built-in configuration for Gamescope to optimize your gaming experience.
- **Playtime Tracking:** Keep track of how much time you've spent in each game and when you last played.
- **Modern UI:** A beautiful, responsive interface following GNOME HIG, featuring both grid and list views.
- **Customizable:** Configure global Proton and umu-launcher paths to suit your setup.

## Installation

### Flatpak (Recommended)

Quarkpad is available via a custom Flatpak repository.

1. **Add the repository:**
   ```bash
   flatpak remote-add --if-not-exists galib-flatpaks \
   https://galib-flatpaks.netlify.app/repo/galib-flatpaks.flatpakrepo
   ```

2. **Install required runtimes (for 32-bit compatibility and GPU acceleration):**
   ```bash
   flatpak install org.freedesktop.Platform.Compat.i386//25.08
   flatpak install org.freedesktop.Platform.GL32.default//25.08
   flatpak install org.freedesktop.Platform.VAAPI.Intel.i386//25.08
   ```

3. **Install Quarkpad:**
   ```bash
   flatpak install org.galib.quarkpad
   ```

### Building from Source

1. Clone the repository:
   ```bash
   git clone https://github.com/galib45/quarkpad.git
   cd quarkpad
   ```

2. Build the project:
   ```bash
   cargo build --release
   ```

3. Run Quarkpad:
   ```bash
   ./target/release/quarkpad
   ```

## Usage

1. **Configure Paths:** Upon first launch, head to the settings page to set your Proton and `umu-launcher` paths.
2. **Add Games:** Click the "+" button to add a new game. You'll need to provide the executable path, a wineprefix directory, and optional cover art.
3. **Launch:** Simply click on a game in your library and hit the play button!

## Development

Quarkpad uses `blueprint` for its UI files. If you modify any `.blp` files in `resources/ui/`, ensure you have the `blueprint-compiler` installed to see the changes reflected in the generated `.ui` files during the build process.

## License

This project is licensed under the **Unlicense** - see the [LICENSE](LICENSE) file for details.

## Author

**Asadul Al Galib** - [GitHub](https://github.com/galib45)
