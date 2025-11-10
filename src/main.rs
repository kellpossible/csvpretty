use clap::Parser;
use csv::ReaderBuilder;
use owo_colors::{OwoColorize, Rgb};
use std::io::{self, Read};
use terminal_colorsaurus::{theme_mode, QueryOptions, ThemeMode};
use unicode_width::UnicodeWidthStr;

/// Color palette for dark terminal themes.
/// Colors cycle through columns: Orange → Cyan → Purple → Pink → Yellow → (repeat)
///
/// These RGB values are taken from the csvlens project:
/// https://github.com/YS-L/csvlens/blob/main/src/theme.rs
const DARK_THEME_COLORS: [(u8, u8, u8); 5] = [
    (253, 151, 31),  // Orange
    (102, 217, 239), // Cyan
    (190, 132, 255), // Purple
    (249, 38, 114),  // Pink
    (230, 219, 116), // Yellow
];

/// Color palette for light terminal themes.
/// Darker variants of the dark theme colors for better contrast on light backgrounds.
///
/// These RGB values are taken from the csvlens project:
/// https://github.com/YS-L/csvlens/blob/main/src/theme.rs
const LIGHT_THEME_COLORS: [(u8, u8, u8); 5] = [
    (207, 112, 0),   // Darker Orange
    (0, 137, 179),   // Darker Cyan/Blue
    (104, 77, 153),  // Darker Purple
    (249, 0, 90),    // Darker Pink
    (153, 143, 47),  // Darker Yellow/Olive
];

/// Detects the terminal's theme (dark/light) and returns the appropriate color palette.
/// Queries the terminal using OSC escape sequences to determine background color.
/// Falls back to dark theme if detection fails.
fn detect_theme() -> &'static [(u8, u8, u8); 5] {
    match theme_mode(QueryOptions::default()) {
        Ok(ThemeMode::Dark) => &DARK_THEME_COLORS,
        Ok(ThemeMode::Light) => &LIGHT_THEME_COLORS,
        _ => &DARK_THEME_COLORS, // Default to dark theme on error
    }
}

/// Gets the RGB color for a column index using modulo to cycle through the palette.
/// Example: columns 0-4 use colors 0-4, column 5 wraps to color 0, etc.
fn get_column_color(col_index: usize, theme: &[(u8, u8, u8); 5]) -> (u8, u8, u8) {
    theme[col_index % theme.len()]
}

/// Configuration for table rendering.
/// Consolidates display options to reduce function parameter counts.
struct RenderConfig<'a> {
    wrap_mode: WrapMode,
    show_line_numbers: bool,
    /// Theme colors if enabled. None when --no-color is used.
    theme: Option<&'a [(u8, u8, u8); 5]>,
    terminal_width: usize,
}

#[derive(Parser, Debug)]
#[command(name = "csvpretty")]
#[command(about = "Format CSV input into a beautiful table", long_about = None)]
struct Args {
    /// Text wrapping mode: word, char, or none
    #[arg(long, default_value = "word")]
    wrap: WrapMode,

    /// Show line numbers
    #[arg(short = 'n', long)]
    line_numbers: bool,

    /// Disable column colors
    #[arg(long)]
    no_color: bool,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
enum WrapMode {
    Word,
    Char,
    None,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Read all stdin
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    if input.trim().is_empty() {
        eprintln!("Error: No CSV input provided");
        std::process::exit(1);
    }

    // Parse CSV
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(input.as_bytes());

    let headers = reader.headers()?.clone();
    let header_count = headers.len();

    // Collect all records
    let mut records: Vec<Vec<String>> = Vec::new();
    for result in reader.records() {
        let record = result?;
        let mut row: Vec<String> = record.iter().map(|s| s.to_string()).collect();

        // Pad row if it has fewer columns than headers
        while row.len() < header_count {
            row.push(String::new());
        }

        records.push(row);
    }

    // Get terminal width (or use large value for no-wrap mode)
    let terminal_width = match args.wrap {
        WrapMode::None => usize::MAX,
        _ => terminal_size::terminal_size()
            .map(|(w, _)| w.0 as usize)
            .unwrap_or(80),
    };

    // Detect theme and check if colors should be enabled
    // Colors are enabled by default unless --no-color flag or NO_COLOR env var is set
    let colors_enabled = !args.no_color && std::env::var("NO_COLOR").is_err();
    let theme = if colors_enabled {
        Some(detect_theme())
    } else {
        None
    };

