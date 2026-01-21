#[derive(Clone, Debug)]
pub struct Authorship {
    pub user_did: String,
    pub aln: String,
    pub bostrom_address: String,
    pub eibon_label: String,
    pub neurorights_version: String,
}

#[derive(Clone, Debug)]
pub struct EvidenceStamp {
    pub hex_stamp: String,
}

impl EvidenceStamp {
    pub fn default_hex() -> Self {
        Self {
            hex_stamp: "0xaf31c8e924f5703d8c4f2a19e5d44cb7".to_string(),
        }
    }
}
