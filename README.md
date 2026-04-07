This utility forwards Nexus Mods `nxm://` links to Mod Organizer 2 within a Wine prefix. It spawns a file dialog so you can select your Wine prefix, persists the location for future use, and forwards the download request to MO2.

### Usage

*   **Build the executable:**
    ```bash
    cargo build --release
    ```

*   **Run from the terminal:**
    ```bash
    ./target/release/nxmhandler --nxm-url "nxm://..."
    ```

*   **Arguments:**
    *   `-n, --nxm-url <NXM_URL>`: The NXM URL to handle.
    *   `-w, --wineprefix <WINEPREFIX>`: Path to the Wine prefix. Prompts via file dialog if not set.
    *   `-a, --winearch <WINEARCH>`: Wine prefix architecture [default: win64] [possible values: win64, win32]
    *   `-m <MO2_PATH>`: MO2 path relative to the prefix's `drive_c` directory [default: `Modding/MO2`]
    *   `-c, --create-desktop-file`: Create a .desktop entry to register the NXM protocol *(mutually exclusive with other arguments)*
    *   `-h, --help`: Print help

### Browser Integration (Nexus Mods):
    
* Run `cargo run --release -- --create-desktop-file`

> [!NOTE]
> Next time you download a mod, the browser will prompt you to choose an application tomanage nxm:// links. Choose `NXM Handler`. Since then the browser will use the created desktop entry, which automatically forwards the NXM URL to nxmhandler

<details>

<summary>If you want to do it manually</summary>

Create a `.desktop` file (e.g. `~/.local/share/applications/nxm-handler.desktop`) with the following content. Update the `Exec` path to point to your compiled binary.
After saving the file, run `update-desktop-database ~/.local/share/applications` to register the handler.

```ini
[Desktop Entry]
Comment[en_US]=
Comment=
Exec=path-to-nxmhandler --nxm-url %u
GenericName[en_US]=
GenericName=
Icon=1204_ModOrganizer.0
MimeType=x-scheme-handler/nxm;
Name[en_US]=NXM Handler
Name=NXM Handler
Path=
StartupNotify=true
Terminal=false
Type=Application
X-KDE-SubstituteUID=false
X-KDE-Username=
```
</details>

> [!TIP]
> To verify the handler is registered, run:
> ```bash
> xdg-mime query default x-scheme-handler/nxm
> ```
> It should output `~/.local/share/applications/nxmhandler.desktop`