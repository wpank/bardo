//! Typed event bus and replay fabric.

use std::{
    collections::VecDeque,
    convert::TryFrom,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::{SystemTime, UNIX_EPOCH},
};

use parking_lot::RwLock;
use serde::Serialize;
use tokio::sync::broadcast;

const REPLAY_CAPACITY: usize = 10_000;

fn current_ts_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| {
            u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
        })
}

/// Subsystem that produced an event.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Subsystem {
    /// Heartbeat subsystem.
    Heartbeat,
    /// Perception subsystem.
    Perception,
    /// Daimon subsystem.
    Daimon,
    /// Mortality subsystem.
    Mortality,
    /// Grimoire subsystem.
    Grimoire,
    /// Dreams subsystem.
    Dreams,
    /// Context subsystem.
    Context,
    /// Inference subsystem.
    Inference,
    /// Tools subsystem.
    Tools,
    /// Risk subsystem.
    Risk,
    /// Coordination subsystem.
    Coordination,
    /// Lifecycle subsystem.
    Lifecycle,
    /// Engagement subsystem.
    Engagement,
    /// Session subsystem.
    Session,
    /// Creature subsystem.
    Creature,
    /// System subsystem.
    System,
}

/// Sequenced event emitted by the runtime.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GolemEvent {
    /// Monotonic event sequence.
    pub seq: u64,
    /// Milliseconds since Unix epoch.
    pub ts_millis: u64,
    /// Heartbeat tick when the event was emitted.
    pub tick: u64,
    /// Originating subsystem.
    pub subsystem: Subsystem,
    /// Typed event payload.
    pub payload: EventPayload,
}

/// Typed event payloads emitted by the runtime.
#[allow(missing_docs)]
#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum EventPayload {
    HeartbeatTick {
        tick: u64,
        tier: String,
        pe: f64,
        threshold: f64,
    },
    HeartbeatComplete {
        tick: u64,
        duration_ms: u64,
        actions_taken: u32,
    },
    MarketObservation {
        regime: String,
        anomalies: Vec<String>,
        probe_count: u32,
    },
    DaimonAppraisal {
        pleasure: f64,
        arousal: f64,
        dominance: f64,
        emotion: String,
        markers_fired: u32,
    },
    SomaticMarkerFired {
        situation: String,
        valence: f64,
        source: String,
    },
    VitalityUpdate {
        economic: f64,
        epistemic: f64,
        stochastic: f64,
        composite: f64,
        phase: String,
    },
    PhaseTransition {
        from: String,
        to: String,
        cause: String,
    },
    DeathClockAlarm {
        clock: String,
        value: f64,
        threshold: f64,
    },
    InsightPromoted {
        id: String,
        category: String,
        confidence: f64,
    },
    HeuristicEvolved {
        id: String,
        description: String,
    },
    KnowledgeDecayed {
        count: u32,
        reason: String,
    },
    WarningActivated {
        id: String,
        severity: String,
    },
    ScarRecorded {
        source_golem: String,
        warning: String,
    },
    CausalLinkUpdated {
        from: String,
        to: String,
        strength: f64,
    },
    CuratorCycleComplete {
        entries_validated: u32,
        entries_pruned: u32,
        entries_promoted: u32,
    },
    DreamStart {
        trigger: String,
    },
    DreamPhaseTransition {
        from: String,
        to: String,
    },
    DreamReplay {
        episode_id: String,
        utility: f64,
    },
    DreamCounterfactual {
        hypothesis: String,
        outcome: String,
    },
    DreamConsolidation {
        playbook_edits: u32,
        insights_generated: u32,
    },
    DreamComplete {
        cycles_completed: u32,
    },
    MicroConsolidation {
        entries_processed: u32,
        depotentiation_count: u32,
    },
    ContextAssembled {
        total_tokens: u32,
        categories: Vec<(String, u32)>,
    },
    ContextPolicySelfTuned {
        revision: u32,
        adjustments: Vec<String>,
    },
    InferenceStart {
        model: String,
        tier: String,
        input_tokens: u32,
    },
    InferenceToken {
        token: String,
    },
    InferenceComplete {
        output_tokens: u32,
        cost: f64,
        latency_ms: u64,
    },
    ToolStart {
        tool: String,
        category: String,
    },
    ToolProgress {
        tool: String,
        step: String,
        pct: f32,
    },
    ToolComplete {
        tool: String,
        success: bool,
        duration_ms: u64,
    },
    PermitCreated {
        id: String,
        action: String,
        value_limit: String,
    },
    PermitStateChange {
        id: String,
        from: String,
        to: String,
    },
    RiskAssessment {
        layer: String,
        result: String,
    },
    CladeSyncComplete {
        entries_sent: u32,
        entries_received: u32,
    },
    BloomUpdated {
        domains: Vec<String>,
    },
    PheromoneDeposited {
        layer: String,
        domain: String,
        intensity: f64,
    },
    PheromoneRead {
        threats: u32,
        opportunities: u32,
        wisdom: u32,
    },
    BloodstainReceived {
        source_generation: u32,
        warning: String,
    },
    CausalEdgePublished {
        from_var: String,
        to_var: String,
    },
    LifecycleTransition {
        from: String,
        to: String,
    },
    DeathInitiated {
        cause: String,
    },
    SuccessorSpawned {
        successor_id: String,
    },
    HealthStatus {
        process: String,
        ok: bool,
        message: String,
    },
    AchievementUnlocked {
        id: String,
        description: String,
    },
    MilestoneReached {
        name: String,
        value: f64,
    },
    UserMessage {
        content: String,
        session_id: String,
    },
    GolemResponseChunk {
        content: String,
        is_final: bool,
    },
    CreatureFormEvolved {
        from_form: u8,
        to_form: u8,
    },
    ExpressionUpdated {
        expression: String,
    },
    ParticleEffectTriggered {
        effect_type: String,
    },
    ShutdownInitiated {
        phase: String,
    },
    ResourceWarning {
        resource: String,
        utilization: f64,
    },
}

