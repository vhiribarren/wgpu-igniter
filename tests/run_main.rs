use std::time::Duration;

use assert_cmd::Command;
use cargo_metadata::{MetadataCommand, TargetKind};

const TIMEOUT_DURATION: Duration = Duration::from_secs(5);

#[test]
fn main_doesnt_panic() -> Result<(), anyhow::Error> {
    Command::cargo_bin(env!("CARGO_PKG_NAME"))?
        .env("HEADLESS", "true")
        .timeout(TIMEOUT_DURATION)
        .assert()
        .success();
    Ok(())
}

#[test]
fn examples_dont_pannic() -> Result<(), anyhow::Error> {
    let metadata = MetadataCommand::new()
        .exec()
        .expect("Failed to fetch metadata");
    for package in metadata.packages {
        if !metadata.workspace_members.contains(&package.id) {
            continue;
        }
        for target in package.targets {
            if target.is_kind(TargetKind::Example) {
                let example_under_test = escargot::CargoBuild::new()
                    .example(target.name)
                    .run()
                    .unwrap();
                Command::from_std(example_under_test.command())
                    .env("HEADLESS", "true")
                    .timeout(TIMEOUT_DURATION)
                    .assert()
                    .success();
            }
        }
    }
    Ok(())
}
