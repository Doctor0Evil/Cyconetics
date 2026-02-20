//! Validators and policy checks for decisions, neuro-consent, and ecological protection.
//! These functions enforce the structural rules that make the system safe.

use crate::types::*;
use serde::{Serialize, Deserialize};
use std::fmt;

/// Neuro-consent violation types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NeuroConsentViolation {
    /// Non-host entity received modulation or closed-loop signal
    InsectInvasion,
    /// Plant electrical activity overloaded
    PlantElectricalOverload,
    /// Host did not provide explicit consent
    HostConsentMissing,
    /// BFC power level excessive for zone
    BFCPowerExcessive,
    /// Observation-based intervention (forbidden)
    NonInvasionViolated,
}

impl fmt::Display for NeuroConsentViolation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NeuroConsentViolation::InsectInvasion => write!(f, "Insect neural invasion attempt"),
            NeuroConsentViolation::PlantElectricalOverload => write!(f, "Plant electrical activity overload"),
            NeuroConsentViolation::HostConsentMissing => write!(f, "Host consent missing"),
            NeuroConsentViolation::BFCPowerExcessive => write!(f, "BFC power level excessive"),
            NeuroConsentViolation::NonInvasionViolated => write!(f, "Non-invasion principle violated"),
        }
    }
}

/// Validate a BFC broadcast proposal for neuro-consent and ecological safety
pub fn validate_bfc_broadcast(proposal: &BFCBroadcastProposal) -> Result<(), NeuroConsentViolation> {
    // Zero-touch rule: non-host entities must have NO modulation or closed-loop
    for entity_type in &proposal.target_entities {
        match entity_type {
            NeuroEntityType::HostBrain => {
                // Host can consent to anything; verify consent exists
                let has_host_consent = proposal.consent_records.iter()
                    .any(|r| r.entity == NeuroEntityType::HostBrain);
                if !has_host_consent {
                    return Err(NeuroConsentViolation::HostConsentMissing);
                }
            },
            NeuroEntityType::NonhostNervousSystem => {
                // Insects, worms, etc. — ZERO TOUCH
                if proposal.consent_records.iter().any(|r| 
                    r.entity == NeuroEntityType::NonhostNervousSystem 
                    && (r.can_modulate || r.can_closedloop)
                ) {
                    return Err(NeuroConsentViolation::InsectInvasion);
                }
            },
            NeuroEntityType::PlantElectricalActivity => {
                // Passive telemetry only; power limited
                if proposal.consent_records.iter().any(|r|
                    r.entity == NeuroEntityType::PlantElectricalActivity
                    && r.bfc_power_level > 30
                ) {
                    return Err(NeuroConsentViolation::PlantElectricalOverload);
                }
            },
        }
    }

    // Non-invasive rule: all BFC broadcasts must be observation-free
    if !proposal.zero_observation {
        return Err(NeuroConsentViolation::NonInvasionViolated);
    }

    Ok(())
}

/// Validate that RoH complies with a zone's ceiling
pub fn validate_roh_complies_with_zone(
    roh: f32,
    zone_ceiling: f32,
) -> Result<(), String> {
    if roh >= zone_ceiling {
        Err(format!("RoH {:.3} exceeds zone ceiling {:.3}", roh, zone_ceiling))
    } else {
        Ok(())
    }
}

/// Validate a complete decision record
pub fn validate_decision_record(record: &DecisionRecord) -> Result<(), String> {
    // 1. Check that pre_roh and post_roh are monotone for Reject/Approve
    match record.decision {
        DecisionKind::Reject => {
            // Reject should not increase RoH
            if record.post_roh > record.pre_roh {
                return Err("Reject should not increase RoH".to_string());
            }
        },
        DecisionKind::Approve | DecisionKind::Authorize => {
            // Should stay below 0.3
            if record.post_roh >= 0.3 {
                return Err(format!("Post-RoH {:.3} >= 0.3 ceiling", record.post_roh));
            }
        },
        _ => {}
    }

    // 2. Verify ledger key matches record fields
    if record.ledger_key.host_did != record.host_did
        || record.ledger_key.upgrade_id != record.upgrade_id
        || record.ledger_key.evolution_id != record.evolution_id
    {
        return Err("Ledger key does not match record fields".to_string());
    }

    // 3. Check KSR band is in valid ranges
    if record.ksr_band.knowledge < 0xD0 {
        return Err(format!("Knowledge score too low: {:02X}", record.ksr_band.knowledge));
    }
    if record.ksr_band.risk > 0x30 {
        return Err(format!("Risk score too high: {:02X}", record.ksr_band.risk));
    }

    Ok(())
}