    // Create render configuration
    let config = RenderConfig {
        wrap_mode: args.wrap,
        show_line_numbers: args.line_numbers,
        theme,
        terminal_width,
    };

    // Render the table
    render_table(&headers, &records, &config);

    Ok(())
}

fn render_table(headers: &csv::StringRecord, records: &[Vec<String>], config: &RenderConfig) {
    let header_vec: Vec<&str> = headers.iter().collect();

    // Calculate row number width (for the leftmost column)
    let row_num_width = if config.show_line_numbers {
        records.len().to_string().len().max(1)
    } else {
        0
    };

    // Calculate column widths
    let col_widths = calculate_column_widths(&header_vec, records, config.terminal_width, config.wrap_mode, row_num_width);

    // Render top border
    print_horizontal_border(&col_widths, row_num_width, BorderType::Top, config.show_line_numbers);

    // Render header
    print_header_row(&header_vec, &col_widths, row_num_width, config);

    // Render separator after header
    print_horizontal_border(&col_widths, row_num_width, BorderType::HeaderSeparator, config.show_line_numbers);

    // Render data rows
    for (idx, record) in records.iter().enumerate() {
        print_data_row(idx + 1, record, &col_widths, row_num_width, config);
    }

    // Render bottom border (only for no-wrap mode to match the example)
    if matches!(config.wrap_mode, WrapMode::None) {
        print_horizontal_border(&col_widths, row_num_width, BorderType::Bottom, config.show_line_numbers);
    }
}

/// Calculates column widths based on content and terminal constraints.
///
/// For no-wrap mode: columns are sized to fit their content exactly (table may exceed terminal width).
///
/// For wrap modes: uses a "waterfall" allocation strategy:
/// 1. Calculate natural width (max content width) for each column
/// 2. If all columns fit naturally, use those widths
/// 3. Otherwise: allocate natural width to smallest columns first, then distribute
///    remaining space proportionally to larger columns that need wrapping
///
/// This ensures narrow columns don't get over-allocated space while wide columns share
/// the burden of wrapping.
fn calculate_column_widths(headers: &[&str], records: &[Vec<String>], terminal_width: usize, wrap_mode: WrapMode, row_num_width: usize) -> Vec<usize> {
    let num_cols = headers.len();

    if matches!(wrap_mode, WrapMode::None) {
        // For no-wrap mode, size columns to content
        let mut widths = Vec::new();
        for col_idx in 0..num_cols {
            let header_width = UnicodeWidthStr::width(headers[col_idx]);
            let max_content_width = records.iter()
                .map(|row| {
                    row.get(col_idx)
                        .map(|s| UnicodeWidthStr::width(s.as_str()))
                        .unwrap_or(0)
                })
                .max()
                .unwrap_or(0);
            widths.push(header_width.max(max_content_width) + 2); // +2 for padding
        }
        widths
    } else {
        // For wrap modes, distribute terminal width
        // Calculate overhead: row number column + borders + padding
        // Format with line numbers: "N  │ content │ content │"
        // Format without line numbers: " content │ content │"
        // Row number area (if enabled): N (row_num_width) + "  │" (3 chars)
        // Each column: " content │" (1 space before + content + 1 space + 1 separator = content + 3)
        // So overhead is everything except the content widths
        let row_overhead = if row_num_width > 0 {
            row_num_width + 3  // "N  │"
        } else {
            0  // No row number column
        };
        let overhead = row_overhead + (num_cols * 3);

        let available_width = terminal_width.saturating_sub(overhead);

        // Calculate natural widths for proportional distribution
        let mut natural_widths = Vec::new();
        for col_idx in 0..num_cols {
            let header_width = UnicodeWidthStr::width(headers[col_idx]);
            let max_content_width = records.iter()
                .map(|row| {
                    row.get(col_idx)
                        .map(|s| UnicodeWidthStr::width(s.as_str()))
                        .unwrap_or(0)
                })
                .max()
                .unwrap_or(0);
            natural_widths.push(header_width.max(max_content_width));
        }

        let total_natural: usize = natural_widths.iter().sum();

        if total_natural == 0 {
            return vec![10; num_cols]; // Fallback
        }

        // Strategy: Give columns their natural width if possible, wrap only when needed
        let mut widths = vec![0; num_cols];

        // Check if all columns fit naturally
        if total_natural <= available_width {
            // All columns fit, just give them their natural widths
            for (i, &natural) in natural_widths.iter().enumerate() {
                widths[i] = natural;
            }
            // Distribute any remaining space to the last column
            let used: usize = widths.iter().sum();
            if used < available_width {
                widths[num_cols - 1] += available_width - used;
            }
        } else {
            // Not all columns fit, need to wrap
            // Strategy: Give smaller columns their natural width, let bigger columns share remaining

            // Sort column indices by their natural width
            let mut sorted_cols: Vec<(usize, usize)> = natural_widths.iter()
                .enumerate()
                .map(|(i, &w)| (i, w))
                .collect();
            sorted_cols.sort_by_key(|&(_, w)| w);

            let mut remaining = available_width;
            let mut unallocated_cols = num_cols;

            // Allocate to smallest columns first
            for &(col_idx, natural) in &sorted_cols {
                let avg_remaining = remaining / unallocated_cols;

                if natural <= avg_remaining {
                    // This column can have its natural width
                    widths[col_idx] = natural;
                    remaining = remaining.saturating_sub(natural);
                } else {
                    // This and remaining larger columns need to share
                    break;
                }
                unallocated_cols -= 1;
            }

            // Distribute remaining space to unallocated columns proportionally
            if unallocated_cols > 0 {
                let unallocated_natural: usize = sorted_cols.iter()
                    .filter(|(i, _)| widths[*i] == 0)
                    .map(|(_, w)| w)
                    .sum();

                let per_col_min = remaining / unallocated_cols;
                let mut leftover = remaining;

                for &(col_idx, natural) in &sorted_cols {
                    if widths[col_idx] == 0 {
                        unallocated_cols -= 1;
                        if unallocated_cols == 0 {
                            // Last column gets remainder
                            widths[col_idx] = leftover.max(5);
                        } else if unallocated_natural > 0 {
                            // Proportional allocation
                            let alloc = ((remaining * natural) / unallocated_natural).max(per_col_min).max(5);
                            widths[col_idx] = alloc;
                            leftover = leftover.saturating_sub(alloc);
                        } else {
                            widths[col_idx] = per_col_min.max(5);
                            leftover = leftover.saturating_sub(per_col_min.max(5));
                        }
                    }
                }
            }
        }

        widths
    }
}

enum BorderType {
    Top,
    HeaderSeparator,
    Bottom,
}

fn print_horizontal_border(col_widths: &[usize], row_num_width: usize, border_type: BorderType, show_line_numbers: bool) {
    match border_type {
        BorderType::Top => {
            // Top border: just a line across the header
            let row_area = if show_line_numbers { row_num_width + 3 } else { 0 };
            // Each column contributes width + 3 (space + content + space + separator)
            // but the last column has no separator, so subtract 1
            let total_width: usize = row_area + col_widths.iter().map(|w| w + 3).sum::<usize>() - 1;
            println!("{}", "─".repeat(total_width));
        }
        BorderType::HeaderSeparator => {
            // Separator after header: ────┬────┬────
            if show_line_numbers {
                // Row number area is: "{:>width$}  │" = row_num_width + 3 chars total
                // The ┬ replaces the │, so we need row_num_width + 2 dashes before it
                print!("{}", "─".repeat(row_num_width + 2));
                print!("┬");
            }
            for (i, &width) in col_widths.iter().enumerate() {
                // Each column prints: " {text}{padding}" with optional " │" between
                // The ┬ replaces the │, so we need width + 2 dashes before it
                print!("{}", "─".repeat(width + 2));
                // Print ┬ only between columns, not after the last one
                if i < col_widths.len() - 1 {
                    print!("┬");
                }
            }
            println!();
        }
        BorderType::Bottom => {
            // Bottom border (for no-wrap mode)
            if show_line_numbers {
                print!("{}", "─".repeat(row_num_width + 2));
                print!("┴");
            }
            for (i, &width) in col_widths.iter().enumerate() {
                print!("{}", "─".repeat(width + 2));
                // Print ┴ only between columns, not after the last one
                if i < col_widths.len() - 1 {
                    print!("┴");
                }
            }
            println!();
        }
    }
}

/// Prints the header row with optional colors and bold formatting.
/// Each column gets a color from the theme palette, cycling through colors.
/// Headers are always bold when colors are enabled.
fn print_header_row(headers: &[&str], col_widths: &[usize], row_num_width: usize, config: &RenderConfig) {
    // Match the data row format: "{:>width$}  │" = row_num_width + 3 chars (if line numbers enabled)
    if config.show_line_numbers {
        print!("{}", " ".repeat(row_num_width + 3));
    }
    for (i, &header) in headers.iter().enumerate() {
        let width = col_widths[i];
        let header_width = UnicodeWidthStr::width(header);
        let padding = width.saturating_sub(header_width);

        // Apply color if theme is enabled (same color as data cells in this column)
        if let Some(theme) = config.theme {
            let (r, g, b) = get_column_color(i, theme);
            print!(" {}{}", header.color(Rgb(r, g, b)).bold(), " ".repeat(padding));
        } else {
            print!(" {}{}", header, " ".repeat(padding));
        }

        // Print separator only between columns, not after the last one
        if i < headers.len() - 1 {
            print!(" │");
        }
    }
    println!();
}

/// Prints a data row with optional line numbers and colors.
/// Handles multi-line cells by wrapping text and aligning all cells to the tallest cell.
/// Each column uses the same color as its header (cycling through the palette).
fn print_data_row(row_num: usize, record: &[String], col_widths: &[usize], row_num_width: usize, config: &RenderConfig) {
    // Wrap each cell and determine max lines needed
    let wrapped_cells: Vec<Vec<String>> = record.iter()
        .zip(col_widths.iter())
        .map(|(cell, &width)| wrap_text(cell, width, config.wrap_mode))
        .collect();

    let max_lines = wrapped_cells.iter().map(|lines| lines.len()).max().unwrap_or(1);

    // Print each line of the multi-line row
    for line_idx in 0..max_lines {
        if config.show_line_numbers {
            if line_idx == 0 {
                // First line: show row number
                print!("{:>width$}  │", row_num, width = row_num_width);
            } else {
                // Subsequent lines: empty row number area for alignment
                print!("{}  │", " ".repeat(row_num_width));
            }
        }

        for (col_idx, lines) in wrapped_cells.iter().enumerate() {
            let width = col_widths[col_idx];
            let text = lines.get(line_idx).map(|s| s.as_str()).unwrap_or("");
            let text_width = UnicodeWidthStr::width(text);
            let padding = width.saturating_sub(text_width);

            // Apply color if theme is enabled
            if let Some(theme) = config.theme {
                let (r, g, b) = get_column_color(col_idx, theme);
                print!(" {}{}", text.color(Rgb(r, g, b)), " ".repeat(padding));
            } else {
                print!(" {}{}", text, " ".repeat(padding));
            }

            // Print separator only between columns, not after the last one
            if col_idx < wrapped_cells.len() - 1 {
                print!(" │");
            }
        }
        println!();
    }
}

fn wrap_text(text: &str, max_width: usize, wrap_mode: WrapMode) -> Vec<String> {
    if text.is_empty() {
        return vec![String::new()];
    }

    match wrap_mode {
        WrapMode::None => {
            vec![text.to_string()]
        }
        WrapMode::Word => {
            wrap_text_word(text, max_width)
        }
        WrapMode::Char => {
            wrap_text_char(text, max_width)
        }
    }
}

fn wrap_text_word(text: &str, max_width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();
    let mut current_width = 0;

    for word in text.split_whitespace() {
        let word_width = UnicodeWidthStr::width(word);

        if current_width == 0 {
            // First word on line
            if word_width <= max_width {
                current_line = word.to_string();
                current_width = word_width;
            } else {
                // Word is too long, split it character by character
                for line in wrap_text_char(word, max_width) {
                    lines.push(line);
                }
            }
        } else if current_width + 1 + word_width <= max_width {
            // Add word to current line
            current_line.push(' ');
            current_line.push_str(word);
            current_width += 1 + word_width;
        } else {
            // Start new line
            lines.push(current_line);
            if word_width <= max_width {
                current_line = word.to_string();
                current_width = word_width;
            } else {
                // Word is too long, split it
                current_line = String::new();
                current_width = 0;
                for line in wrap_text_char(word, max_width) {
                    lines.push(line);
                }
            }
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}

fn wrap_text_char(text: &str, max_width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();
    let mut current_width = 0;

    for ch in text.chars() {
        let ch_width = UnicodeWidthStr::width(ch.to_string().as_str());

        if current_width + ch_width <= max_width {
            current_line.push(ch);
            current_width += ch_width;
        } else {
            if !current_line.is_empty() {
                lines.push(current_line);
            }
            current_line = ch.to_string();
            current_width = ch_width;
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}
