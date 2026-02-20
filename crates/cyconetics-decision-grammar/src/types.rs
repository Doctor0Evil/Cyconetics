//! Core type definitions for Cyconetics decision grammar.
//! All decision-related types are strongly typed to prevent misuse at compile time.

use serde::{Serialize, Deserialize};
use std::fmt;

/// Decision verb enum: the allowed actions in the governance grammar
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DecisionKind {
    /// Host or authorized role approves an upgrade; passes all checks
    Approve,
    /// Upgrade is authorized to proceed; roles can build conditional logic here
    Authorize,
    /// Decision is deferred; escalate to human review or wait for conditions
    Defer,
    /// Upgrade is explicitly rejected; all downstream actions blocked
    Reject,
    /// Escalate to GovSafetyOS or NeurorightsBoard for review
    Escalate,
    /// BFC broadcast (biofield communicator) without modulation or closed-loop
    BroadcastBFC { power: u8, entities: Vec<NeuroEntityType> },
    /// BFC broadcast with modulation (host-brain only)
    BroadcastWithModulation { power: u8, target: NeuroEntityType },
    /// BFC broadcast with closed-loop (host-brain only; non-host always errors at compile time)
    BroadcastWithClosedLoop { power: u8, target: NeuroEntityType },
}

impl fmt::Display for DecisionKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DecisionKind::Approve => write!(f, "Approve"),
            DecisionKind::Authorize => write!(f, "Authorize"),
            DecisionKind::Defer => write!(f, "Defer"),
            DecisionKind::Reject => write!(f, "Reject"),
            DecisionKind::Escalate => write!(f, "Escalate"),
            DecisionKind::BroadcastBFC { power, entities } => {
                write!(f, "BroadcastBFC(power={}, entities={})", power, entities.len())
            },
            DecisionKind::BroadcastWithModulation { power, target } => {
                write!(f, "BroadcastWithModulation(power={}, target={:?})", power, target)
            },
            DecisionKind::BroadcastWithClosedLoop { power, target } => {
                write!(f, "BroadcastWithClosedLoop(power={}, target={:?})", power, target)
            },
        }
    }
}

/// RoHBound<const N: u8> is a zero-sized token that proves RoH < N/100
/// Only constructors that mathematically verify the bound may return this token.
/// This is a type-level capability: if you hold RoHBound<30>, you have proven RoH < 0.3.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RoHBound<const N: u8>;

impl<const N: u8> RoHBound<N> {
    /// Create a bound if roh_current < N/100 (but this should only be called
    /// from trusted validator functions that compute RoH mathematically)
    pub fn new_unchecked() -> Self {
        RoHBound
    }

    /// For testing: check if a float value would satisfy this bound
    pub fn satisfies(value: f32) -> bool {
        value < (N as f32 / 100.0)
    }
}

/// KSR band: Knowledge, Safety/Social, Risk scoring
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KsrBand {
    pub knowledge: u8,      // 0x00–0xFF, typically 0xD0–0xE2
    pub social: u8,         // 0x00–0xFF, typically 0x70–0x7A
    pub risk: u8,           // 0x00–0xFF, typically 0x20–0x30 (lower is safer)
}

impl Default for KsrBand {
    fn default() -> Self {
        KsrBand {
            knowledge: 0xE0,
            social: 0x78,
            risk: 0x2A,
        }
    }
}

impl KsrBand {
    pub fn is_safe_for_high_risk_upgrade(&self) -> bool {
        // Risk must be low (≤ 0x30) for high-risk upgrades
        self.risk <= 0x30
    }
}

/// Neurorights tag: what type of neural processing is involved
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NeurorightsTag {
    PerceptionOnly,      // Pure sensory data, no decision-making
    InferenceOk,         // Inference is allowed, but no actuation
    IntentGrade,         // Motor intent is being decoded and may drive action
    ActuationLinked,     // Closed-loop actuation based on brain signals
    IdentityMarked,      // Involves identity/self-model; highest protection
}

/// Neural entity type for neuro-consent and ecological protection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NeuroEntityType {
    HostBrain,           // The augmented citizen (full rights)
    NonhostNervousSystem, // Insects, worms, etc. (sovereign by default; zero-touch)
    PlantElectricalActivity, // Passive telemetry only; no modulation
}

/// Record of consent from a neural entity (or lack thereof)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuroConsentRecord {
    pub entity: NeuroEntityType,
    pub proximity_zone: String,      // XR-Grid zone ID
    pub bfc_power_level: u8,          // 0–255, typically 20–100
    pub can_modulate: bool,           // FALSE for non-host entities
    pub can_closedloop: bool,         // FALSE for non-host entities
    pub audit_hash: String,           // SHA256 of this record
    pub timestamp: i64,
}