/// Validate neuro-consent for a specific entity type
pub fn validate_neuro_entity_consent(
    entity: NeuroEntityType,
    consent_records: &[NeuroConsentRecord],
) -> Result<(), NeuroConsentViolation> {
    match entity {
        NeuroEntityType::HostBrain => {
            // Host can consent to anything
            let _ = consent_records.iter()
                .find(|r| r.entity == NeuroEntityType::HostBrain)
                .ok_or(NeuroConsentViolation::HostConsentMissing)?;
            Ok(())
        },
        NeuroEntityType::NonhostNervousSystem => {
            // Non-host: check that modulation/closedloop are false
            let record = consent_records.iter()
                .find(|r| r.entity == NeuroEntityType::NonhostNervousSystem)
                .ok_or(NeuroConsentViolation::HostConsentMissing)?;
            
            if record.can_modulate || record.can_closedloop {
                Err(NeuroConsentViolation::InsectInvasion)
            } else {
                Ok(())
            }
        },
        NeuroEntityType::PlantElectricalActivity => {
            // Plant: passive only
            let record = consent_records.iter()
                .find(|r| r.entity == NeuroEntityType::PlantElectricalActivity)
                .ok_or(NeuroConsentViolation::HostConsentMissing)?;
            
            if record.bfc_power_level > 30 {
                Err(NeuroConsentViolation::PlantElectricalOverload)
            } else {
                Ok(())
            }
        },
    }
}

/// Metadata validator: ensure BCI/HCI/XR metadata is present and sane
pub fn validate_upgrade_metadata(record: &DecisionRecord) -> Result<(), String> {
    // BCI upgrades must have non-zero brain_specs
    if record.brain_specs.max_roh <= 0.0 {
        return Err("Brain specs max_roh must be > 0".to_string());
    }

    // Host budget must have non-negative reserves
    if record.host_budget.auet_budget < 0.0 {
        return Err("Host budget cannot be negative".to_string());
    }

    if record.host_budget.blood_tokens_reserved < 0.0 {
        return Err("Blood token reserve cannot be negative".to_string());
    }

    Ok(())
}

/// Electrocardiogram (ECG) safety check: ensure heart rate is in safe range
pub fn validate_ecg_safe(heart_rate: f32, systolic_bp: f32, diastolic_bp: f32) -> Result<(), String> {
    if heart_rate < 40.0 || heart_rate > 140.0 {
        return Err(format!("Heart rate {} out of safe range [40, 140] BPM", heart_rate));
    }

    if systolic_bp < 80.0 || systolic_bp > 160.0 {
        return Err(format!("Systolic BP {} out of safe range", systolic_bp));
    }

    if diastolic_bp < 50.0 || diastolic_bp > 100.0 {
        return Err(format!("Diastolic BP {} out of safe range", diastolic_bp));
    }

    Ok(())
}

/// Glucose safety check
pub fn validate_glucose_safe(glucose: f32) -> Result<(), String> {
    if glucose < 70.0 {
        Err("Hypoglycemia: glucose < 70 mg/dL".to_string())
    } else if glucose > 300.0 {
        Err("Hyperglycemia: glucose > 300 mg/dL".to_string())
    } else {
        Ok(())
    }
}

/// Temperature safety check
pub fn validate_temperature_safe(core_temp: f32) -> Result<(), String> {
    if core_temp < 36.0 || core_temp > 39.0 {
        Err(format!("Core temp {} out of safe range [36.0, 39.0]°C", core_temp))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_bfc_broadcast_zero_touch() {
        let mut proposal = BFCBroadcastProposal {
            host_did: "test".to_string(),
            bfc_id: "bfc-001".to_string(),
            power_level: 50,
            target_entities: vec![NeuroEntityType::NonhostNervousSystem],
            consent_records: vec![NeuroConsentRecord::zero_touch(
                NeuroEntityType::NonhostNervousSystem,
                "zone-test".to_string(),
            )],
            zone: "zone-test".to_string(),
            zero_observation: true,
        };
        assert!(validate_bfc_broadcast(&proposal).is_ok());

        // Now try with modulation (should fail)
        proposal.consent_records[0].can_modulate = true;
        assert!(validate_bfc_broadcast(&proposal).is_err());
    }

    #[test]
    fn test_validate_ecg_safe() {
        assert!(validate_ecg_safe(70.0, 120.0, 80.0).is_ok());
        assert!(validate_ecg_safe(30.0, 120.0, 80.0).is_err()); // Too low
        assert!(validate_ecg_safe(150.0, 120.0, 80.0).is_err()); // Too high
    }

    #[test]
    fn test_validate_glucose_safe() {
        assert!(validate_glucose_safe(100.0).is_ok());
        assert!(validate_glucose_safe(60.0).is_err());
        assert!(validate_glucose_safe(350.0).is_err());
    }
}
