use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use solana_program::pubkey::Pubkey;

#[derive(Serialize, Deserialize, Clone, Copy, Default)]
pub enum FetchReason {
    #[default]
    Create,
    Update,
    Close,
}

#[derive(Serialize, Deserialize, Clone, Copy, Default)]
pub struct ProgramQueue {
    pub program_account: Pubkey,
    // no program data account when it's a close instruction
    pub program_data_account: Option<Pubkey>,
    pub deploy_slot: Option<u64>,
    pub fetch_reason: FetchReason,
}

#[derive(Serialize, Deserialize, Clone, Copy, Default)]
pub struct ProgramQueueWrapper {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    #[serde(flatten)]
    pub program_queue: ProgramQueue,
}
