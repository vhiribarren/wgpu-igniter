/*
MIT License

Copyright (c) 2025 Vincent Hiribarren

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/

use cargo_metadata::MetadataCommand;
use std::fs::File;
use std::io::Write;
use std::path::Path;

const TEST_FILE_PATH: &str = "tests/run_examples_generated.rs";
const TEST_FILE_HEADER: &str = r#"
// File generated by build.rs, do not modify directly

/*
MIT License

Copyright (c) 2025 Vincent Hiribarren

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/

// File generated by build.rs, do not modify directly

use assert_cmd::Command;
use escargot;
use std::time::Duration;

const TIMEOUT_DURATION: Duration = Duration::from_secs(10);

macro_rules! generate_test_case {
    ($funcname:ident, $testname:tt) => {
        #[test]
        fn $funcname() {
            let example_under_test = escargot::CargoBuild::new()
                .example($testname)
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
"#;

fn main() {
    generate_example_tests();
    println!("cargo:rerun-if-changed=build.rs");
}

fn generate_example_tests() {
    let test_file_path = Path::new(TEST_FILE_PATH);
    let mut test_file = File::create(test_file_path)
        .unwrap_or_else(|_| panic!("Failed to create {TEST_FILE_PATH}"));

    writeln!(test_file, "{TEST_FILE_HEADER}").unwrap();

    let metadata = MetadataCommand::new()
        .exec()
        .expect("Failed to fetch metadata");
    let workspace_members = metadata.workspace_members;
    let packages = metadata.packages;

    for package in packages {
        if !workspace_members.contains(&package.id) {
            continue;
        }
        for target in package.targets {
            if !target.is_example() {
                continue;
            }
            let example_name = target.name;
            let func_name = format!("example_{}_doesnt_panic", example_name.replace("-", "_"));
            writeln!(
                test_file,
                "generate_test_case!({func_name}, \"{example_name}\");"
            )
            .unwrap();
        }
    }
}
