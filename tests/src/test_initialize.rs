use anchor_client::anchor_lang::system_program::System;
use anchor_client::anchor_lang::Id;
use anchor_client::solana_client::rpc_client::RpcClient;
use anchor_client::solana_sdk::native_token::LAMPORTS_PER_SOL;
use anchor_client::solana_sdk::signature::Keypair;
use anchor_client::solana_sdk::signer::Signer;
use anchor_client::{
    solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey},
    Client, Cluster, Program,
};
use std::str::FromStr;
use std::thread::sleep;
use std::time::Duration;
use sync_contract::types::ProgramState;

pub fn init_program(program: &Program<&Keypair>, payer: &Keypair) -> (Pubkey, ProgramState) {
    // We need to wait a bit for validator to start
    sleep(Duration::from_secs(10));

    let (program_state, _) = Pubkey::find_program_address(
        &[b"sync_program".as_ref(), b"global_state".as_ref()],
        &program.id(),
    );

    // First initialization should be successful
    program
        .request()
        .accounts(sync_contract::accounts::Initialize {
            program_state: program_state.clone(),
            signer: payer.pubkey(),
            system_program: System::id(),
        })
        .args(sync_contract::instruction::Initialize {})
        .send()
        .expect("First initialization must be successful");

    // Second initialization should return an error
    program
        .request()
        .accounts(sync_contract::accounts::Initialize {
            program_state: program_state.clone(),
            signer: payer.pubkey(),
            system_program: System::id(),
        })
        .args(sync_contract::instruction::Initialize {})
        .send()
        .expect_err("Second initialization should not be successful");

    // We need to check if admin is set to payer
    let program_state_data: ProgramState = program
        .account(program_state.clone())
        .expect("Program state must exists after initialization");
    if program_state_data.admin != payer.pubkey() {
        panic!("Administrative must be payer");
    }

    (program_state, program_state_data)
}

#[test]
fn test_initialize() {
    let program_id = "HkUDiDMSDntG1p4CgEaT9nhVrZ6MpPTVyL8rYiX3nvxy";
    let anchor_rpc_client = RpcClient::new(Cluster::Localnet.url());

    let payer = Keypair::new();
    anchor_rpc_client
        .request_airdrop(&payer.pubkey(), 10000 * LAMPORTS_PER_SOL)
        .unwrap();

    let client = Client::new_with_options(Cluster::Localnet, &payer, CommitmentConfig::confirmed());
    let program_id = Pubkey::from_str(program_id).unwrap();
    let program = client.program(program_id).unwrap();

    init_program(&program, &payer);
}
