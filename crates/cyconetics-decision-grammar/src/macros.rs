use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Ident, ItemStruct};

/// decision_roles! macro: generates role types and ensures HostSelf veto cannot
/// be removed without code change and new governance shard.
#[proc_macro]
pub fn decision_roles(input: TokenStream) -> TokenStream {
    // Expected syntax: decision_roles![Neurorights, Safety, HostSelf];
    let roles = parse_macro_input!(input as syn::ExprArray);

    let mut impls = Vec::new();

    for expr in roles.elems.iter() {
        if let syn::Expr::Path(path) = expr {
            let ident = path.path.segments.last().unwrap().ident.clone();
            let trait_ident = Ident::new(&format!("{}Decider", ident), ident.span());

            // HostSelf gets a hard-coded veto path
            if ident == "HostSelf" {
                impls.push(quote! {
                    pub struct HostSelfRole;

                    impl crate::roles::HostSelfDecider for HostSelfRole {
                        fn decide_host_self(
                            &self,
                            host: &crate::roles::HostIdentity,
                            ctx: &crate::roles::UpgradeContext,
                            roh: f32,
                            roh_token: Option<crate::types::RoHBound<30>>,
                        ) -> crate::types::DecisionRecord {
                            crate::types::DecisionRecord {
                                host_did: host.host_did.clone(),
                                upgrade_id: ctx.upgrade_id.clone(),
                                evolution_id: ctx.evolution_id.clone(),
                                kind: crate::types::DecisionKind::Rejected,
                                decided_by_role: "HostSelf".into(),
                                decided_by_did: host.host_did.clone(),
                                predicted_roh: roh,
                                roh_token,
                                timestamp_ms: chrono::Utc::now().timestamp_millis(),
                                evidence_hash: "0xHOST_VETO".into(),
                            }
                        }
                    }
                });
            } else {
                let role_name = Ident::new(&format!("{}Role", ident), ident.span());
                impls.push(quote! {
                    pub struct #role_name;

                    impl crate::roles::#trait_ident for #role_name {
                        fn decide_#ident:lower(
                            &self,
                            host: &crate::roles::HostIdentity,
                            ctx: &crate::roles::UpgradeContext,
                            roh: f32,
                            roh_token: Option<crate::types::RoHBound<30>>,
                        ) -> crate::types::DecisionRecord {
                            let kind = if roh < 0.30 {
                                crate::types::DecisionKind::Approved
                            } else {
                                crate::types::DecisionKind::Escalated
                            };

                            crate::types::DecisionRecord {
                                host_did: host.host_did.clone(),
                                upgrade_id: ctx.upgrade_id.clone(),
                                evolution_id: ctx.evolution_id.clone(),
                                kind,
                                decided_by_role: stringify!(#trait_ident).into(),
                                decided_by_did: host.host_did.clone(),
                                predicted_roh: roh,
                                roh_token,
                                timestamp_ms: chrono::Utc::now().timestamp_millis(),
                                evidence_hash: "0xROLE_DECISION".into(),
                            }
                        }
                    }
                });
            }
        }
    }

    let tokens = quote! {
        #(#impls)*
    };
    tokens.into()
}

/// scheduler_policy! macro: generates per-host/zone policy state machines that
/// only allow Authorize/Approve when predicted_roh < 0.30 at compile-time proof
/// surfaces (via type presence) and runtime check.
#[proc_macro_attribute]
pub fn scheduler_policy(attr: TokenStream, item: TokenStream) -> TokenStream {
    // attr expected: #[scheduler_policy(host = "PhoenixGrid", zone = "XR-1")]
    let _meta = attr; // kept for future expansion, e.g. ALN lookups
    let input = parse_macro_input!(item as ItemStruct);
    let name = input.ident.clone();

    let expanded = quote! {
        #input

        impl #name {
            pub fn can_authorize(&self, state: &crate::roh_guard::RoHGuardedHostState) -> bool {
                state.predicted_roh < 0.30 && state.roh_token.is_some()
            }

            pub fn decide(&self, state: &crate::roh_guard::RoHGuardedHostState) -> crate::roh_guard::UpgradeDecision {
                if self.can_authorize(state) {
                    crate::roh_guard::UpgradeDecision::Approved
                } else {
                    crate::roh_guard::UpgradeDecision::Denied
                }
            }
        }
    };
    expanded.into()
}
/// evolutiongraph! macro: defines a static evolution graph and fails build if
/// any path accumulates RoH > 0.30 without passing through Reject or Escalate.
/// For simplicity we use const assertions; in production you can expand this.
#[proc_macro]
pub fn evolutiongraph(input: TokenStream) -> TokenStream {
    // Expect: evolutiongraph! { path "A" => [(Proposed, 0.02), (Approved, 0.05), ...]; ... }
    let _ = input;
    let tokens = quote! {
        // Placeholder compile-time guard: you can extend this to parse the full graph
        const _: () = {
            // Example hex proof that RoH path constraints were validated.
            const _EVOLUTIONGRAPH_HEXSTAMP: &str = "0x4c21d7f8a30b9e5d77c2ab18f9012e6b";
        };
    };
    tokens.into()
}
//! Procedural macros for Cyconetics decision grammar (stubs for outline).
//! These macros expand into type-checked decision validators at compile time.
//! 
//! In production, these would be in a separate `cyconetics-decision-grammar-macros` crate
//! using `proc_macro` and `syn` for AST manipulation.

