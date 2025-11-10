mod helpers;

use helpers::*;

#[test]
fn test_word_wrap_mode() {
    let csv_input = load_fixture("long_text.csv");
    let output = run_csvpretty_in_pty(&csv_input, 80, &["--wrap", "word"])
        .expect("Failed to run csvpretty");

    insta::assert_snapshot!("word_wrap_mode", output);
}

#[test]
fn test_char_wrap_mode() {
    let csv_input = load_fixture("long_text.csv");
    let output = run_csvpretty_in_pty(&csv_input, 80, &["--wrap", "char"])
        .expect("Failed to run csvpretty");

    insta::assert_snapshot!("char_wrap_mode", output);
}

#[test]
fn test_none_wrap_mode() {
    let csv_input = load_fixture("long_text.csv");
    let output = run_csvpretty_in_pty(&csv_input, 80, &["--wrap", "none"])
        .expect("Failed to run csvpretty");

    insta::assert_snapshot!("none_wrap_mode", output);
}

#[test]
fn test_default_wrap_mode() {
    let csv_input = load_fixture("simple.csv");
    let output = run_csvpretty_in_pty(&csv_input, 80, &[])
        .expect("Failed to run csvpretty");

    insta::assert_snapshot!("default_wrap_mode", output);
}

#[test]
fn test_word_wrap_simple_data() {
    let csv_input = load_fixture("simple.csv");
    let output = run_csvpretty_in_pty(&csv_input, 80, &["--wrap", "word"])
        .expect("Failed to run csvpretty");

    insta::assert_snapshot!("word_wrap_simple_data", output);
}

#[test]
fn test_char_wrap_simple_data() {
    let csv_input = load_fixture("simple.csv");
    let output = run_csvpretty_in_pty(&csv_input, 80, &["--wrap", "char"])
        .expect("Failed to run csvpretty");

    insta::assert_snapshot!("char_wrap_simple_data", output);
}

#[test]
fn test_none_wrap_simple_data() {
    let csv_input = load_fixture("simple.csv");
    let output = run_csvpretty_in_pty(&csv_input, 80, &["--wrap", "none"])
        .expect("Failed to run csvpretty");

    insta::assert_snapshot!("none_wrap_simple_data", output);
}
