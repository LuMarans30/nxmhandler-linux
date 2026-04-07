use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use directories::{BaseDirs, ProjectDirs};
use rfd::{FileDialog, MessageDialog};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(ValueEnum, Debug, Clone, PartialEq, Default)]
#[clap(rename_all = "kebab_case")]
enum WineArch {
    #[default]
    Win64,
    Win32,
}

impl WineArch {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Win64 => "win64",
            Self::Win32 => "win32",
        }
    }
}

impl std::fmt::Display for WineArch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

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

    /// Wine prefix architecture
    #[arg(short = 'a', long, default_value_t = WineArch::Win64)]
    winearch: WineArch,

    /// MO2 path relative to wineprefix's drive_c
    #[arg(short, long, default_value = "Modding/MO2")]
    mo2_path: PathBuf,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let wineprefix = cli.wineprefix.map_or_else(select_wineprefix, Ok)?;

    if let Err(e) = save_last_path(&wineprefix) {
        eprintln!("Failed to save last used path: {}", e);
    }

    let mo2_dir = wineprefix.join("drive_c").join(&cli.mo2_path);
    let nxmhandler = mo2_dir.join("nxmhandler.exe");

    anyhow::ensure!(
        mo2_dir.is_dir(),
        "MO2 directory not found at {}",
        mo2_dir.display()
    );
    anyhow::ensure!(
        nxmhandler.is_file(),
        "nxmhandler.exe not found at {}",
        nxmhandler.display()
    );

    spawn_mo2(
        &wineprefix,
        &mo2_dir,
        &nxmhandler,
        &cli.nxm_url,
        cli.winearch,
    )?;

    Ok(())
}

fn is_wineprefix(path: &Path) -> bool {
    path.join("drive_c").is_dir() && path.join("dosdevices").is_dir()
}

fn spawn_mo2(
    wineprefix: &Path,
    mo2_dir: &Path,
    nxmhandler: &Path,
    nxm_url: &str,
    winearch: WineArch,
) -> Result<()> {
    let status = Command::new("wine")
        .env("WINEARCH", winearch.to_string())
        .env("WINEPREFIX", wineprefix)
        .current_dir(mo2_dir)
        .arg(nxmhandler)
        .arg(nxm_url)
        .status()
        .context("Failed to launch wine process")?;

    anyhow::ensure!(
        status.success(),
        "Wine exited with an error status: {}",
        status
    );

    Ok(())
}

fn select_wineprefix() -> Result<PathBuf> {
    let initial_dir = load_last_path().unwrap_or_else(|_| {
        BaseDirs::new()
            .map(|base| base.home_dir().join(".wine"))
            .expect("Could not determine user home directory")
    });

    loop {
        if let Some(path) = FileDialog::new().set_directory(&initial_dir).pick_folder() {
            if is_wineprefix(&path) {
                return Ok(path);
            }

            MessageDialog::new()
                .set_title("Invalid Wine Prefix")
                .set_description("The selected directory does not contain a valid 'drive_c' and 'dosdevices' structure.")
                .set_level(rfd::MessageLevel::Warning)
                .show();
        } else {
            anyhow::bail!("Wineprefix selection was cancelled by the user.");
        }
    }
}

// XDG compliant config

fn get_project_dirs() -> Result<ProjectDirs> {
    ProjectDirs::from("", "", "nxm-handler")
        .context("Could not determine valid configuration directories for your OS")
}

fn save_last_path(path: &Path) -> Result<()> {
    let proj_dirs = get_project_dirs()?;
    let config_dir = proj_dirs.config_dir();

    fs::create_dir_all(config_dir).context("Failed to create configuration directory")?;

    let config_file = config_dir.join("last_prefix");
    fs::write(&config_file, path.to_string_lossy().as_bytes())
        .context("Failed to write prefix path to configuration file")?;

    Ok(())
}

fn load_last_path() -> Result<PathBuf> {
    let config_file = get_project_dirs()?.config_dir().join("last_prefix");

    let content = fs::read_to_string(config_file)
        .context("No previous configuration found or file is unreadable")?;

    let path = PathBuf::from(content.trim());
    anyhow::ensure!(
        path.is_dir(),
        "The previously saved prefix is no longer a valid directory"
    );

    Ok(path)
}
