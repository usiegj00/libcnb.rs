//! Integration tests using libcnb-test.
//!
//! All integration tests are skipped by default (using the `ignore` attribute),
//! since performing builds is slow. To run the tests use: `cargo test -- --ignored`

// Enable Clippy lints that are disabled by default.
// https://rust-lang.github.io/rust-clippy/stable/index.html
#![warn(clippy::pedantic)]

use indoc::indoc;
use libcnb_test::{
    assert_contains, assert_empty, assert_not_contains, BuildpackReference, PackResult, TestConfig,
    TestRunner,
};
use std::path::PathBuf;
use std::time::Duration;
use std::{env, fs, thread};

#[test]
#[ignore]
fn basic_build() {
    TestRunner::default().run_test(
        TestConfig::new("heroku/builder:22", "test-fixtures/procfile").buildpacks(vec![
            BuildpackReference::Other(String::from("heroku/procfile")),
        ]),
        |context| {
            assert_empty!(context.pack_stderr);
            assert_contains!(
                context.pack_stdout,
                indoc! {"
                    [Discovering process types]
                    Procfile declares types -> web, worker, echo-args
                "}
            );
        },
    );
}

#[test]
#[ignore]
fn rebuild() {
    TestRunner::default().run_test(
        TestConfig::new("heroku/builder:22", "test-fixtures/procfile").buildpacks(vec![
            BuildpackReference::Other(String::from("heroku/procfile")),
        ]),
        |context| {
            assert_empty!(context.pack_stderr);
            assert_not_contains!(context.pack_stdout, "Reusing layer");

            let config = context.config.clone();
            context.run_test(config, |rebuild_context| {
                assert_empty!(rebuild_context.pack_stderr);
                assert_contains!(rebuild_context.pack_stdout, "Reusing layer");
            });
        },
    );
}

#[test]
#[ignore]
fn starting_containers() {
    TestRunner::default().run_test(
        TestConfig::new("heroku/builder:22", "test-fixtures/procfile").buildpacks(vec![
            BuildpackReference::Other(String::from("heroku/procfile")),
        ]),
        |context| {
            context
                .prepare_container()
                .start_with_default_process(|container| {
                    // Give the server time to boot up.
                    // TODO: Make requests to the server using a client that retries, and fetch logs after
                    // that, instead of sleeping. This will also allow us to test `expose_port()` etc.
                    thread::sleep(Duration::from_secs(2));

                    let log_output_until_now = container.logs_now();
                    assert_empty!(log_output_until_now.stderr);
                    assert_contains!(
                        log_output_until_now.stdout,
                        "Serving HTTP on 0.0.0.0 port 8000"
                    );

                    let exec_log_output = container.shell_exec("ps");
                    assert_empty!(exec_log_output.stderr);
                    assert_contains!(exec_log_output.stdout, "python3");
                });

            // TODO: Add a test for `start_with_default_process_args` based on the above,
            // that passes "5000" as the argument. This isn't possible at the moment,
            // since `lifecycle` seems to have a bug around passing arguments to
            // non-direct processes (and Procfile creates processes as non-direct).

            context
                .prepare_container()
                .start_with_process(String::from("worker"), |container| {
                    let all_log_output = container.logs_wait();
                    assert_empty!(all_log_output.stderr);
                    assert_eq!(all_log_output.stdout, "this is the worker process!\n");
                });

            context.prepare_container().start_with_process_args(
                String::from("echo-args"),
                ["Hello!"],
                |container| {
                    let all_log_output = container.logs_wait();
                    assert_empty!(all_log_output.stderr);
                    assert_eq!(all_log_output.stdout, "Hello!\n");
                },
            );

            context
                .prepare_container()
                .env("TEST_VAR", "TEST_VALUE")
                .start_with_shell_command("env", |container| {
                    let all_log_output = container.logs_wait();
                    assert_empty!(all_log_output.stderr);
                    assert_contains!(all_log_output.stdout, "TEST_VAR=TEST_VALUE");
                });
        },
    );
}

#[test]
#[ignore]
#[should_panic(
    expected = "Could not package current crate as buildpack: BuildBinariesError(ConfigError(NoBinTargetsFound))"
)]
fn buildpack_packaging_failure() {
    TestRunner::default().run_test(
        TestConfig::new("libcnb/invalid-builder", "test-fixtures/empty"),
        |_| {},
    );
}

#[test]
#[ignore]
#[should_panic(expected = "pack command unexpectedly failed with exit-code 1!

pack stdout:


