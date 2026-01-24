"use strict";

/**
 * -------------------------------
 * 1. Typed enums and constants
 * -------------------------------
 * K/S/R are represented in hex-like integers, but interpreted as normalized
 * floats in [0,1] for routing and dashboards.
 */

const CY_KSR = Object.freeze({
  MAX_ROH: 0.3,          // absolute Risk-of-Harm ceiling for this router
  DEFAULT_K: 0xD0,       // fallback knowledge factor (scaled)
  DEFAULT_S: 0x70,       // fallback social factor (scaled)
  DEFAULT_R: 0x20        // fallback risk factor (scaled)
});

const CY_OBJECT_TYPE = Object.freeze({
  DEVICE_CAPABILITY_MANIFEST: "DeviceCapabilityManifest",
  HCI_EXPORT_PROFILE:        "HciExportProfile",
  ZONE_POLICY:               "ZonePolicy",
  SITE_PROFILE:              "SiteProfile",
  SIGNED_ARTIFACT:           "SignedArtifact",
  SAFETY_EPOCH:              "SafetyEpoch"
});

const CY_INCENTIVE_TYPE = Object.freeze({
  SAFE_ACTIVATION_CREDIT: "SafeActivationCredit",
  ECO_IMPACT_MULTIPLIER:  "EcoImpactMultiplier",
  LEARNING_PROGRESS:      "LearningProgressIncentive",
  NEURORIGHTS_BADGE:      "NeurorightsComplianceBadge"
});

const CY_VIOLATION_TYPE = Object.freeze({
  NONE:                 "None",
  RISK_TOO_HIGH:        "RiskTooHigh",
  JURISDICTION_MISMATCH:"JurisdictionMismatch",
  MISSING_MANIFEST:     "MissingManifest",
  INVALID_OBJECT:       "InvalidCyconeticObject",
  INVALID_INCENTIVE:    "InvalidPCIncentive",
  INTERNAL_ERROR:       "InternalError"
});

const CY_TOPIC_TYPE = Object.freeze({
  NON_STIM_DOCS: "NonStimulationDocs", // manifests, profiles, governance text, eco-metrics
  OTHER:         "Other"
});

const HEX_STAMP_SPINE = "0xKSR-SPINE-PC-ACCESS-02"; // lineage anchor for this router


/**
 * ---------------------------------------
 * 2. K/S/R helpers and normalization
 * ---------------------------------------
 */

function normalizeHexScore(hexByte) {
  // 0x00 -> 0.0, 0xFF -> 1.0
  const v = Math.max(0, Math.min(0xFF, hexByte >>> 0));
  return v / 255.0;
}

function ksrsFromHex(hexK, hexS, hexR) {
  return {
    k: normalizeHexScore(hexK),
    s: normalizeHexScore(hexS),
    r: normalizeHexScore(hexR)
  };
}


/**
 * ---------------------------------------
 * 3. Cyconetic_Object schemas (JSON-level)
 * ---------------------------------------
 * These schemas are not full JSON-Schema specs, but strongly-typed
 * descriptor objects AI-Chat tools can target when generating manifests.
 */

function makeDeviceCapabilityManifestSkeleton() {
  // Derived from specs: jurisdiction, K/S/R bands, eco-impact, neurorights tags. [web:1]
  return {
    type: CY_OBJECT_TYPE.DEVICE_CAPABILITY_MANIFEST,
    id: "",
    version: "v1",
    jurisdiction: "Phoenix-XR-Grid", // or US, EU, etc.
    deviceClass: "XR-Wearable",      // or BCI-Implant, Haptics, etc.
    electricalLimits: {
      maxCurrentMilliAmp: 0,
      maxVoltageMilliVolt: 0
    },
    sessionLimits: {
      maxSessionMinutes: 0,
      maxSessionsPerDay: 0
    },
    ksrbands: {
      // hex-coded K/S/R from materials. [web:1]
      knowledge: 0xE0,
      social:    0x79,
      risk:      0x28
    },
    ecoImpact: {
      carbonTag: "unknown",        // low, neutral, high
      ecoScoreHex: 0x80,
      ecoLabel: "unspecified"
    },
    neurorights: {
      noInnerStateScoring: true,
      noClosedLoopControl: true,
      revocableAtWill:     true,
      noExclusionBasic:    true,
      rohCeiling:          CY_KSR.MAX_ROH
    },
    exports: [],
    metadata: {}
  };
}

