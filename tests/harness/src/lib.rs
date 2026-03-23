//! `bardo-test-harness` - end-to-end test harness for the Bardo workspace.
//!
//! Provides `BardoTestHarness` which can spawn and manage terminal, golem, and
//! mirage instances for integration testing.

#![deny(unsafe_code)]
#![warn(missing_docs)]
#![allow(
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::missing_errors_doc
)]

use std::time::Duration;

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// HealthReport
// ---------------------------------------------------------------------------

/// Aggregate health status across managed components.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HealthReport {
    /// Whether the terminal RPC server responded to a health check.
    pub terminal_responding: bool,
    /// Status string returned by `terminal.health`, if any.
    pub terminal_status: Option<String>,
    /// Whether the golem wrapper is alive.
    pub golem_alive: bool,
    /// Current golem lifecycle phase, if available.
    pub golem_phase: Option<String>,
    /// Whether mirage is ready.
    pub mirage_ready: bool,
}

// ---------------------------------------------------------------------------
// HarnessConfig
// ---------------------------------------------------------------------------

/// Configuration for spawning test components.
#[derive(Debug, Clone)]
pub struct HarnessConfig {
    /// Spawn and manage a terminal instance.
    pub enable_terminal: bool,
    /// JSON-RPC port for the headless terminal.
    pub terminal_rpc_port: u16,
    /// Spawn and manage a golem wrapper.
    pub enable_golem: bool,
    /// Spawn and manage a mirage instance (requires `mirage` feature).
    pub enable_mirage: bool,
    /// Port for the mirage JSON-RPC server.
    pub mirage_port: u16,
}

impl Default for HarnessConfig {
    fn default() -> Self {
        Self {
            enable_terminal: false,
            terminal_rpc_port: 19100,
            enable_golem: false,
            enable_mirage: false,
            mirage_port: 18545,
        }
    }
}

// ---------------------------------------------------------------------------
// TerminalProbe
// ---------------------------------------------------------------------------

/// Manages a headless `bardo-terminal` child process and provides RPC access.
pub struct TerminalProbe {
    child: tokio::process::Child,
    port: u16,
    client: reqwest::Client,
}

impl TerminalProbe {
    /// Spawns `bardo-terminal --headless --rpc-port {port}` and waits for readiness.
    pub async fn spawn(port: u16) -> anyhow::Result<Self> {
        let child = tokio::process::Command::new("cargo")
            .args([
                "run",
                "-p",
                "bardo-terminal",
                "--",
                "--headless",
                "--rpc-port",
                &port.to_string(),
            ])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()?;

        let client = reqwest::Client::new();

        for _ in 0..30 {
            tokio::time::sleep(Duration::from_millis(500)).await;
            if let Ok(resp) =
                Self::rpc_call_inner(&client, port, "terminal.health", serde_json::Value::Null)
                    .await
            {
                if resp.get("status").and_then(|s| s.as_str()) == Some("ok") {
                    return Ok(Self {
                        child,
                        port,
                        client,
                    });
                }
            }
        }

        anyhow::bail!("terminal failed to become ready within 15s")
    }

    /// Checks terminal health.
    pub async fn health(&self) -> anyhow::Result<serde_json::Value> {
        self.rpc_call("terminal.health", serde_json::Value::Null)
            .await
    }

    /// Takes a snapshot of the current terminal state.
    pub async fn snapshot(&self) -> anyhow::Result<serde_json::Value> {
        self.rpc_call("terminal.snapshot", serde_json::Value::Null)
            .await
    }

    /// Injects an action into the terminal.
    pub async fn inject_action(&self, action: &str) -> anyhow::Result<serde_json::Value> {
        self.rpc_call("terminal.action", serde_json::json!({ "action": action }))
            .await
    }

    /// Sends a shutdown request to the terminal.
    pub async fn shutdown(&mut self) -> anyhow::Result<()> {
        let _ = self
            .rpc_call("terminal.shutdown", serde_json::Value::Null)
            .await;
        let _ = tokio::time::timeout(Duration::from_secs(5), self.child.wait()).await;
        if self.child.try_wait()?.is_none() {
            self.child.kill().await?;
        }
        Ok(())
    }

    async fn rpc_call(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        Self::rpc_call_inner(&self.client, self.port, method, params).await
    }

    async fn rpc_call_inner(
        client: &reqwest::Client,
        port: u16,
        method: &str,
        params: serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        let url = format!("http://127.0.0.1:{port}/");
        let resp = client
            .post(&url)
            .json(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": method,
                "params": params,
            }))
            .send()
            .await?;
        let body: serde_json::Value = resp.json().await?;
        body.get("result")
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("missing result in RPC response"))
    }
}

// ---------------------------------------------------------------------------
// TestGolemInstance
// ---------------------------------------------------------------------------

