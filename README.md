# csvpretty

A command-line tool that formats CSV input into tables with Unicode box-drawing characters.

Output style and color themes are based on [csvlens](https://github.com/YS-L/csvlens).

## Installation

```bash
cargo install --path .
```

## Usage

```bash
cat data.csv | csvpretty
```

### Options

```
Format CSV input into a beautiful table

Usage: csvpretty [OPTIONS]

Options:
      --wrap <WRAP>   Text wrapping mode: word, char, or none [default: word] [possible values: word, char, none]
  -n, --line-numbers  Show line numbers
      --no-color      Disable column colors
  -h, --help          Print help
```

## Examples

```bash
# Basic usage with colors and word wrapping
cat data.csv | csvpretty

# With line numbers
cat data.csv | csvpretty -n

# No wrapping, for use with pager
cat data.csv | csvpretty --wrap none | less -S

# Without colors
cat data.csv | csvpretty --no-color
```

## License

MIT