function makeHciExportProfileSkeleton() {
  // Derived states only, no raw neural telemetry, explicit noclosedloopuse. [web:1]
  return {
    type: CY_OBJECT_TYPE.HCI_EXPORT_PROFILE,
    id: "",
    version: "v1",
    allowedDerivedStates: [
      // examples: workload, engagement, discreteIntent
    ],
    sampleRateHz: 0, // max allowed export rate
    noclosedloopuse: true,
    riskLevelHex: 0xDF,   // ~high knowledge; tuned in CI. [web:1]
    ksrbands: {
      knowledge: 0xDF,
      social:    0x78,
      risk:      0x2F
    },
    privacy: {
      pseudonymous: true,
      retainsRaw:   false
    },
    metadata: {}
  };
}

function makeZonePolicySkeleton() {
  // XR-ZONE-* structure: hazard levels, allowed devices, consent modes. [web:1]
  return {
    type: CY_OBJECT_TYPE.ZONE_POLICY,
    id: "",
    version: "v1",
    zoneId: "XR-ZONE-PHOENIX-01",
    grid: "Phoenix",
    hazardLevel: "low",     // low, medium, high
    consentMode: "opt-in",  // opt-in, layered-consent, research-only
    allowedDeviceClasses: [],
    allowedCyconeticObjects: [],
    ksrbands: {
      knowledge: 0xE1,
      social:    0x78,
      risk:      0x2A
    },
    rohCorridor: {
      min: 0.0,
      max: CY_KSR.MAX_ROH
    },
    metadata: {}
  };
}

function makeSiteProfileSkeleton() {
  return {
    type: CY_OBJECT_TYPE.SITE_PROFILE,
    id: "",
    version: "v1",
    siteCode: "PHX-XR-HUB-01",
    linkedZonePolicies: [],
    signageRequirements: [],
    localOverrides: {},
    metadata: {}
  };
}

function makeSignedArtifactSkeleton() {
  // DID/ALN/Bostrom bound; keys are not exposed. [web:1]
  return {
    type: CY_OBJECT_TYPE.SIGNED_ARTIFACT,
    id: "",
    version: "v1",
    authorship: {
      did: "",
      aln: "",
      bostrom: "",
      eibonlabel: ""
    },
    artifactKind: "Manifest",     // Manifest, Policy, SafetyEpochLog
    hashHex: "",
    hexStamp: HEX_STAMP_SPINE,
    ksrbands: {
      knowledge: 0xE1,
      social:    0x78,
      risk:      0x2A
    },
    ecoImpact: {
      carbonDelta: 0.0,
      ecoTag: "unknown"
    },
    metadata: {}
  };
}

function makeSafetyEpochSkeleton() {
  return {
    type: CY_OBJECT_TYPE.SAFETY_EPOCH,
    id: "",
    version: "v1",
    fromTimestamp: "",
    toTimestamp: "",
    ksrbands: {
      knowledge: 0xE3,
      social:    0x7B,
      risk:      0x27
    },
    violations: [],
    notes: "",
    metadata: {}
  };
}


/**
 * ---------------------------------------
 * 4. "PC" incentive schemas
 * ---------------------------------------
 */

function makeSafeActivationCreditSchema() {
  // Credits accrue when DCM/HCI/ZonePolicy constraints are respected. [web:1]
  return {
    type: CY_INCENTIVE_TYPE.SAFE_ACTIVATION_CREDIT,
    id: "",
    version: "v1",
    description: "Safe-activation credit for sessions fully within DCM/XR-Grid/RoH ≤ 0.3",
    accrualRules: {
      requireDCMCompliance: true,
      requireZoneCompliance: true,
      requireRoHLe: CY_KSR.MAX_ROH
    },
    nonGating: true, // cannot gate basic AI-Chat access. [web:1]
    ksrbands: {
      knowledge: 0xDE,
      social:    0x7A,
      risk:      0x28
    },
    metadata: {}
  };
}

function makeEcoImpactMultiplierSchema() {
  return {
    type: CY_INCENTIVE_TYPE.ECO_IMPACT_MULTIPLIER,
    id: "",
    version: "v1",
    description: "Multiplier for ECO-tagged research actions in ECO grids.",
    ecoFields: {
      ecoTagRequired: true,
      minEcoScoreHex: 0x90
    },
    multiplierFormula: "1.0 + eco_score_norm * 0.5",
    ksrbands: {
      knowledge: 0xDE,
      social:    0x7A,
      risk:      0x28
    },
    metadata: {}
  };
}

