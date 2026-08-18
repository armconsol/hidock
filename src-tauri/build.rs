use std::env;
use std::path::PathBuf;

fn main() {
    tauri_build::build();

    // Optionally download FFmpeg binaries during build
    // Set FFMPEG_DOWNLOAD=1 environment variable to enable
    if env::var("FFMPEG_DOWNLOAD").unwrap_or_default() == "1" {
        println!("cargo:warning=FFmpeg download enabled via FFMPEG_DOWNLOAD=1");
        download_ffmpeg_binaries();
    } else {
        println!("cargo:warning=FFmpeg auto-download disabled. Set FFMPEG_DOWNLOAD=1 to enable.");
        println!("cargo:warning=FFmpeg will be detected from system PATH at runtime.");
    }
}

fn download_ffmpeg_binaries() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();

    println!(
        "cargo:warning=Target platform: {} {}",
        target_os, target_arch
    );

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let binaries_dir = manifest_dir.join("binaries").join(&target_os);

    // Create binaries directory if it doesn't exist
    if let Err(e) = std::fs::create_dir_all(&binaries_dir) {
        println!("cargo:warning=Failed to create binaries directory: {}", e);
        return;
    }

    let binary_name = if target_os == "windows" {
        "ffmpeg.exe"
    } else {
        "ffmpeg"
    };

    let binary_path = binaries_dir.join(binary_name);

    // Skip download if binary already exists
    if binary_path.exists() {
        println!(
            "cargo:warning=FFmpeg binary already exists at: {}",
            binary_path.display()
        );
        return;
    }

    println!("cargo:warning=FFmpeg binary download not yet implemented");
    println!("cargo:warning=Manual installation required:");

    match target_os.as_str() {
        "macos" => {
            println!("cargo:warning=  brew install ffmpeg");
            println!("cargo:warning=Or download from: https://evermeet.cx/ffmpeg/");
        }
        "linux" => {
            println!("cargo:warning=  sudo apt install ffmpeg  # Debian/Ubuntu");
            println!("cargo:warning=  sudo dnf install ffmpeg  # Fedora");
            println!("cargo:warning=Or download from: https://johnvansickle.com/ffmpeg/");
        }
        "windows" => {
            println!("cargo:warning=Download from: https://www.gyan.dev/ffmpeg/builds/");
            println!("cargo:warning=Or: https://github.com/BtbN/FFmpeg-Builds/releases");
        }
        _ => {
            println!("cargo:warning=Platform not supported for auto-download");
        }
    }

    // TODO: Implement actual download logic
    // This would typically:
    // 1. Download the appropriate FFmpeg build from a trusted source
    // 2. Verify checksums
    // 3. Extract to binaries directory
    // 4. Set executable permissions (Unix)
}
