#[macro_use]
extern crate error_chain;

pub mod consts;
pub mod errors;
pub mod models;
pub mod mongo_repo;

use crate::{errors::*, models::queue_model::FetchReason, mongo_repo::MongoRepo};
use consts::*;
use solana_account_decoder::{UiAccountEncoding, UiDataSliceConfig};
use solana_client::{rpc_client::*, rpc_config::*};
use solana_sdk::{account::Account, pubkey::Pubkey};

struct Fetcher {
    rpc_client: RpcClient,
}

impl Fetcher {
    pub fn fetch_all_program_pubkey(self) -> Result<Vec<(Pubkey, Account)>> {
        let config = RpcProgramAccountsConfig {
            filters: None,
            account_config: RpcAccountInfoConfig {
                encoding: Some(UiAccountEncoding::Base64),
                data_slice: Some(UiDataSliceConfig {
                    offset: 0,
                    length: 0,
                }),
                commitment: None,
                min_context_slot: None,
            },
            with_context: None,
        };
        Ok(self
            .rpc_client
            .get_program_accounts_with_config(&BPF_LOADER, config)?)
    }

    pub fn fetch_all_program_accounts(self) -> Result<Vec<(Pubkey, Account)>> {
        let accounts = self.fetch_all_program_pubkey()?;
        Ok(accounts)
    }
}

struct Scheduler {
    db: MongoRepo,
    fetcher: Fetcher,
}

impl Scheduler {
    pub async fn new() -> Self {
        let db = MongoRepo::init().await;
        let fetcher = Fetcher {
            rpc_client: RpcClient::new("https://api.mainnet-beta.solana.com"),
        };
        Scheduler { db, fetcher }
    }

    pub async fn manage(self) -> Result<()> {
        self.init_db_with_programs().await?;
        Ok(())
    }

    pub async fn init_db_with_programs(self) -> Result<i32> {
        let accounts = self.fetcher.fetch_all_program_accounts()?;
        let pubkeys = accounts.iter().map(|x| x.0).collect::<Vec<Pubkey>>();
        self.db
            .insert_many_into_queue(pubkeys, FetchReason::Create)
            .await?;
        println!("Fetched {} accounts", accounts.len());
        Ok(accounts.len() as i32)
    }
}

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    let scheduler = Scheduler::new().await;
    let ret = scheduler.manage().await;
    if ret.is_err() {
        println!("{}", ret.err().unwrap());
    } else {
        println!("Finished all tasks successfully !");
    }
}