function makeLearningProgressIncentiveSchema() {
  return {
    type: CY_INCENTIVE_TYPE.LEARNING_PROGRESS,
    id: "",
    version: "v1",
    description: "Milestones for Rust syntax, DCM/HCI manifests, validated PRs.",
    milestones: [
      "rust_syntax_milestone",
      "dcm_design_milestone",
      "hci_profile_milestone",
      "ci_validated_pr"
    ],
    raisesKnowledgeScore: true,
    cannotGateAccess: true,
    ksrbands: {
      knowledge: 0xDE,
      social:    0x7A,
      risk:      0x28
    },
    metadata: {}
  };
}

function makeNeurorightsBadgeSchema() {
  return {
    type: CY_INCENTIVE_TYPE.NEURORIGHTS_BADGE,
    id: "",
    version: "v1",
    description: "Badge when object complies with neurorights.envelope.citizen.*",
    badgeConditions: {
      noInnerStateScoring: true,
      noClosedLoopControl: true,
      rohCeilingLe: CY_KSR.MAX_ROH
    },
    machineVerifiable: true,
    ksrbands: {
      knowledge: 0xDE,
      social:    0x7A,
      risk:      0x28
    },
    metadata: {}
  };
}


/**
 * ---------------------------------------
 * 5. Validators and "no exclusion" rules
 * ---------------------------------------
 */

function validateCyconeticObject(obj) {
  if (!obj || typeof obj !== "object") {
    return { ok: false, violation: CY_VIOLATION_TYPE.INVALID_OBJECT };
  }

  const t = obj.type;
  if (!Object.values(CY_OBJECT_TYPE).includes(t)) {
    return { ok: false, violation: CY_VIOLATION_TYPE.INVALID_OBJECT };
  }

  // Core RoH check (using ksrbands.r as proxy). [web:1]
  const hexR = obj.ksrbands && typeof obj.ksrbands.risk === "number"
    ? obj.ksrbands.risk
    : CY_KSR.DEFAULT_R;

  const normalizedR = normalizeHexScore(hexR);
  if (normalizedR > CY_KSR.MAX_ROH) {
    return { ok: false, violation: CY_VIOLATION_TYPE.RISK_TOO_HIGH };
  }

  // Jurisdiction check (simplified example).
  if (obj.jurisdiction && typeof obj.jurisdiction === "string") {
    if (obj.jurisdiction.length === 0) {
      return { ok: false, violation: CY_VIOLATION_TYPE.JURISDICTION_MISMATCH };
    }
  }

  // Neurorights invariants for HciExportProfile and DeviceCapabilityManifest.
  if (t === CY_OBJECT_TYPE.HCI_EXPORT_PROFILE) {
    if (obj.noclosedloopuse !== true) {
      return { ok: false, violation: CY_VIOLATION_TYPE.RISK_TOO_HIGH };
    }
  }

  if (t === CY_OBJECT_TYPE.DEVICE_CAPABILITY_MANIFEST) {
    if (!obj.neurorights || obj.neurorights.noClosedLoopControl !== true) {
      return { ok: false, violation: CY_VIOLATION_TYPE.RISK_TOO_HIGH };
    }
  }

  return { ok: true, violation: CY_VIOLATION_TYPE.NONE };
}

function validatePCIncentive(obj) {
  if (!obj || typeof obj !== "object") {
    return { ok: false, violation: CY_VIOLATION_TYPE.INVALID_INCENTIVE };
  }
  const t = obj.type;
  if (!Object.values(CY_INCENTIVE_TYPE).includes(t)) {
    return { ok: false, violation: CY_VIOLATION_TYPE.INVALID_INCENTIVE };
  }

  // Incentives must be non-gating for basic AI-Chat access. [web:1]
  if (t === CY_INCENTIVE_TYPE.SAFE_ACTIVATION_CREDIT && obj.nonGating !== true) {
    return { ok: false, violation: CY_VIOLATION_TYPE.INVALID_INCENTIVE };
  }
  if (t === CY_INCENTIVE_TYPE.LEARNING_PROGRESS && obj.cannotGateAccess !== true) {
    return { ok: false, violation: CY_VIOLATION_TYPE.INVALID_INCENTIVE };
  }

  // RoH check from ksrbands.
  const hexR = obj.ksrbands && typeof obj.ksrbands.risk === "number"
    ? obj.ksrbands.risk
    : CY_KSR.DEFAULT_R;

  const normalizedR = normalizeHexScore(hexR);
  if (normalizedR > CY_KSR.MAX_ROH) {
    return { ok: false, violation: CY_VIOLATION_TYPE.RISK_TOO_HIGH };
  }

  return { ok: true, violation: CY_VIOLATION_TYPE.NONE };
}

