use mongodb::bson;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use solana_program::pubkey::Pubkey;

#[derive(Serialize, Deserialize, Clone)]
pub struct Timestamp(bson::Timestamp);

impl Default for Timestamp {
    fn default() -> Timestamp {
        Timestamp(bson::Timestamp {
            time: 0,
            increment: 0,
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct ProgramVersion {
    pub data: Vec<u8>,
    pub fetched_at: Timestamp,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Program {
    pub pubkey: Pubkey,
    pub version: ProgramVersion,
}
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct ProgramWrapper {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    #[serde(flatten)]
    pub program: Program,
}
