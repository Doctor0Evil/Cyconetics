//! Example: Phoenix BCI upgrade decision using cyconetics-decision-grammar
//! 
//! This example demonstrates:
//! 1. Type-safe decision records
//! 2. RoH ≤ 0.3 enforcement
//! 3. Role-based decision making
//! 4. Evidence bundle creation
//! 5. ALN shard binding
//! 6. CI sidecar checks

use cyconetics_decision_grammar::*;
use chrono::Utc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Cyconetics Decision Grammar Example: Phoenix BCI Upgrade ===\n");

    // 1. Create host state (augmented citizen in Phoenix zone)
    let host_did = "bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7";
    let mut host_state = RoHGuardedHostState {
        host_budget: HostBudget {
            auet_budget: 1000.0,
            blood_tokens_reserved: 50.0,
            kcal_today: 2000.0,
            glucose_band: 2,       // Normal
            hydration_index: 0.75,
            fat_reserve_index: 0.60,
            protein_reserve_index: 0.55,
        },
        brain_specs: BrainSpecs::default_phoenix_baseline(),
        bci_snapshot: BciHostSnapshot {
            eeg_corridor_state: "nominal".to_string(),
            plasticity_used_percent: 0.35,
            neural_rope_anchor_integrity: 0.92,
            active_neural_roi: 4,
        },
        current_roh: 0.15,
    };

    println!("Host State:");
    println!("  DID: {}", host_did);
    println!("  Current RoH: {:.3}", host_state.current_roh);
    println!("  AU.ET Budget: {:.1}", host_state.host_budget.auet_budget);
    println!("  Glucose Band: {} (0=hypo, 1=low, 2=normal, 3=high, 4=hyper)", host_state.host_budget.glucose_band);
    println!("  Hydration Index: {:.2}", host_state.host_budget.hydration_index);
    println!();

    // 2. Create an upgrade descriptor (BCI enhancement)
    let upgrade_desc = UpgradeDescriptor {
        upgrade_id: "bci-enhancement-001".to_string(),
        upgrade_class: UpgradeClass::BCI,
        estimated_roh_delta: 0.08,  // Adding 8% risk
        requires_host_veto: false,
        blood_token_cost: 10.0,
    };

    println!("Upgrade Descriptor:");
    println!("  ID: {}", upgrade_desc.upgrade_id);
    println!("  Class: {:?}", upgrade_desc.upgrade_class);
    println!("  Estimated RoH Delta: {:.3}", upgrade_desc.estimated_roh_delta);
    println!("  Blood Token Cost: {:.1}", upgrade_desc.blood_token_cost);
    println!();

    // 3. Evaluate upgrade safety
    let decision = evaluate_upgrade(&host_state, &upgrade_desc);
    println!("Upgrade Evaluation: {:?}", decision);
    println!();

    // 4. Create evidence bundle (biomarkers at time of decision)
    let mut evidence = EvidenceBundle::new("zone-phoenix-west".to_string());
    evidence.biomarkers = EvidenceBiomarkers {
        il6_level: 8.5,         // Normal (~5–10)
        crp_level: 1.2,         // Normal (<3)
        cortisol_level: 280.0,  // Normal (morning ~200–400)
        heart_rate: 72.0,
        systolic_bp: 118.0,
        diastolic_bp: 76.0,
        core_temperature: 37.1,
        glucose_blood: 105.0,
        lactate_level: 1.5,     // Normal (~0.5–2.0)
        oxygen_saturation: 98.5,
    };

    println!("Evidence Bundle (Biomarkers):");
    println!("  IL-6: {:.1} ng/mL", evidence.biomarkers.il6_level);
    println!("  Heart Rate: {:.0} BPM", evidence.biomarkers.heart_rate);
    println!("  Blood Pressure: {:.0}/{:.0} mmHg", 
             evidence.biomarkers.systolic_bp, evidence.biomarkers.diastolic_bp);
    println!("  Glucose: {:.0} mg/dL", evidence.biomarkers.glucose_blood);
    println!("  Core Temp: {:.1}°C", evidence.biomarkers.core_temperature);
    println!();

    // 5. Run CI sidecar checks
    let sidecar = CISidecarm::new();
    let evidence_check = sidecar.check_evidence_bundle(&evidence);
    println!("CI Sidecar Evidence Check: {}", evidence_check);
    println!();

    // 6. Compute RoH from biomarma vector
    let biokarma = BioKarmaRiskVector {
        metabolic_risk: 0.08,      // Glucose normal but slight uptake
        hemodynamic_risk: 0.06,    // HR elevated slightly
        thermal_risk: 0.03,
        cognitive_risk: 0.10,      // BCI session induces cognitive load
        psych_risk: 0.05,
    };
    let biokarma_roh = roh_from_biokarma(&biokarma);
    println!("BioKarma RoH Calculation:");
    println!("  Metabolic Risk: {:.3}", biokarma.metabolic_risk);
    println!("  Hemodynamic Risk: {:.3}", biokarma.hemodynamic_risk);
    println!("  Cognitive Risk: {:.3}", biokarma.cognitive_risk);
    println!("  Psych Risk: {:.3}", biokarma.psych_risk);
    println!("  Composite RoH: {:.3}", biokarma_roh);
    println!();

    // 7. Create decision record
    let predicted_roh = host_state.current_roh + upgrade_desc.estimated_roh_delta;
    
    // Try to create RoHBound<30>
    let _roh_bound = try_roh_bound_30(predicted_roh)
        .expect("RoH must be < 0.3 for authorization");

    let decision_record = DecisionRecord {
        host_did: host_did.to_string(),
        upgrade_id: upgrade_desc.upgrade_id.clone(),
        evolution_id: "evo-bci-001".to_string(),
        decision: DecisionKind::Authorize,
        pre_roh: host_state.current_roh,
        post_roh: predicted_roh,
        brain_specs: host_state.brain_specs.clone(),
        host_budget: host_state.host_budget.clone(),
        bci_snapshot: host_state.bci_snapshot.clone(),
        ksr_band: KsrBand {
            knowledge: 0xE0,
            social: 0x78,
            risk: 0x26,
        },
        ledger_key: DecisionLedgerKey {
            host_did: host_did.to_string(),
            upgrade_id: upgrade_desc.upgrade_id.clone(),
            evolution_id: "evo-bci-001".to_string(),
        },
        timestamp: Utc::now().timestamp(),
    };

    println!("Decision Record Created:");
    println!("  Decision Verb: {}", decision_record.decision);
    println!("  Pre-RoH: {:.3}", decision_record.pre_roh);
    println!("  Post-RoH: {:.3}", decision_record.post_roh);
    println!("  KSR Band: K={:02X}, S={:02X}, R={:02X}",
             decision_record.ksr_band.knowledge,
             decision_record.ksr_band.social,
             decision_record.ksr_band.risk);
    println!();

    // 8. Check decision record via CI sidecar
    let record_check = sidecar.check_decision_record(&decision_record);
    println!("CI Sidecar Record Check: {}", record_check);
    println!();

    // 9. Create ledger entry and commit to ALN shard
    let mut ledger_entry = DecisionLedgerEntry::new(
        decision_record.ledger_key.clone(),
        decision_record.decision,
        decision_record.ksr_band,
        evidence,
    );
    
    // Add blood token coupling
    ledger_entry.blood_coupling = Some(BloodSpendProof {
        blood_tokens_spent: upgrade_desc.blood_token_cost,
        blood_reserve_remaining: host_state.host_budget.blood_tokens_reserved - upgrade_desc.blood_token_cost,
        homeostasis_protected: true,
        evidence_hash: "evidence_hash_placeholder".to_string(),
    });

    println!("Decision Ledger Entry Created:");
    println!("  Ledger Key: {}", ledger_entry.key);
    println!("  Entry Hash: {}", ledger_entry.ledger_entry_hash);
    if let Some(blood) = &ledger_entry.blood_coupling {
        println!("  Blood Tokens Spent: {:.1}", blood.blood_tokens_spent);
        println!("  Blood Reserve Remaining: {:.1}", blood.blood_reserve_remaining);
    }
    println!();

    // 10. Commit to ALN shard
    let mut decision_shard = DecisionLedgerShard::new();
    let shard_entry = ledger_entry.to_aln_shard();
    decision_shard.append(shard_entry);

    println!("ALN Shard Committed:");
    println!("  Shard Name: {}", decision_shard.shard_name);
    println!("  Entries: {}", decision_shard.entries.len());
    println!("  Committed Hash: {}", decision_shard.last_committed_hash);
    println!();

    // 11. Verify neuro-consent for BFC broadcast (ecological safety example)
    let bfc_proposal = BFCBroadcastProposal {
        host_did: host_did.to_string(),
        bfc_id: "bfc-phoenix-001".to_string(),
        power_level: 20,
        target_entities: vec![NeuroEntityType::HostBrain, NeuroEntityType::NonhostNervousSystem],
        consent_records: vec![
            NeuroConsentRecord::host_full_consent("zone-phoenix-west".to_string(), 50),
            NeuroConsentRecord::zero_touch(NeuroEntityType::NonhostNervousSystem, "zone-phoenix-west".to_string()),
        ],
        zone: "zone-phoenix-west".to_string(),
        zero_observation: true,
    };

    println!("BFC Broadcast Proposal:");
    println!("  BFC ID: {}", bfc_proposal.bfc_id);
    println!("  Power Level: {}", bfc_proposal.power_level);
    println!("  Target Entities: {} entities", bfc_proposal.target_entities.len());
    println!("  Zero Observation: {}", bfc_proposal.zero_observation);

    let neuro_check = sidecar.check_bfc_broadcast(&bfc_proposal);
    println!("  CI Neuro-Consent Check: {}", neuro_check);
    println!();

    // 12. Create neuro-consent registry entry
    let mut neuro_shard = NeurorightsConsentShard::new();
    neuro_shard.register(NeuroConsentEntry {
        entity_id: "insect-pollinator-001".to_string(),
        entity_type: "NonhostNervousSystem".to_string(),
        consent_level: 1,  // Passive telemetry only
        zones_allowed: vec!["zone-phoenix-west".to_string()],
        consent_timestamp: Utc::now().timestamp(),
        expires_at: None,
        audit_hash: "consent_audit_hash".to_string(),
    });

    println!("Neurorights Consent Registry:");
    println!("  Shard Name: {}", neuro_shard.shard_name);
    println!("  Consents: {}", neuro_shard.consents.len());
    println!("  Registry Hash: {}", neuro_shard.last_committed_hash);
    println!();

    println!("✓ Example completed successfully!");
    println!("\nSummary:");
    println!("  • Decision authorized with RoH < 0.3");
    println!("  • Biomarkers validated (all in safe range)");
    println!("  • Blood tokens coupled and homeostasis protected");
    println!("  • Ecological consent enforced (zero-touch for non-host entities)");
    println!("  • All evidence committed to immutable ALN shards");
    println!("  • Full audit trail available for incident-driven tightening");

    Ok(())
}
