# csvpretty

A command-line tool that formats CSV input into beautifully formatted tables with Unicode box-drawing characters.

## Features

- Reads CSV from stdin
- Automatically formats data into a nicely aligned table
- Handles multi-line cell content with word wrapping
- Fits output to terminal width (or allows overflow for paging)
- Adds row numbers for easy reference
- Uses Unicode box-drawing characters for clean borders

## Installation

```bash
cargo build --release
cargo install --path .
```

## Usage

```bash
cat data.csv | csvpretty [OPTIONS]
```

### Options

- `--wrap <MODE>`: Control text wrapping behavior (default: `word`)
  - `word`: Wrap text at word boundaries, fit to terminal width
  - `char`: Wrap text at any character, fit to terminal width
  - `none`: No wrapping, columns sized to content (table may exceed terminal width, pipe to pager)

- `-n, --line-numbers`: Show line numbers (disabled by default)

### Examples

**Basic usage with word wrapping (default):**
```bash
cat data.csv | csvpretty
```

**With line numbers:**
```bash
cat data.csv | csvpretty -n
# or
cat data.csv | csvpretty --line-numbers
```

**No wrapping (for use with pager):**
```bash
cat data.csv | csvpretty --wrap none | less -S
```

**Character-based wrapping:**
```bash
cat data.csv | csvpretty --wrap char
```

## Example Output

Input CSV:
```
tablename,comment
after_payment_human_product_survey_answers,"Per-product, per-human (who is a participant in any order item) surveys, to be completed after payment."
after_payment_human_survey_answers,"Per-human (who is a participant in any order item) surveys, to be completed after payment."
```

Output:
```
──────────────────────────────────────────────────────────────────────────────────────────
       tablename                                       comment
────┬────────────────────────────────────────────────────────────────────────────────────┬
1   │  after_payment_human_product_survey_answers      Per-product, per-human (who is a  │
    │                                                  participant in any order item)    │
    │                                                  surveys, to be completed after    │
    │                                                  payment.                          │
2   │  after_payment_human_survey_answers              Per-human (who is a participant  │
    │                                                  in any order item) surveys, to be │
    │                                                  completed after payment.          │
```

## Behavior

### Terminal Width Handling

- **Word/Char wrap modes**: Table width matches terminal width. Columns are proportionally sized based on content, and text wraps to fit.
- **No wrap mode**: Columns are sized to fit their content. Table may exceed terminal width. Use with a pager like `less -S` for horizontal scrolling.

### Malformed CSV Handling

- Rows with fewer columns than the header are padded with empty values
- Processing continues gracefully with inconsistent row lengths

## TODO

- [ ] Create project structure and documentation
  - [x] README.md with requirements and examples
  - [ ] TODO tracking section
- [ ] Set up Rust project configuration
  - [ ] Fix Cargo.toml edition (2024 → 2021)
  - [ ] Add `csv` dependency for CSV parsing
  - [ ] Add `unicode-width` dependency for accurate width calculations
  - [ ] Add `terminal_size` dependency for terminal width detection
  - [ ] Add `clap` dependency for CLI argument parsing
- [ ] Implement core functionality
  - [ ] CLI argument parsing with `--wrap` option
  - [ ] CSV reading from stdin
  - [ ] Handle malformed CSV (pad missing columns)
  - [ ] Terminal width detection
  - [ ] Column width calculation (terminal-constrained vs content-based)
  - [ ] Text wrapping logic (word/char/none modes)
  - [ ] Table rendering with Unicode box-drawing characters
  - [ ] Row numbering
  - [ ] Multi-line cell alignment
- [ ] Error handling
  - [ ] Empty CSV input
  - [ ] IO errors from stdin
  - [ ] Clear error messages
- [ ] Testing
  - [ ] Test with sample CSV data
  - [ ] Test all wrap modes
  - [ ] Test malformed CSV handling
  - [ ] Test edge cases (empty cells, very long text, etc.)

## License

MIT
