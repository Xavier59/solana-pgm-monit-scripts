use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use solana_program::pubkey::Pubkey;

#[derive(Serialize, Deserialize, Clone)]
pub struct Program {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub pubkey: Pubkey,
    pub data: Vec<u8>,
}
