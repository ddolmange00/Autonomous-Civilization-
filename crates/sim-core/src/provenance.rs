use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EvidenceKind { Measured, Calculated, Fitted, Inferred, GameCalibrated }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SourceMetadata {
    pub source_id: String,
    pub citation: String,
    pub url_or_doi: String,
    pub version_or_access_date: String,
    pub license_note: String,
    pub evidence_kind: EvidenceKind,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PropertyDatum {
    pub property: String,
    pub value_si: f64,
    pub lower_si: Option<f64>,
    pub upper_si: Option<f64>,
    pub original_unit: String,
    pub temperature_k: Option<f64>,
    pub pressure_pa: Option<f64>,
    pub moisture_fraction: Option<f64>,
    pub process_condition: Option<String>,
    pub source: SourceMetadata,
    pub transformation_note: Option<String>,
    pub fidelity_tier: u8,
}
