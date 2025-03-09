use assert_cmd::Command;
use std::time::Duration;

const TIMEOUT_DURATION: Duration = Duration::from_secs(5);

#[test]
fn main_doesnt_panic() {
    Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .expect("Failed to find binary")
        .env("HEADLESS", "true")
        .timeout(TIMEOUT_DURATION)
        .assert()
        .success();
}