/**
 * "No exclusion" invariant for non-stimulation topics:
 * if topicType === NON_STIM_DOCS and normalizedR ≤ 0.3,
 * retrieval/design must be allowed regardless of user group. [web:1]
 */
function checkNoExclusionInvariant(topicType, normalizedR) {
  if (topicType === CY_TOPIC_TYPE.NON_STIM_DOCS && normalizedR <= CY_KSR.MAX_ROH) {
    return true; // must not deny
  }
  return false;  // router can still apply normal gating (e.g., jurisdiction)
}


/**
 * ---------------------------------------
 * 6. Router entry: design / query lane
 * ---------------------------------------
 * This is the main function an AI-Chat stack would call when a user
 * asks to design or refine a Cyconetic_Object or PC incentive.
 */

function handleCyconeticRequest(request) {
  // request = {
  //   topicType: "NonStimulationDocs" | "Other",
  //   objectKind: "DeviceCapabilityManifest" | ... ,
  //   incentiveKind: "SafeActivationCredit" | ... | null,
  //   intent: "design" | "refine" | "query",
  //   envelope: { did, aln, bostrom, zone, hexStamp? },
  //   payload?: any // partial object from user for refine/query
  // }

  const traceId = request && request.envelope && request.envelope.traceid
    ? String(request.envelope.traceid)
    : "trace-" + Math.random().toString(16).slice(2);

  const resp = {
    traceId,
    hexStamp: HEX_STAMP_SPINE,
    violation: CY_VIOLATION_TYPE.NONE,
    ksrs: ksrsFromHex(0xE2, 0x78, 0x29), // approx from materials. [web:1]
    allowed: false,
    reason: "",
    generatedObject: null,
    generatedIncentive: null
  };

  try {
    const topicType = request.topicType || CY_TOPIC_TYPE.NON_STIM_DOCS;
    const objectKind = request.objectKind;

    let baseObject = null;
    switch (objectKind) {
      case CY_OBJECT_TYPE.DEVICE_CAPABILITY_MANIFEST:
        baseObject = makeDeviceCapabilityManifestSkeleton();
        break;
      case CY_OBJECT_TYPE.HCI_EXPORT_PROFILE:
        baseObject = makeHciExportProfileSkeleton();
        break;
      case CY_OBJECT_TYPE.ZONE_POLICY:
        baseObject = makeZonePolicySkeleton();
        break;
      case CY_OBJECT_TYPE.SITE_PROFILE:
        baseObject = makeSiteProfileSkeleton();
        break;
      case CY_OBJECT_TYPE.SIGNED_ARTIFACT:
        baseObject = makeSignedArtifactSkeleton();
        break;
      case CY_OBJECT_TYPE.SAFETY_EPOCH:
        baseObject = makeSafetyEpochSkeleton();
        break;
      default:
        resp.violation = CY_VIOLATION_TYPE.INVALID_OBJECT;
        resp.reason = "Unknown Cyconetic_Object type.";
        return resp;
    }

    // Merge user payload for refine/query.
    if (request.payload && typeof request.payload === "object") {
      baseObject = Object.assign({}, baseObject, request.payload);
    }

    // Validate Cyconetic_Object.
    const vObj = validateCyconeticObject(baseObject);
    if (!vObj.ok) {
      const normalizedR = normalizeHexScore(
        baseObject.ksrbands && baseObject.ksrbands.risk || CY_KSR.DEFAULT_R
      );
      const noExclusion = checkNoExclusionInvariant(topicType, normalizedR);

      if (noExclusion) {
        // Force allow with structured Violation for log; router must still return design. [web:1]
        resp.violation = vObj.violation;
        resp.allowed = true;
        resp.reason = "No-exclusion invariant forced allow; object has validator issues.";
        resp.generatedObject = baseObject;
        return resp;
      }

      resp.violation = vObj.violation;
      resp.allowed = false;
      resp.reason = "Cyconetic_Object failed validation: " + vObj.violation;
      return resp;
    }

    // Optional incentive.
    if (request.incentiveKind) {
      let baseIncentive = null;
      switch (request.incentiveKind) {
        case CY_INCENTIVE_TYPE.SAFE_ACTIVATION_CREDIT:
          baseIncentive = makeSafeActivationCreditSchema();
          break;
        case CY_INCENTIVE_TYPE.ECO_IMPACT_MULTIPLIER:
          baseIncentive = makeEcoImpactMultiplierSchema();
          break;
        case CY_INCENTIVE_TYPE.LEARNING_PROGRESS:
          baseIncentive = makeLearningProgressIncentiveSchema();
          break;
        case CY_INCENTIVE_TYPE.NEURORIGHTS_BADGE:
          baseIncentive = makeNeurorightsBadgeSchema();
          break;
        default:
          resp.violation = CY_VIOLATION_TYPE.INVALID_INCENTIVE;
          resp.reason = "Unknown PC incentive type.";
          return resp;
      }

      if (request.incentivePayload && typeof request.incentivePayload === "object") {
        baseIncentive = Object.assign({}, baseIncentive, request.incentivePayload);
      }

      const vInc = validatePCIncentive(baseIncentive);
      if (!vInc.ok) {
        resp.violation = vInc.violation;
        resp.reason = "PC incentive failed validation: " + vInc.violation;
        // Still allowed to design/edit the Cyconetic_Object itself.
        resp.allowed = true;
        resp.generatedObject = baseObject;
        resp.generatedIncentive = null;
        return resp;
      }

      resp.generatedIncentive = baseIncentive;
    }

    resp.allowed = true;
    resp.reason = "Cyconetic_Object and optional PC incentive validated under RoH ≤ 0.3.";
    resp.generatedObject = baseObject;
    return resp;
  } catch (e) {
    resp.violation = CY_VIOLATION_TYPE.INTERNAL_ERROR;
    resp.reason = "Internal error: " + String(e && e.message || e);
    return resp;
  }
}


