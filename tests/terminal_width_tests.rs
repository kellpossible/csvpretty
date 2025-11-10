mod helpers;

use helpers::*;

#[test]
fn test_narrow_terminal_40_cols() {
    let csv_input = load_fixture("simple.csv");
    let output = run_csvpretty_in_pty(&csv_input, 40, &["--wrap", "word"])
        .expect("Failed to run csvpretty");

    insta::assert_snapshot!("narrow_40_cols_word_wrap", output);
}

#[test]
fn test_standard_terminal_80_cols() {
    let csv_input = load_fixture("simple.csv");
    let output = run_csvpretty_in_pty(&csv_input, 80, &["--wrap", "word"])
        .expect("Failed to run csvpretty");

    insta::assert_snapshot!("standard_80_cols_word_wrap", output);
}

#[test]
fn test_wide_terminal_120_cols() {
    let csv_input = load_fixture("simple.csv");
    let output = run_csvpretty_in_pty(&csv_input, 120, &["--wrap", "word"])
        .expect("Failed to run csvpretty");

    insta::assert_snapshot!("wide_120_cols_word_wrap", output);
}

#[test]
fn test_very_wide_terminal_200_cols() {
    let csv_input = load_fixture("simple.csv");
    let output = run_csvpretty_in_pty(&csv_input, 200, &["--wrap", "word"])
        .expect("Failed to run csvpretty");

    insta::assert_snapshot!("very_wide_200_cols_word_wrap", output);
}

#[test]
fn test_long_text_narrow_80_cols() {
    let csv_input = load_fixture("long_text.csv");
    let output = run_csvpretty_in_pty(&csv_input, 80, &["--wrap", "word"])
        .expect("Failed to run csvpretty");

    insta::assert_snapshot!("long_text_narrow_80_cols", output);
}

#[test]
fn test_long_text_wide_120_cols() {
    let csv_input = load_fixture("long_text.csv");
    let output = run_csvpretty_in_pty(&csv_input, 120, &["--wrap", "word"])
        .expect("Failed to run csvpretty");

    insta::assert_snapshot!("long_text_wide_120_cols", output);
}

#[test]
fn test_many_rows_80_cols() {
    let csv_input = load_fixture("many_rows.csv");
    let output = run_csvpretty_in_pty(&csv_input, 80, &["--wrap", "word"])
        .expect("Failed to run csvpretty");

    insta::assert_snapshot!("many_rows_80_cols", output);
}

#[test]
fn test_many_rows_120_cols() {
    let csv_input = load_fixture("many_rows.csv");
    let output = run_csvpretty_in_pty(&csv_input, 120, &["--wrap", "word"])
        .expect("Failed to run csvpretty");

    insta::assert_snapshot!("many_rows_120_cols", output);
}
