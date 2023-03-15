use penumbra_crypto::{asset, note, Nullifier};
use std::string::String;

pub fn token_supply(asset_id: &asset::Id) -> String {
    format!("shielded_pool/assets/{asset_id}/token_supply")
}

pub fn known_assets() -> &'static str {
    "shielded_pool/known_assets"
}

pub fn denom_by_asset(asset_id: &asset::Id) -> String {
    format!("shielded_pool/assets/{asset_id}/denom")
}

pub fn note_source(note_commitment: &note::Commitment) -> String {
    format!("shielded_pool/note_source/{note_commitment}")
}

pub fn compact_block(height: u64) -> String {
    format!("shielded_pool/compact_block/{height}")
}

pub fn epoch_anchor_by_index(index: u64) -> String {
    format!("shielded_pool/epoch_anchor/{index}")
}

pub fn spent_nullifier_lookup(nullifier: &Nullifier) -> String {
    format!("shielded_pool/spent_nullifiers/{nullifier}")
}