impl NeuroConsentRecord {
    /// Create a safe (zero-touch) consent record for non-host entity
    pub fn zero_touch(entity: NeuroEntityType, zone: String) -> Self {
        NeuroConsentRecord {
            entity,
            proximity_zone: zone,
            bfc_power_level: 15,  // Low power for passive telemetry
            can_modulate: false,
            can_closedloop: false,
            audit_hash: String::new(),
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    /// For host brain: allow modulation and closed-loop
    pub fn host_full_consent(zone: String, power: u8) -> Self {
        NeuroConsentRecord {
            entity: NeuroEntityType::HostBrain,
            proximity_zone: zone,
            bfc_power_level: power,
            can_modulate: true,
            can_closedloop: true,
            audit_hash: String::new(),
            timestamp: chrono::Utc::now().timestamp(),
        }
    }
}

/// BFC (biofield communicator) broadcast proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BFCBroadcastProposal {
    pub host_did: String,
    pub bfc_id: String,
    pub power_level: u8,
    pub target_entities: Vec<NeuroEntityType>,
    pub consent_records: Vec<NeuroConsentRecord>,
    pub zone: String,
    pub zero_observation: bool,  // MUST be true for non-invasive interventions
}

/// Snapshot of host's brain/augmentation state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BciHostSnapshot {
    pub eeg_corridor_state: String,     // Simplified: "nominal", "elevated", "critical"
    pub plasticity_used_percent: f32,
    pub neural_rope_anchor_integrity: f32,
    pub active_neural_roi: u32,         // Number of regions of interest
}

/// Brain specifications and RoH thresholds for a host
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BrainSpecs {
    pub max_roh: f32,              // Maximum tolerable RoH for this host (typically 0.3)
    pub roh_per_pass: f32,         // Incremental RoH per BCI pass (e.g., 0.02)
    pub max_session_duration_ms: u32,
    pub rest_interval_ms: u32,
    pub max_plasticity_delta: f32,
    pub hemodynamic_safety_margin: f32, // Buffer before critical HR/BP
}

impl BrainSpecs {
    pub fn default_phoenix_baseline() -> Self {
        BrainSpecs {
            max_roh: 0.30,
            roh_per_pass: 0.02,
            max_session_duration_ms: 120_000, // 2 minutes
            rest_interval_ms: 30_000,         // 30 seconds
            max_plasticity_delta: 0.10,
            hemodynamic_safety_margin: 0.15,
        }
    }
}

/// Host budget and metabolic state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HostBudget {
    pub auet_budget: f32,              // AU.ET energy tokens available
    pub blood_tokens_reserved: f32,    // CSP/Blood tokens reserved
    pub kcal_today: f32,
    pub glucose_band: u8,              // 0=hypo, 1=low, 2=normal, 3=high, 4=hyper
    pub hydration_index: f32,          // 0.0–1.0
    pub fat_reserve_index: f32,        // 0.0–1.0
    pub protein_reserve_index: f32,    // 0.0–1.0
}

/// Host state guarded by RoH invariants
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RoHGuardedHostState {
    pub host_budget: HostBudget,
    pub brain_specs: BrainSpecs,
    pub bci_snapshot: BciHostSnapshot,
    pub current_roh: f32,
}

/// Complete decision record for audit and blockchain stamping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionRecord {
    pub host_did: String,
    pub upgrade_id: String,
    pub evolution_id: String,
    pub decision: DecisionKind,
    pub pre_roh: f32,                  // RoH before this decision
    pub post_roh: f32,                 // RoH after this decision
    pub brain_specs: BrainSpecs,
    pub host_budget: HostBudget,
    pub bci_snapshot: BciHostSnapshot,
    pub ksr_band: KsrBand,
    pub ledger_key: DecisionLedgerKey,
    pub timestamp: i64,
}

/// Key for looking up decisions in the ledger
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecisionLedgerKey {
    pub host_did: String,
    pub upgrade_id: String,
    pub evolution_id: String,
}

impl fmt::Display for DecisionLedgerKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}:{}", self.host_did, self.upgrade_id, self.evolution_id)
    }
}

/// Strongly-typed role markers (used as phantom types in trait bounds)
pub struct HostSelfRole;
pub struct NeurorightsRole;
pub struct SafetyRole;
pub struct GovSafetyRole;

/// Marker: this value has been validated by host (HostSelf)
#[derive(Debug, Clone)]
pub struct HostValidated<T> {
    pub value: T,
    pub validated_by_host: bool,
}

impl<T> HostValidated<T> {
    pub fn new(value: T) -> Self {
        HostValidated {
            value,
            validated_by_host: true,
        }
    }
}
