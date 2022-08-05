use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Cursor {
    pub signature: String,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct CursorWrapper {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    #[serde(flatten)]
    pub cursor: Cursor,
}
