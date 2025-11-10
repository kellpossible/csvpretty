mod helpers;

use helpers::*;

#[test]
fn test_without_line_numbers() {
    let csv_input = load_fixture("simple.csv");
    let output = run_csvpretty_in_pty(&csv_input, 80, &[])
        .expect("Failed to run csvpretty");

    insta::assert_snapshot!("without_line_numbers", output);
}

#[test]
fn test_with_line_numbers_short_flag() {
    let csv_input = load_fixture("simple.csv");
    let output = run_csvpretty_in_pty(&csv_input, 80, &["-n"])
        .expect("Failed to run csvpretty");

    insta::assert_snapshot!("with_line_numbers_short_flag", output);
}

#[test]
fn test_with_line_numbers_long_flag() {
    let csv_input = load_fixture("simple.csv");
    let output = run_csvpretty_in_pty(&csv_input, 80, &["--line-numbers"])
        .expect("Failed to run csvpretty");

    insta::assert_snapshot!("with_line_numbers_long_flag", output);
}

#[test]
fn test_line_numbers_with_word_wrap() {
    let csv_input = load_fixture("long_text.csv");
    let output = run_csvpretty_in_pty(&csv_input, 80, &["-n", "--wrap", "word"])
        .expect("Failed to run csvpretty");

    insta::assert_snapshot!("line_numbers_with_word_wrap", output);
}

#[test]
fn test_line_numbers_with_char_wrap() {
    let csv_input = load_fixture("long_text.csv");
    let output = run_csvpretty_in_pty(&csv_input, 80, &["-n", "--wrap", "char"])
        .expect("Failed to run csvpretty");

    insta::assert_snapshot!("line_numbers_with_char_wrap", output);
}

#[test]
fn test_line_numbers_with_none_wrap() {
    let csv_input = load_fixture("long_text.csv");
    let output = run_csvpretty_in_pty(&csv_input, 80, &["-n", "--wrap", "none"])
        .expect("Failed to run csvpretty");

    insta::assert_snapshot!("line_numbers_with_none_wrap", output);
}

#[test]
fn test_line_numbers_many_rows() {
    let csv_input = load_fixture("many_rows.csv");
    let output = run_csvpretty_in_pty(&csv_input, 80, &["-n"])
        .expect("Failed to run csvpretty");

    insta::assert_snapshot!("line_numbers_many_rows", output);
}

#[test]
fn test_no_line_numbers_many_rows() {
    let csv_input = load_fixture("many_rows.csv");
    let output = run_csvpretty_in_pty(&csv_input, 80, &[])
        .expect("Failed to run csvpretty");

    insta::assert_snapshot!("no_line_numbers_many_rows", output);
}