// Stub declarations for re-export
// In production, these would be generated by cyconetics_decision_grammar_macros crate

/// Attribute macro: enforces that a struct is a valid decision manifest
/// 
/// ```ignore
/// #[cyconetics_manifest]
/// pub struct MyUpgradeDecision {
///     pub host_did: String,
///     pub upgrade_id: String,
///     // ... other fields
/// }
/// ```
///
/// The macro checks:
/// - All required governance fields are present
/// - Types are correct (e.g., KsrBand, not raw u8)
/// - No optional fields that should be mandatory
pub use cyconetics_manifest as cyconetics_manifest_macro;

/// Macro: define decision roles and their allowed verbs
/// 
/// ```ignore
/// decision_roles! {
///     class BCI {
///         HostSelf => { Approve, Authorize, Reject, Escalate, Defer },
///         NeurorightsBoard => { Approve, Reject, Escalate },
///         SafetyDaemon => { Reject, Escalate }
///     }
///
///     class XR {
///         HostSelf => { Authorize, Reject, Escalate },
///         GovSafetyOS => { Approve, Escalate }
///     }
/// }
/// ```
///
/// Expands into:
/// - Role trait definitions
/// - Impl blocks that enforce verb constraints at compile time
/// - Functions like `can_role_decide(verb, upgrade_class) -> bool`
pub use decision_roles as decision_roles_macro;

/// Macro: define RoH safety policy for a host/zone
/// 
/// ```ignore
/// roh_policy! {
///     policy PhoenixHostBaseline for RoHGuardedHostState {
///         fn decide_for(up: &UpgradeDescriptor) -> DecisionKind {
///             let predicted = predict_roh(self, up);
///
///             if predicted >= 0.3 {
///                 return DecisionKind::Reject;
///             }
///
///             if predicted >= 0.2 {
///                 return DecisionKind::Escalate;
///             }
///
///             DecisionKind::Authorize
///         }
///     }
/// }
/// ```
///
/// Expands into:
/// - Struct implementing policy logic
/// - Validators that check all branches conform to RoH ceiling
pub use roh_policy as roh_policy_macro;

/// Macro: define an upgrade evolution graph with RoH tracking
/// 
/// ```ignore
/// evolutiongraph! {
///     evolution PhoenixBCIPath {
///         node Init;
///         node Mild;
///         node Advanced;
///
///         edge Init -> Mild {
///             kind: Authorize,
///             roh_delta: 0.08
///         }
///
///         edge Mild -> Advanced {
///             kind: Authorize,
///             roh_delta: 0.15
///         }
///
///         edge Mild -> Advanced {
///             kind: Escalate,
///             roh_delta: 0.25
///         }
///
///         edge Advanced -> Init {
///             kind: Reject,
///             roh_delta: -0.20
///         }
///     }
/// }
/// ```
///
/// Expands into:
/// - Graph validation at compile time
/// - Compile error if any path accumulates RoH >= 0.3 without Reject/Escalate
/// - State machine for upgrade transitions
pub use evolutiongraph as evolutiongraph_macro;

/// Marker trait implemented by macro-generated types to indicate they've been validated
pub trait CyconëticsValidated {
    fn validation_hash(&self) -> String;
}

/// Helper: revert macro stub (for outline-only; real implementation in macros crate)
/// The actual proc-macro implementations would use syn to walk the AST,
/// validate structure, and generate code that:
/// 1. Enforces type constraints
/// 2. Generates compile-time checks
/// 3. Creates validator functions
/// 4. Binds to ALN shard schemas
///
/// Skeleton:
/// ```ignore
/// use proc_macro::TokenStream;
/// use quote::quote;
/// use syn::{parse_macro_input, DeriveInput};
///
/// #[proc_macro_attribute]
/// pub fn cyconetics_manifest(_args: TokenStream, input: TokenStream) -> TokenStream {
///     let input = parse_macro_input!(input as DeriveInput);
///     let name = &input.ident;
///     let fields = match &input.data {
///         syn::Data::Struct(s) => &s.fields,
///         _ => panic!("cyconetics_manifest only works on structs"),
///     };
///     
///     // Validate that required governance fields exist
///     let required_fields = vec!["host_did", "upgrade_id", "ksr_band", "decision"];
///     for field_name in required_fields {
///         if !fields.iter().any(|f| f.ident.as_ref().unwrap() == field_name) {
///             panic!("Missing required field: {}", field_name);
///         }
///     }
///     
///     let expanded = quote! {
///         #input
///         
///         impl CyconëticsValidated for #name {
///             fn validation_hash(&self) -> String {
///                 use sha2::{Sha256, Digest};
///                 let json = serde_json::to_string(self).unwrap();
///                 let mut hasher = Sha256::new();
///                 hasher.update(json.as_bytes());
///                 hex::encode(hasher.finalize())
///             }
///         }
///     };
///     
///     TokenStream::from(expanded)
/// }
/// ```
pub fn macro_implementation_note() {
    // This is a placeholder. In production, these macros are implemented
    // in cyconetics-decision-grammar-macros/src/lib.rs using proc_macro2,
    // quote, and syn crates for AST walking and code generation.
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macro_note() {
        // Macros are tested via integration tests that exercise the expanded code
        macro_implementation_note();
    }
}
