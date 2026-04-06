use anyhow::{Context, Result};
use clap::Parser;
use rfd::{FileDialog, MessageDialog};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Parser)]
#[command(
    version,
    about = "Handles Nexus Mods (nxm) requests by sending them to MO2 in a wine prefix"
)]
struct Cli {
    /// NXM URL
    #[arg(short, long, required = true)]
    nxm_url: String,

    /// Path to wineprefix (prompts if not provided)
    #[arg(short, long)]
    wineprefix: Option<PathBuf>,

    /// MO2 path relative to wineprefix's drive_c
    #[arg(short, long, default_value = "Modding/MO2")]
    mo2_path: PathBuf,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let wineprefix = match cli.wineprefix {
        Some(p) => p,
        None => select_wineprefix()?,
    };

    let mo2_dir = wineprefix.join("drive_c").join(&cli.mo2_path);
    let nxmhandler = mo2_dir.join("nxmhandler.exe");

    if !mo2_dir.is_dir() {
        anyhow::bail!("MO2 directory not found at {}", mo2_dir.display());
    }

    if !nxmhandler.is_file() {
        anyhow::bail!("nxmhandler.exe not found at {}", nxmhandler.display());
    }

    spawn_mo2(&wineprefix, &mo2_dir, &cli.nxm_url)?;

    Ok(())
}

fn is_wineprefix(path: &Path) -> bool {
    path.join("drive_c").is_dir() && path.join("dosdevices").is_dir()
}

fn spawn_mo2(wineprefix: &Path, mo2_dir: &Path, nxm_url: &str) -> Result<()> {
    let cmd = Command::new("wine")
        .env("WINEARCH", "win64")
        .env("WINEPREFIX", wineprefix)
        .current_dir(mo2_dir)
        .arg("nxmhandler.exe")
        .arg(nxm_url)
        .spawn()
        .context("Failed to launch wine")?;

    let output = cmd.wait_with_output()?;
    println!("wine output: {:?}", output);

    Ok(())
}

fn select_wineprefix() -> Result<PathBuf> {
    let initial_dir = match load_last_path() {
        Some(p) => p,
        None => {
            let home = std::env::var("HOME").context("HOME environment variable not set")?;
            PathBuf::from(format!("{}/.wine", home))
        }
    };

    loop {
        let folder = FileDialog::new().set_directory(&initial_dir).pick_folder();

        match folder {
            Some(path) => {
                if is_wineprefix(&path) {
                    if let Err(e) = save_last_path(&path) {
                        eprintln!("Failed to save last used path: {}", e);
                    }
                    return Ok(path);
                }

                MessageDialog::new()
                    .set_title("Invalid Wine Prefix")
                    .set_description(
                        "The selected directory does not appear to be a valid Wine prefix",
                    )
                    .set_level(rfd::MessageLevel::Warning)
                    .show();
            }
            None => anyhow::bail!("No wineprefix selected"),
        }
    }
}

fn get_config_path() -> Option<PathBuf> {
    std::env::var("HOME")
        .ok()
        .map(|h| PathBuf::from(h).join("$HOME/.config/nxm-handler/last_prefix"))
}

fn save_last_path(path: &Path) -> Result<()> {
    if let Some(config_path) = get_config_path() {
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).context("Failed to create config directory")?;
        }
        fs::write(&config_path, path.to_string_lossy().as_bytes())
            .context("Failed to write config file")?;
    }
    Ok(())
}

fn load_last_path() -> Option<PathBuf> {
    let config_path = get_config_path()?;
    if config_path.exists() {
        let content = fs::read_to_string(config_path).ok()?;
        let path = PathBuf::from(content.trim());
        if path.is_dir() {
            return Some(path);
        }
    }
    None
}
