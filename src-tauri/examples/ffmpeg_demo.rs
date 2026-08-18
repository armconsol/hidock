/// FFmpeg Integration Demo
///
/// This example demonstrates the FFmpeg wrapper functionality.
/// Run with: cargo run --example ffmpeg_demo
use hinotes_desktop_lib::audio::FFmpegWrapper;
use std::env;
use std::path::PathBuf;
use tempfile::TempDir;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    println!("=== FFmpeg Integration Demo ===\n");

    // Initialize FFmpeg wrapper
    println!("1. Initializing FFmpeg wrapper...");
    let wrapper = match FFmpegWrapper::new() {
        Ok(w) => {
            println!("   ✓ FFmpeg found at: {}", w.binary_path().display());
            w
        }
        Err(e) => {
            eprintln!("   ✗ FFmpeg not found: {}", e);
            eprintln!("\nPlease install FFmpeg:");
            eprintln!("  macOS:   brew install ffmpeg");
            eprintln!("  Linux:   sudo apt install ffmpeg");
            eprintln!("  Windows: Download from https://ffmpeg.org/download.html");
            return Err(e);
        }
    };

    // Validate FFmpeg installation
    println!("\n2. Validating FFmpeg installation...");
    match wrapper.validate() {
        Ok(version) => println!("   ✓ {}", version),
        Err(e) => {
            eprintln!("   ✗ Validation failed: {}", e);
            return Err(e);
        }
    }

    // Check for demo audio file
    println!("\n3. Checking for demo audio file...");
    let demo_file = env::args().nth(1);

    if demo_file.is_none() {
        println!("   ℹ No audio file provided");
        println!("\nUsage: cargo run --example ffmpeg_demo [audio_file.wav]");
        println!("\nWithout an audio file, only basic validation is performed.");
        return Ok(());
    }

    let input_path = PathBuf::from(demo_file.unwrap());
    if !input_path.exists() {
        eprintln!("   ✗ File not found: {}", input_path.display());
        return Ok(());
    }

    println!("   ✓ Using file: {}", input_path.display());

    // Get audio info
    println!("\n4. Reading audio metadata...");
    match wrapper.get_audio_info(&input_path) {
        Ok(info) => {
            println!("   ✓ Duration: {:.2} seconds", info.duration);
        }
        Err(e) => {
            eprintln!("   ✗ Failed to read metadata: {}", e);
            return Err(e);
        }
    }

    // Create temporary directory for output
    let temp_dir = TempDir::new()?;
    println!("\n5. Testing audio operations...");
    println!("   Output directory: {}", temp_dir.path().display());

    // Test 1: Convert to MP3
    println!("\n   Test 1: Converting to MP3...");
    let mp3_output = temp_dir.path().join("output.mp3");
    match wrapper.convert_audio(&input_path, &mp3_output, "mp3") {
        Ok(_) => {
            println!("      ✓ Conversion successful");
            println!("      → {}", mp3_output.display());
        }
        Err(e) => {
            eprintln!("      ✗ Conversion failed: {}", e);
        }
    }

    // Test 2: Extract segment
    println!("\n   Test 2: Extracting 5-second segment...");
    let segment_output = temp_dir.path().join("segment.wav");
    match wrapper.extract_segment(&input_path, &segment_output, 0.0, 5.0, "wav") {
        Ok(_) => {
            println!("      ✓ Extraction successful");
            println!("      → {}", segment_output.display());

            // Verify segment duration
            if let Ok(info) = wrapper.get_audio_info(&segment_output) {
                println!("      → Segment duration: {:.2} seconds", info.duration);
            }
        }
        Err(e) => {
            eprintln!("      ✗ Extraction failed: {}", e);
        }
    }

    // Test 3: Merge files (duplicate the input twice)
    println!("\n   Test 3: Merging audio files...");
    let merged_output = temp_dir.path().join("merged.wav");

    // Create a second temp file (copy of input for demo)
    let temp_input2 = temp_dir.path().join("temp_input2.wav");
    std::fs::copy(&input_path, &temp_input2)?;

    match wrapper.merge_audio_files(&[&input_path, &temp_input2], &merged_output, "wav") {
        Ok(_) => {
            println!("      ✓ Merge successful");
            println!("      → {}", merged_output.display());

            // Verify merged duration
            if let Ok(info) = wrapper.get_audio_info(&merged_output) {
                println!("      → Merged duration: {:.2} seconds", info.duration);
            }
        }
        Err(e) => {
            eprintln!("      ✗ Merge failed: {}", e);
        }
    }

    println!("\n=== Demo Complete ===");
    println!("\nOutput files are in: {}", temp_dir.path().display());
    println!("(They will be deleted when this program exits)\n");

    // Keep temp dir alive for inspection if desired
    // Uncomment to prevent cleanup:
    // let _ = temp_dir.into_path();

    Ok(())
}
