use std::env;
extern crate dotenv;
use crate::errors::*;
use dotenv::dotenv;
use mongodb::{
    bson::{self, doc},
    options::UpdateOptions,
    Client, Collection,
};
use solana_program::pubkey::Pubkey;

use crate::models::{
    cursor_model::{Cursor, CursorWrapper},
    program_model::{Program, ProgramWrapper},
    program_queue_model::{FetchReason, ProgramQueue, ProgramQueueWrapper},
};
use solana_sdk::signature::Signature;

pub struct MongoRepo {
    program: Collection<ProgramWrapper>,
    queue: Collection<ProgramQueueWrapper>,
    cursor: Collection<CursorWrapper>,
}

// TODO : Create a Wrapper trait later if everything is working

impl MongoRepo {
    pub async fn init() -> Self {
        dotenv().ok();
        let uri = match env::var("MONGOURI") {
            Ok(v) => v.to_string(),
            Err(_) => format!("Error loading MONGOURI env variable"),
        };
        let client = Client::with_uri_str(uri).await.unwrap();
        let db = client.database("pgm-monit");
        let program: Collection<ProgramWrapper> = db.collection("Program");
        let queue: Collection<ProgramQueueWrapper> = db.collection("Queue");
        let cursor: Collection<CursorWrapper> = db.collection("Cursor");
        MongoRepo {
            program,
            queue,
            cursor,
        }
    }

    pub async fn insert_current_signature(&self, signature: &String) -> Result<()> {
        let filter = doc! {};
        let update = doc! { "$set": {"signature": signature } };
        let mut options = UpdateOptions::default();
        options.upsert = Some(true);
        self.cursor.update_one(filter, update, options).await?;
        Ok(())
    }

    pub async fn get_last_signature(&self) -> Result<String> {
        let filter = doc! {};
        let cw = self.cursor.find_one(filter, None).await?;
        Ok(cw.unwrap().cursor.signature)
    }

    pub async fn insert_program(&self, program: &Program) -> Result<()> {
        let new_doc = ProgramWrapper {
            program: program.clone(),
            ..ProgramWrapper::default()
        };
        self.program.insert_one(new_doc, None).await?;
        Ok(())
    }

    pub async fn update_program(&self, program: &Program) -> Result<()> {
        let filter = doc! {"pubkey": bson::to_bson(&program.pubkey).ok() };
        let update = doc! { "$set": bson::to_bson(program).ok() };
        self.program.update_one(filter, update, None).await?;
        Ok(())
    }

    pub async fn insert_many_into_queue(&self, program_queues: &Vec<ProgramQueue>) -> Result<()> {
        let mut new_docs: Vec<ProgramQueueWrapper> = Vec::new();
        for &program_queue in program_queues {
            new_docs.push(ProgramQueueWrapper {
                program_queue,
                ..ProgramQueueWrapper::default()
            });
        }
        self.queue.insert_many(new_docs, None).await?;
        Ok(())
    }
}
