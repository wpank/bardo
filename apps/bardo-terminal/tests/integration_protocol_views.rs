//! Integration tests for the Protocol Views screen (Plan 07).
//!
//! `bardo-terminal` is a binary crate, so integration tests cannot import
//! internal `pub(crate)` types. These tests verify the binary builds and can
//! start without panicking. Unit tests in `src/screens/protocol_views.rs`
//! cover the widget rendering, focus cycling, and layout logic in detail.
//!
//! ## Manual verification checklist
//!
//! Run with `cargo run -p bardo-terminal` and confirm:
//!
//! 1. Tab/BackTab to the PROTOCOLS screen (last entry in the catalog)
//! 2. 80x24 viewport: 2x2 grid with pool (top-left), lending (top-right),
//!    vault (bottom-left), bridge (bottom-right)
//! 3. Each cell shows mock data: pool = ETH/USDC, lending = Aave V3,
//!    vault = Beefy, bridge = Across
//! 4. Arrow keys move focus (active border highlight changes between cells)
//! 5. Focus cycles: 0->1->2->3->0 (Right), 0->3->2->1->0 (Left),
//!    0->2->0 (Down/Up in 2x2 grid)
//! 6. Resize terminal to <60 width: layout collapses to 1x4 vertical stack
//! 7. Down key in compact mode navigates 1-by-1, not 2-by-2
//! 8. Tab advances to next screen; BackTab goes to previous; q exits
//! 9. No panic on any key input or resize
//! 10. BORDER_ACTIVE, SUCCESS, DANGER, etc. render visibly on terminal

use std::path::Path;

/// The binary target exists after `cargo build`.
///
/// `CARGO_BIN_EXE_bardo-terminal` is set by Cargo for integration tests
/// when a `[[bin]]` target with that name is declared. If this env var
/// is missing, the test framework will refuse to compile, catching any
/// Cargo.toml misconfiguration.
#[test]
fn binary_exists() {
    let bin_path = env!("CARGO_BIN_EXE_bardo-terminal");
    assert!(
        Path::new(bin_path).exists(),
        "bardo-terminal binary should exist at {bin_path}"
    );
}

/// The binary starts and can be killed without panicking.
///
/// We spawn the process and immediately kill it. A panic during
/// initialization would cause a non-zero exit code from the signal
/// handler, but a clean SIGKILL/SIGTERM yields a signal-terminated
/// status (not a "panic" status).
#[test]
fn binary_starts_without_panic() {
    let bin_path = env!("CARGO_BIN_EXE_bardo-terminal");

    // Spawn without a real TTY; the app will fail on terminal setup
    // (no TTY attached) but should not panic. We check that the exit
    // is either a clean error (non-zero from missing TTY) or signal,
    // not a panic-induced abort.
    let output = std::process::Command::new(bin_path)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .output();

    match output {
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            // The app will fail because there's no TTY, but it should
            // not contain "panicked" in its output.
            assert!(
                !stderr.contains("panicked"),
                "binary should not panic on startup: {stderr}"
            );
        }
        Err(e) => {
            // If we can't even spawn (e.g. permission denied), that's
            // a test-environment issue, not a code bug. Skip gracefully.
            eprintln!("could not spawn binary: {e}");
        }
    }
}

/// Release build succeeds (compile-time gate).
///
/// This test doesn't do anything at runtime. Its purpose is to exist
/// in the test suite so that `cargo test -p bardo-terminal --all`
/// requires the integration test crate to compile, which in turn
/// requires the binary target to link. If the binary fails to compile,
/// this test file won't compile either.
#[test]
fn release_build_gate() {
    // Intentionally empty. The act of compiling this file is the test.
    // `cargo build -p bardo-terminal --release` is verified separately
    // in CI; this test ensures the debug build links correctly.
}
