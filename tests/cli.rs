use std::path::PathBuf;
use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_payments-engine"))
}

fn testdata(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("testdata")
        .join(name)
}

fn stdout_csv(file: &str) -> String {
    let output = bin()
        .arg(testdata(file))
        .output()
        .expect("failed to run payments-engine");

    assert!(
        output.status.success(),
        "engine failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8(output.stdout).expect("stdout must be utf-8")
}

#[test]
fn sample_matches_spec() {
    let stdout = stdout_csv("sample.csv");

    assert_eq!(
        stdout,
        "\
client,available,held,total,locked
1,1.5000,0.0000,1.5000,false
2,2.0000,0.0000,2.0000,false
"
    );
}

#[test]
fn whitespace_is_accepted() {
    let stdout = stdout_csv("whitespace.csv");

    assert_eq!(
        stdout,
        "\
client,available,held,total,locked
1,1.5000,0.0000,1.5000,false
2,2.0000,0.0000,2.0000,false
"
    );
}

#[test]
fn chargeback_locks_the_account() {
    let stdout = stdout_csv("chargeback.csv");

    assert_eq!(
        stdout,
        "\
client,available,held,total,locked
1,0.0000,0.0000,0.0000,true
"
    );
}

#[test]
fn missing_path_exits_with_error() {
    let output = bin().output().expect("failed to run payments-engine");
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("missing input file path"));
}

#[test]
fn malformed_input_exits_with_error() {
    let output = bin()
        .arg(testdata("missing_amount.csv"))
        .output()
        .expect("failed to run payments-engine");

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("missing a required amount"));
}
