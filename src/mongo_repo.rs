use std::env;
extern crate dotenv;
use dotenv::dotenv;
use mongodb::{
    bson::{self, doc},
    error,
    results::{InsertManyResult, InsertOneResult, UpdateResult},
    Client, Collection,
};
use solana_program::pubkey::Pubkey;

use crate::models::{
    program_model::Program,
    queue_model::{FetchReason, Queue},
};

pub struct MongoRepo {
    program: Collection<Program>,
    queue: Collection<Queue>,
}

impl MongoRepo {
    pub async fn init() -> Self {
        dotenv().ok();
        let uri = match env::var("MONGOURI") {
            Ok(v) => v.to_string(),
            Err(_) => format!("Error loading MONGOURI env variable"),
        };
        let client = Client::with_uri_str(uri).await.unwrap();
        let db = client.database("pgm-monit");
        let program: Collection<Program> = db.collection("Program");
        let queue: Collection<Queue> = db.collection("Queue");
        MongoRepo { program, queue }
    }

    pub async fn insert_program(
        &self,
        pubkey: Pubkey,
        data: Vec<u8>,
    ) -> Result<InsertOneResult, error::Error> {
        let new_doc = Program {
            id: None,
            pubkey,
            data,
        };
        self.program.insert_one(new_doc, None).await
    }

    pub async fn update_program(&self, program: Program) -> Result<UpdateResult, error::Error> {
        let filter = doc! {"pubkey": bson::to_bson(&program.pubkey).ok() };
        let update = doc! { "$set": bson::to_bson(&program).ok() };
        self.program.update_one(filter, update, None).await
    }

    pub async fn insert_into_queue(
        &self,
        program_account: Pubkey,
        fetch_reason: FetchReason,
    ) -> Result<InsertOneResult, error::Error> {
        let new_doc = Queue {
            id: None,
            program_account,
            fetch_reason,
        };
        self.queue.insert_one(new_doc, None).await
    }

    pub async fn insert_many_into_queue(
        &self,
        program_accounts: Vec<Pubkey>,
        fetch_reason: FetchReason,
    ) -> Result<InsertManyResult, error::Error> {
        let mut new_docs: Vec<Queue> = Vec::new();
        for program_account in program_accounts {
            new_docs.push(Queue {
                id: None,
                program_account,
                fetch_reason,
            });
        }
        self.queue.insert_many(new_docs, None).await
    }
}