pack stderr:
ERROR: failed to build: failed to fetch builder image 'index.docker.io/libcnb/invalid-builder:latest'")]
fn unexpected_pack_failure() {
    TestRunner::default().run_test(
        TestConfig::new("libcnb/invalid-builder", "test-fixtures/empty").buildpacks(Vec::new()),
        |_| {},
    );
}

#[test]
#[ignore]
#[should_panic(expected = "pack command unexpectedly succeeded with exit-code 0!

pack stdout:
")]
fn unexpected_pack_success() {
    TestRunner::default().run_test(
        TestConfig::new("heroku/builder:22", "test-fixtures/procfile")
            .buildpacks(vec![BuildpackReference::Other(String::from(
                "heroku/procfile",
            ))])
            .expected_pack_result(PackResult::Failure),
        |_| {},
    );
}

#[test]
#[ignore]
fn expected_pack_failure() {
    TestRunner::default().run_test(
        TestConfig::new("libcnb/invalid-builder", "test-fixtures/empty")
            .buildpacks(Vec::new())
            .expected_pack_result(PackResult::Failure),
        |context| {
            assert_empty!(context.pack_stdout);
            assert_contains!(
                context.pack_stderr,
                "ERROR: failed to build: failed to fetch builder image 'index.docker.io/libcnb/invalid-builder:latest'"
            );
        },
    );
}

#[test]
#[ignore]
#[should_panic(
    expected = "Could not package current crate as buildpack: BuildBinariesError(ConfigError(NoBinTargetsFound))"
)]
fn expected_pack_failure_still_panics_for_non_pack_failure() {
    TestRunner::default().run_test(
        TestConfig::new("libcnb/invalid-builder", "test-fixtures/empty")
            .expected_pack_result(PackResult::Failure),
        |_| {},
    );
}

#[test]
#[ignore]
fn app_dir_preprocessor() {
    TestRunner::default().run_test(
        TestConfig::new("heroku/builder:22", "test-fixtures/nested_dirs")
            .buildpacks(vec![BuildpackReference::Other(String::from(
                "heroku/procfile",
            ))])
            .app_dir_preprocessor(|app_dir| {
                assert!(app_dir.join("file1.txt").exists());
                fs::write(app_dir.join("Procfile"), "list-files: find . | sort").unwrap();
            }),
        |context| {
            context
                .prepare_container()
                .start_with_default_process(|container| {
                    let log_output = container.logs_wait();
                    assert_contains!(
                        log_output.stdout,
                        indoc! {"
                            ./Procfile
                            ./file1.txt
                            ./subdir1
                            ./subdir1/file2.txt
                            ./subdir1/subdir2
                            ./subdir1/subdir2/subdir3
                            ./subdir1/subdir2/subdir3/file3.txt
                        "}
                    );
                });
        },
    );

    // Check that the original fixture was left untouched.
    let fixture_dir = env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap()
        .join("test-fixtures/nested_dirs");
    assert!(fixture_dir.join("file1.txt").exists());
    assert!(!fixture_dir.join("Procfile").exists());
}

#[test]
#[ignore]
fn app_dir_absolute_path() {
    let absolute_app_dir = env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap()
        .join("test-fixtures/procfile")
        .canonicalize()
        .unwrap();

    TestRunner::default().run_test(
        TestConfig::new("heroku/builder:22", absolute_app_dir).buildpacks(vec![
            BuildpackReference::Other(String::from("heroku/procfile")),
        ]),
        |_| {},
    );
}

#[test]
#[ignore]
// TODO: We should validate `app_dir` explicitly before passing to pack:
// https://github.com/heroku/libcnb.rs/issues/448
#[should_panic(expected = "pack stderr:
ERROR: failed to build: invalid app path")]
fn app_dir_invalid_path() {
    TestRunner::default().run_test(
        TestConfig::new("heroku/builder:22", "test-fixtures/non-existent-fixture")
            .buildpacks(Vec::new()),
        |_| {},
    );
}

#[test]
#[ignore]
// TODO: We should validate `app_dir` explicitly before passing to app_dir_preprocessor:
// https://github.com/heroku/libcnb.rs/issues/448
#[should_panic(
    expected = "Could not copy app to temporary location: CopyAppError(Error { kind: NotFound"
)]
fn app_dir_invalid_path_with_preprocessor() {
    TestRunner::default().run_test(
        TestConfig::new("heroku/builder:22", "test-fixtures/non-existent-fixture")
            .buildpacks(Vec::new())
            .app_dir_preprocessor(|_| {}),
        |_| {},
    );
}