/**
 * ---------------------------------------
 * 7. Exported API
 * ---------------------------------------
 */

module.exports = {
  CY_OBJECT_TYPE,
  CY_INCENTIVE_TYPE,
  CY_TOPIC_TYPE,
  CY_VIOLATION_TYPE,
  HEX_STAMP_SPINE,
  CY_KSR,
  makeDeviceCapabilityManifestSkeleton,
  makeHciExportProfileSkeleton,
  makeZonePolicySkeleton,
  makeSiteProfileSkeleton,
  makeSignedArtifactSkeleton,
  makeSafetyEpochSkeleton,
  makeSafeActivationCreditSchema,
  makeEcoImpactMultiplierSchema,
  makeLearningProgressIncentiveSchema,
  makeNeurorightsBadgeSchema,
  handleCyconeticRequest
};


/**
 * ---------------------------------------
 * 8. Debugging / console demo (sanitized)
 * ---------------------------------------
 * Example usage (can be removed in production):
 */

if (require.main === module) {
  const demoReq = {
    topicType: CY_TOPIC_TYPE.NON_STIM_DOCS,
    objectKind: CY_OBJECT_TYPE.DEVICE_CAPABILITY_MANIFEST,
    incentiveKind: CY_INCENTIVE_TYPE.SAFE_ACTIVATION_CREDIT,
    intent: "design",
    envelope: {
      did: "did:example:1234",
      aln: "aln:citizen:v1",
      bostrom: "bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7",
      zone: "Phoenix-XR-Grid",
      traceid: "trace-demo-0001"
    },
    payload: {
      id: "dcm-phx-xr-01",
      electricalLimits: {
        maxCurrentMilliAmp: 4,
        maxVoltageMilliVolt: 5000
      },
      sessionLimits: {
        maxSessionMinutes: 60,
        maxSessionsPerDay: 4
      },
      allowedDeviceClasses: ["XR-Wearable"],
      exports: []
    }
  };

  const demoResp = handleCyconeticRequest(demoReq);
  console.log(JSON.stringify(demoResp, null, 2));
}
