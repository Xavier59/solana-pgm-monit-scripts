#[macro_use]
extern crate error_chain;

pub mod consts;
pub mod errors;
pub mod models;
pub mod mongo_repo;

use std::{
    str::FromStr,
    thread::sleep,
    time::{Duration, SystemTime},
};

use crate::{errors::*, models::program_queue_model::FetchReason, mongo_repo::MongoRepo};
use consts::*;
use models::program_queue_model::ProgramQueue;
use solana_account_decoder::{UiAccountEncoding, UiDataSliceConfig};
use solana_client::{
    rpc_client::*, rpc_config::*, rpc_response::RpcConfirmedTransactionStatusWithSignature,
};
use solana_sdk::{account::Account, pubkey::Pubkey, signature::Signature};
use solana_transaction_status::{
    parse_instruction::ParsedInstruction, UiInnerInstructions, UiInstruction, UiParsedInstruction,
    UiTransactionEncoding,
};

struct Fetcher {
    rpc_client: RpcClient,
}

impl Fetcher {
    pub fn fetch_current_slot(&self) -> Result<u64> {
        Ok(self.rpc_client.get_slot()?)
    }

    pub fn fetch_bfp_upgradable_loader_last_signature(&self) -> Result<String> {
        let mut config = GetConfirmedSignaturesForAddress2Config::default();
        config.limit = Some(1);
        let signatures_context = self
            .rpc_client
            .get_signatures_for_address_with_config(&BPF_LOADER, config)?;
        Ok(signatures_context.get(0).unwrap().signature.clone())
    }

    pub fn fetch_all_program_pubkey(&self) -> Result<Vec<(Pubkey, Account)>> {
        let mut config = RpcProgramAccountsConfig::default();
        config.account_config = RpcAccountInfoConfig {
            encoding: Some(UiAccountEncoding::Base64),
            data_slice: Some(UiDataSliceConfig {
                offset: 0,
                length: 0,
            }),
            ..RpcAccountInfoConfig::default()
        };
        Ok(self
            .rpc_client
            .get_program_accounts_with_config(&BPF_LOADER, config)?)
    }

    pub fn fetch_all_program_accounts(&self) -> Result<Vec<ProgramQueue>> {
        let accounts = self.fetch_all_program_pubkey()?;
        let pubkeys = accounts
            .iter()
            .map(|x| ProgramQueue {
                program_account: x.0,
                fetch_reason: FetchReason::Create,
                ..ProgramQueue::default()
            })
            .collect::<Vec<ProgramQueue>>();
        Ok(pubkeys)
    }

    pub fn fetch_last_program_create_or_update_or_close(
        &self,
        last_signature: String,
    ) -> Result<Vec<ProgramQueue>> {
        let mut config = GetConfirmedSignaturesForAddress2Config::default();
        config.until = Some(Signature::new(last_signature.as_bytes()));
        let signatures_context = self
            .rpc_client
            .get_signatures_for_address_with_config(&BPF_LOADER, config)?;
        let mut programs: Vec<ProgramQueue> = Vec::new();
        for RpcConfirmedTransactionStatusWithSignature { signature, .. } in
            signatures_context.iter()
        {
            let tx_context = self.rpc_client.get_transaction(
                &Signature::new(signature.as_bytes()),
                UiTransactionEncoding::Binary,
            )?;

            let tx_data = tx_context.transaction.transaction.decode().unwrap().message;
            let meta = tx_context.transaction.meta.unwrap();
            if meta.err.is_some() {
                continue;
            }
            if meta.inner_instructions.is_some() {
                Fetcher::parse_bpf_upgradable_loader_inner_instructions(
                    tx_context.slot,
                    &meta.inner_instructions.unwrap(),
                    &mut programs,
                )?;
            }
            for instr in tx_data.instructions().into_iter() {
                // need to parse it first
            }
        }
        Ok(programs)
    }

    fn parse_bpf_upgradable_loader_inner_instructions(
        slot: u64,
        inner_instructions: &Vec<UiInnerInstructions>,
        programs: &mut Vec<ProgramQueue>,
    ) -> Result<u64> {
        let no_of_programs = programs.len();
        for instr in inner_instructions
            .into_iter()
            .map(|ix| ix.instructions.clone())
            .flatten()
        {
            if let UiInstruction::Parsed(parsed) = instr {
                if let UiParsedInstruction::Parsed(parsed) = parsed {
                    Fetcher::parse_bpf_loader_compiled_instruction(slot, &parsed, programs)?;
                }
            }
        }
        Ok((programs.len() - no_of_programs) as u64)
    }

    fn parse_bpf_loader_compiled_instruction(
        slot: u64,
        instruction: &ParsedInstruction,
        programs: &mut Vec<ProgramQueue>,
    ) -> Result<u64> {
        let no_of_programs = programs.len();
        if Pubkey::from_str(&instruction.program_id).unwrap() == BPF_LOADER {
            // or is it type because of rename ?
            // TODO : close instr when https://github.com/solana-labs/solana/pull/26926 get merged
            let program = match instruction.parsed["instruction_type"].as_str().unwrap() {
                instr @ ("deployWithMaxDataLen" | "upgrade") => Some(ProgramQueue {
                    program_account: Pubkey::from_str(
                        instruction.parsed["info"]["programAccount"]
                            .as_str()
                            .unwrap(),
                    )
                    .unwrap(),
                    fetch_reason: if instr == "deployWithMaxDataLen" {
                        FetchReason::Create
                    } else {
                        FetchReason::Update
                    },
                    program_data_account: Some(
                        Pubkey::from_str(
                            instruction.parsed["info"]["programDataAccount"]
                                .as_str()
                                .unwrap(),
                        )
                        .unwrap(),
                    ),
                    deploy_slot: Some(slot),
                }),
                _ => None,
            };

            if program.is_some() {
                programs.push(program.unwrap());
            }
        }
        Ok((programs.len() - no_of_programs) as u64)
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

    pub async fn manage(&self) -> Result<()> {
        let signature = self.fetcher.fetch_bfp_upgradable_loader_last_signature()?;
        self.db.insert_current_signature(&signature).await?;
        self.init_db_with_programs().await?;
        loop {
            let ts = SystemTime::now();
            self.insert_last_programs().await?;
            sleep(
                Duration::from_secs(10).saturating_sub(Duration::from_millis(
                    (ts.elapsed()?.as_millis() as u128).try_into().unwrap(),
                )),
            );
        }
    }

    pub async fn init_db_with_programs(&self) -> Result<u32> {
        let pqs = self.fetcher.fetch_all_program_accounts()?;
        self.db.insert_many_into_queue(&pqs).await?;
        println!("Fetched {} accounts", pqs.len());
        Ok(pqs.len() as u32)
    }

    pub async fn insert_last_programs(&self) -> Result<u32> {
        let last_signature = self.db.get_last_signature().await?;
        let pqs = self
            .fetcher
            .fetch_last_program_create_or_update_or_close(last_signature)?;
        self.db.insert_many_into_queue(&pqs).await?;
        Ok(pqs.len() as u32)
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