/// Non-blocking broadcast fabric with bounded replay.
pub struct EventFabric {
    tx: broadcast::Sender<GolemEvent>,
    buffer: Arc<RwLock<VecDeque<GolemEvent>>>,
    seq: AtomicU64,
}

impl EventFabric {
    /// Creates a new event fabric.
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity.max(1));
        Self {
            tx,
            buffer: Arc::new(RwLock::new(VecDeque::with_capacity(REPLAY_CAPACITY))),
            seq: AtomicU64::new(0),
        }
    }

    /// Emits an event to live subscribers and the replay buffer.
    pub fn emit(&self, subsystem: Subsystem, tick: u64, payload: EventPayload) {
        let event = GolemEvent {
            seq: self.seq.fetch_add(1, Ordering::Relaxed),
            ts_millis: current_ts_millis(),
            tick,
            subsystem,
            payload,
        };

        {
            let mut buffer = self.buffer.write();
            if buffer.len() >= REPLAY_CAPACITY {
                buffer.pop_front();
            }
            buffer.push_back(event.clone());
        }

        let _ = self.tx.send(event);
    }

    /// Subscribes to the live broadcast bus.
    pub fn subscribe(&self) -> broadcast::Receiver<GolemEvent> {
        self.tx.subscribe()
    }

    /// Replays buffered events starting at `after_seq` inclusively.
    pub fn replay_from(&self, after_seq: u64) -> Vec<GolemEvent> {
        self.buffer
            .read()
            .iter()
            .filter(|event| event.seq >= after_seq)
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::{EventFabric, EventPayload, Subsystem};

    #[tokio::test]
    async fn event_fabric_broadcast() {
        let fabric = EventFabric::new(100);
        let mut rx = fabric.subscribe();

        fabric.emit(
            Subsystem::Heartbeat,
            1,
            EventPayload::HeartbeatTick {
                tick: 1,
                tier: "T0".to_owned(),
                pe: 0.1,
                threshold: 0.3,
            },
        );

        let event = rx.recv().await.expect("broadcast event");
        assert_eq!(event.seq, 0);
        assert_eq!(event.tick, 1);
    }

    #[test]
    fn event_fabric_replay() {
        let fabric = EventFabric::new(4);
        fabric.emit(
            Subsystem::System,
            1,
            EventPayload::ShutdownInitiated {
                phase: "flush".to_owned(),
            },
        );
        fabric.emit(
            Subsystem::System,
            2,
            EventPayload::ResourceWarning {
                resource: "memory".to_owned(),
                utilization: 0.9,
            },
        );

        let replay = fabric.replay_from(0);
        assert_eq!(replay.len(), 2);
        assert_eq!(replay[0].seq, 0);
        assert_eq!(replay[1].seq, 1);
    }

    #[test]
    fn event_fabric_ring_buffer_cap() {
        let fabric = EventFabric::new(1);
        for tick in 0..10_050 {
            fabric.emit(
                Subsystem::System,
                tick,
                EventPayload::ShutdownInitiated {
                    phase: format!("phase-{tick}"),
                },
            );
        }

        let replay = fabric.replay_from(0);
        assert_eq!(replay.len(), 10_000);
        assert_eq!(replay.first().expect("first").seq, 50);
        assert_eq!(replay.last().expect("last").seq, 10_049);
    }
}