/// Lightweight wrapper around `GolemConfig` for test scenarios.
///
/// No `TestGolem` exists in golem-core yet, so this tracks config and
/// simulated state for use in harness-level assertions.
pub struct TestGolemInstance {
    /// The golem configuration used for this instance.
    pub config: golem_core::GolemConfig,
    /// Simulated lifecycle phase label.
    pub phase: String,
    /// Number of simulated ticks.
    pub tick_count: u64,
}

impl TestGolemInstance {
    /// Creates a test golem from the default config.
    pub fn new() -> Self {
        Self {
            config: golem_core::GolemConfig::default(),
            phase: "Thriving".to_owned(),
            tick_count: 0,
        }
    }

    /// Simulates a tick.
    pub fn tick(&mut self) {
        self.tick_count += 1;
    }

    /// Returns whether the instance is alive (always true for this stub).
    pub fn is_alive(&self) -> bool {
        true
    }
}

impl Default for TestGolemInstance {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// MirageInstance
// ---------------------------------------------------------------------------

/// Wrapper around a mirage-rs test instance.
///
/// When the `mirage` feature is enabled, this holds a real `MirageTestInstance`.
/// Otherwise it's a stub that reports "mirage not available" in health checks.
pub struct MirageInstance {
    #[cfg(feature = "mirage")]
    inner: mirage_rs::MirageTestInstance,
    #[cfg(not(feature = "mirage"))]
    _private: (),
}

impl MirageInstance {
    /// Spawns a mirage test instance (requires `mirage` feature).
    #[cfg(feature = "mirage")]
    pub async fn spawn(port: u16) -> anyhow::Result<Self> {
        let inner = mirage_rs::spawn_mirage_test_instance(None, Some(port))
            .await
            .map_err(|e| anyhow::anyhow!("mirage spawn failed: {e}"))?;
        Ok(Self { inner })
    }

    /// Stub spawn when mirage feature is disabled.
    #[cfg(not(feature = "mirage"))]
    pub async fn spawn(_port: u16) -> anyhow::Result<Self> {
        anyhow::bail!("mirage feature not enabled")
    }

    /// Returns whether mirage is ready.
    pub fn is_ready(&self) -> bool {
        #[cfg(feature = "mirage")]
        {
            true // if we got past spawn, it waited for readiness
        }
        #[cfg(not(feature = "mirage"))]
        {
            false
        }
    }

    /// Shuts down the mirage instance.
    pub async fn shutdown(&mut self) -> anyhow::Result<()> {
        #[cfg(feature = "mirage")]
        {
            self.inner
                .shutdown()
                .await
                .map_err(|e| anyhow::anyhow!("mirage shutdown failed: {e}"))?;
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// BardoTestHarness
// ---------------------------------------------------------------------------

/// Top-level test harness that orchestrates component lifecycle.
pub struct BardoTestHarness {
    /// Mirage sidecar instance, if spawned.
    pub mirage: Option<MirageInstance>,
    /// Headless terminal probe, if spawned.
    pub terminal: Option<TerminalProbe>,
    /// Golem test instance, if created.
    pub golem: Option<TestGolemInstance>,
}

impl BardoTestHarness {
    /// Spawns all enabled components according to `config`.
    pub async fn spawn(config: HarnessConfig) -> anyhow::Result<Self> {
        let mirage = if config.enable_mirage {
            Some(MirageInstance::spawn(config.mirage_port).await?)
        } else {
            None
        };

        let terminal = if config.enable_terminal {
            Some(TerminalProbe::spawn(config.terminal_rpc_port).await?)
        } else {
            None
        };

        let golem = if config.enable_golem {
            Some(TestGolemInstance::new())
        } else {
            None
        };

        Ok(Self {
            mirage,
            terminal,
            golem,
        })
    }

    /// Checks health across all managed components.
    pub async fn health_check(&self) -> HealthReport {
        let mut report = HealthReport::default();

        if let Some(probe) = &self.terminal {
            match probe.health().await {
                Ok(value) => {
                    report.terminal_responding = true;
                    report.terminal_status = value
                        .get("status")
                        .and_then(|s| s.as_str())
                        .map(String::from);
                }
                Err(_) => {
                    report.terminal_responding = false;
                }
            }
        }

        if let Some(golem) = &self.golem {
            report.golem_alive = golem.is_alive();
            report.golem_phase = Some(golem.phase.clone());
        }

        if let Some(mirage) = &self.mirage {
            report.mirage_ready = mirage.is_ready();
        }

        report
    }

    /// Tears down all managed components in reverse order.
    pub async fn teardown(mut self) -> anyhow::Result<()> {
        if let Some(mut terminal) = self.terminal.take() {
            terminal.shutdown().await?;
        }
        if let Some(mut mirage) = self.mirage.take() {
            mirage.shutdown().await?;
        }
        // golem is in-process, nothing to tear down
        Ok(())
    }
}
