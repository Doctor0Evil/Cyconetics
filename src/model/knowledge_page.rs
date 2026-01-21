pub struct KnowledgePage {
    pub id: ContentHash,            // CAS hash (e.g., multihash)
    pub title: String,
    pub sections: Vec<Section>,
    pub ksrb: KsrBand,              // hex K,S,R
    pub roh: RohBound,              // scalar 0x00–0xFF
    pub neurorights: NeuroRightsTag,
    pub corridor_id: Option<CorridorId>,
    pub session_id: Option<SessionId>,
    pub author_did: Did,
    pub created_at: Timestamp,
    pub citations_ok: bool,         // derived, not set by AI
}

pub struct Section {
    pub heading: String,
    pub body_blocks: Vec<BodyBlock>,
}

pub enum BodyBlock {
    Paragraph { text: String, citations: Vec<CitationRef> },
    List { items: Vec<ListItem> },
    Table { schema: TableSchema, rows: Vec<TableRow> },
    Code { language: CodeLang, safe_snippet: String },
}

pub struct Citation {
    pub id: CitationRef,
    pub source_kind: SourceKind,  // Web, File, OnChain, Internal
    pub locator: String,          // URL, tx-hash, file-id
    pub access_time: Timestamp,
}

pub struct KsrBand {
    pub k: u8,  // K useful-knowledge hex
    pub s: u8,  // S social-impact hex
    pub r: u8,  // R risk-of-harm hex
}

pub struct RohBound {
    pub value: u8,  // 0x00–0xFF, target ≤ 0x30
}

pub enum NeuroRightsTag {
    PerceptionOnly,
    ActuationLinked,
    BciCorridorBound,
    SyntheticOnly,
}
