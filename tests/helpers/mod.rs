use portable_pty::{CommandBuilder, PtySize, PtySystem, native_pty_system};
use std::io::{Read, Write};
use std::path::PathBuf;

/// Get the path to the compiled csvpretty binary
pub fn get_binary_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("target");
    path.push("debug");
    path.push("csvpretty");
    path
}

/// Get the path to a test fixture file
pub fn fixture_path(name: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("fixtures");
    path.push(name);
    path
}

/// Load a test fixture file as a string
pub fn load_fixture(name: &str) -> String {
    std::fs::read_to_string(fixture_path(name))
        .unwrap_or_else(|e| panic!("Failed to load fixture {}: {}", name, e))
}

/// Run csvpretty in a PTY with specified terminal width and arguments
pub fn run_csvpretty_in_pty(
    csv_input: &str,
    terminal_cols: u16,
    args: &[&str],
) -> Result<String, Box<dyn std::error::Error>> {
    let binary_path = get_binary_path();

    // Create PTY with specific width
    let pty_system = native_pty_system();
    let pair = pty_system.openpty(PtySize {
        rows: 24,
        cols: terminal_cols,
        pixel_width: 0,
        pixel_height: 0,
    })?;

    // Build command - always add --no-color first for tests
    let mut cmd = CommandBuilder::new(&binary_path);
    cmd.arg("--no-color");
    for arg in args {
        cmd.arg(arg);
    }

    // Spawn process
    let mut child = pair.slave.spawn_command(cmd)?;
    drop(pair.slave); // Close slave end to avoid deadlock

    // Write CSV input to stdin
    let mut writer = pair.master.take_writer()?;
    writer.write_all(csv_input.as_bytes())?;
    drop(writer); // Close stdin

    // Read output
    let mut reader = pair.master.try_clone_reader()?;
    let mut output = String::new();
    reader.read_to_string(&mut output)?;

    // Wait for child to exit
    let _ = child.wait()?;

    // Clean up the output:
    // PTY echoes input and adds control characters
    // We want to remove the echoed CSV input and any control characters
    let cleaned = clean_pty_output(&output, csv_input);

    Ok(cleaned)
}

/// Cleans PTY output by removing echoed input and control characters.
///
/// PTYs echo stdin back to the output and inject control characters. This function:
/// 1. Finds the table's top border (line with >10 '─' characters) to skip echoed CSV
/// 2. Strips ANSI control characters and PTY artifacts (^D, ^C, ␈, ␊)
/// 3. Extracts only the table border from the first line (sometimes CSV is concatenated)
/// 4. Trims empty lines from start/end
fn clean_pty_output(output: &str, _input: &str) -> String {
    // Split output into lines
    let lines: Vec<&str> = output.lines().collect();

    // Find the start of the table by looking for a line with many ─ characters
    // (the top border). The PTY may add control characters, so we check if the line
    // contains a significant number of dashes.
    let start_idx = lines.iter().position(|line| {
        let dash_count = line.chars().filter(|&c| c == '─').count();
        dash_count > 10 // Top border should have many dashes
    }).unwrap_or(0);

    // Clean each line: remove control characters, keep only printable chars
    let mut cleaned_lines: Vec<String> = lines[start_idx..]
        .iter()
        .enumerate()
        .map(|(idx, line)| {
            let mut cleaned = line.chars()
                .filter(|c| {
                    !c.is_control() ||  // Remove control chars
                    *c == '\t'          // Keep tabs (though we don't expect them)
                })
                .collect::<String>();

            // Remove common PTY artifacts (^D, ^C, etc shown as literal text)
            // These are sometimes rendered as caret notation
            cleaned = cleaned.replace("^D", "");
            cleaned = cleaned.replace("^C", "");
            cleaned = cleaned.replace("␈", ""); // Backspace symbol
            cleaned = cleaned.replace("␊", ""); // Newline symbol

            // For the first line (top border), extract only the border part
            // Sometimes PTY concatenates the last CSV line with the border
            if idx == 0 {
                // Find where the continuous sequence of ─ characters starts
                if let Some(dash_start) = cleaned.find('─') {
                    // Extract from the first dash to the end
                    cleaned = cleaned[dash_start..].to_string();
                }
            }

            cleaned
        })
        .collect();

    // Remove empty or whitespace-only lines from the end
    while let Some(last) = cleaned_lines.last() {
        if last.trim().is_empty() {
            cleaned_lines.pop();
        } else {
            break;
        }
    }

    // Remove empty lines from the beginning
    while let Some(first) = cleaned_lines.first() {
        if first.trim().is_empty() {
            cleaned_lines.remove(0);
        } else {
            break;
        }
    }

    // Join back into a string
    cleaned_lines.join("\n")
}

/// Run csvpretty in a PTY with specified terminal width (no additional args)
pub fn run_with_width(csv_input: &str, width: u16) -> String {
    run_csvpretty_in_pty(csv_input, width, &[])
        .expect("Failed to run csvpretty")
}

/// Run csvpretty in a PTY with default width (80 columns)
pub fn run_default(csv_input: &str, args: &[&str]) -> String {
    run_csvpretty_in_pty(csv_input, 80, args)
        .expect("Failed to run csvpretty")
}
