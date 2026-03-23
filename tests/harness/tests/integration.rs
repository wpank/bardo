//! Integration tests for the bardo-test-harness.

use bardo_test_harness::{BardoTestHarness, HarnessConfig, TestGolemInstance};

#[test]
fn test_golem_instance_lifecycle() {
    let mut golem = TestGolemInstance::new();
    assert!(golem.is_alive());
    assert_eq!(golem.phase, "Thriving");
    assert_eq!(golem.tick_count, 0);

    golem.tick();
    golem.tick();
    assert_eq!(golem.tick_count, 2);
}

#[test]
fn test_harness_config_defaults() {
    let config = HarnessConfig::default();
    assert!(!config.enable_terminal);
    assert!(!config.enable_golem);
    assert!(!config.enable_mirage);
    assert_eq!(config.terminal_rpc_port, 19100);
    assert_eq!(config.mirage_port, 18545);
}

#[tokio::test]
async fn test_harness_spawn_golem_only() {
    let harness = BardoTestHarness::spawn(HarnessConfig {
        enable_golem: true,
        ..Default::default()
    })
    .await
    .expect("harness spawns with golem only");

    assert!(harness.golem.is_some());
    assert!(harness.terminal.is_none());
    assert!(harness.mirage.is_none());

    let health = harness.health_check().await;
    assert!(health.golem_alive);
    assert_eq!(health.golem_phase.as_deref(), Some("Thriving"));
    assert!(!health.terminal_responding);
    assert!(!health.mirage_ready);

    harness.teardown().await.expect("teardown succeeds");
}

#[tokio::test]
async fn test_harness_spawn_nothing() {
    let harness = BardoTestHarness::spawn(HarnessConfig::default())
        .await
        .expect("empty harness spawns");

    let health = harness.health_check().await;
    assert!(!health.golem_alive);
    assert!(!health.terminal_responding);
    assert!(!health.mirage_ready);

    harness.teardown().await.expect("teardown succeeds");
}

#[cfg(not(feature = "mirage"))]
#[tokio::test]
async fn test_mirage_spawn_fails_without_feature() {
    let result = BardoTestHarness::spawn(HarnessConfig {
        enable_mirage: true,
        ..Default::default()
    })
    .await;

    let err = match result {
        Err(e) => e.to_string(),
        Ok(_) => panic!("expected error when mirage feature is disabled"),
    };
    assert!(err.contains("mirage feature not enabled"), "got: {err}");
}
