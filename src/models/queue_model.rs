use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use solana_program::pubkey::Pubkey;

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum FetchReason {
    Create,
    Update,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct Queue {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub program_account: Pubkey,
    pub fetch_reason: FetchReason,
}
