mod domain;
mod logging;
mod tools;
mod router;
mod authorship;
mod trace;
mod normalize;

use std::sync::Arc;
use crate::logging::FileLogSink;
use crate::router::CyberRetrievalRouter;
use crate::authorship::AuthorshipConfig;
use crate::normalize::{RawPrompt, normalize_prompt};
use crate::domain::SecurityLevel;
use crate::tools::ToolAdapter;
use crate::adapters::drive_reader::DriveReaderAdapter;

#[tokio::main]
async fn main() {
    // Configure authorship defaults for this deployment.
    let authorship_cfg = AuthorshipConfig::new(
        Some("ALN:Phoenix-XR-Grid".into()),
        Some("bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7".into()),
    );

    // Register tools.
    let tools: Vec<Arc<dyn ToolAdapter>> = vec![Arc::new(DriveReaderAdapter)];
    let log_sink = Arc::new(FileLogSink::new("cyber_retrieval.log"));

    let router = CyberRetrievalRouter::new(tools, log_sink, 0.3);

    // Example normalized call.
    let raw = RawPrompt {
        user_did: "did:example:bostrom-user",
        text: "Retrieve governance-safe cybernetic research log index.",
        security_level: SecurityLevel::Restricted,
        intent_hint: None,
        extra_args: None,
    };

    let envelope = normalize_prompt(raw, &authorship_cfg);
    let _ = router.handle(envelope).await;
}
