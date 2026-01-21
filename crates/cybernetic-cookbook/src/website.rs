use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageBlueprint {
    pub id: String,
    pub path: String,
    pub class: String,
    pub hexstamp: String,
    pub knowledge_factor: f64,
    pub risk_of_harm: f64,
    pub cybostate_factor: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebsiteAsset {
    pub id: String,
    pub version: String,
    pub pages: Vec<PageBlueprint>,
}

/// Cookbook helper: given a neurorights-bound website governance decision,
/// materialize/commit the page blueprint under Cookbook rules.
pub fn commit_page_blueprint(
    website: &mut WebsiteAsset,
    blueprint: PageBlueprint,
) {
    if let Some(existing) = website.pages.iter_mut().find(|p| p.id == blueprint.id) {
        // New version => new hexstamp; previous state remains in history/git/logs
        *existing = blueprint;
    } else {
        website.pages.push(blueprint);
    }
}
