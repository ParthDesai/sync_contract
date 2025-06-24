use std::str::FromStr;
use anchor_client::{Client, Cluster};
use anchor_client::anchor_lang::Id;
use anchor_client::anchor_lang::prelude::{Pubkey, System};
use anchor_client::solana_client::rpc_client::RpcClient;
use anchor_client::solana_sdk::commitment_config::CommitmentConfig;
use anchor_client::solana_sdk::native_token::LAMPORTS_PER_SOL;
use anchor_client::solana_sdk::signature::Keypair;
use anchor_client::solana_sdk::signer::Signer;
use sync_contract::types::{AgentConfig, AgentResponse, DataHeader, Datasubmission, CURRENT_VERSION, DATA_LINK_SIZE};
use crate::test_initialize::init_program;

#[test]
fn test_whole_flow() {
    let program_id = "mDThuqfND6diLJhPqe12PTfYKBpWrpEbZht3RJtABMt";
    let anchor_rpc_client = RpcClient::new(Cluster::Localnet.url());

    let payer = Keypair::new();
    anchor_rpc_client
        .request_airdrop(&payer.pubkey(), 10000 * LAMPORTS_PER_SOL)
        .unwrap();

    let agent = Keypair::new();
    anchor_rpc_client
        .request_airdrop(&agent.pubkey(), 10000 * LAMPORTS_PER_SOL)
        .unwrap();

    let user = Keypair::new();
    anchor_rpc_client
        .request_airdrop(&user.pubkey(), 10000 * LAMPORTS_PER_SOL)
        .unwrap();

    let client = Client::new_with_options(Cluster::Localnet, &payer, CommitmentConfig::confirmed());
    let program_id = Pubkey::from_str(program_id).unwrap();
    let program = client.program(program_id).unwrap();

    init_program(&program, &payer);

    let (agent_config, _) = Pubkey::find_program_address(
        &[b"sync_program".as_ref(), b"agent_config".as_ref(), agent.pubkey().as_ref()],
        &program.id(),
    );

    program
        .request()
        .accounts(sync_contract::accounts::CreateAgent {
            agent_config,
            signer: agent.pubkey(),
            system_program: System::id(),
        })
        .args(sync_contract::instruction::CreateAgent {
        })
        .payer(&agent)
        .send()
        .expect("Valid create agent request must succeed.");

    let agent_config_data: AgentConfig = program
        .account(agent_config.clone())
        .expect("Agent config must exists after creation");
    assert_eq!(agent_config_data, AgentConfig {
        version: CURRENT_VERSION,
        is_enabled: false,
    });

    let (program_state, _) = Pubkey::find_program_address(
        &[b"sync_program".as_ref(), b"global_state".as_ref()],
        &program.id(),
    );

    program
        .request()
        .accounts(sync_contract::accounts::AllowAgent {
            program_state,
            agent_config,
            signer: agent.pubkey(),
        })
        .args(sync_contract::instruction::AllowAgent {
            agent_key: agent.pubkey()
        })
        .payer(&agent)
        .send().expect_err("non admin must not be able to call allow agent");


    // Not enabled agent cannot rate a submission

    // Submitting some data
    let data_link = "Hello world";
    let mut data_link_bytes = [0u8; DATA_LINK_SIZE];
    data_link_bytes[0..data_link.as_bytes().len()].copy_from_slice(data_link.as_bytes());
    let (data_submission, _) = Pubkey::find_program_address(
        &[b"sync_program".as_ref(), b"data_submission".as_ref(), data_link.as_bytes()],
        &program.id(),
    );

    program
        .request()
        .accounts(sync_contract::accounts::SubmitData {
            data_submission,
            signer: user.pubkey(),
            system_program: System::id(),
        })
        .args(sync_contract::instruction::SubmitData {
            data_link: data_link.to_string(),
        })
        .payer(&user)
        .send().expect("user should be able to submit some data");


    let data_submission_content: Datasubmission = program
        .account(data_submission.clone())
        .expect("Data submission must exists after creation");
    assert_eq!(data_submission_content, Datasubmission {
        version: CURRENT_VERSION,
        data_link: data_link_bytes,
        agent_response: None,
        data_header: DataHeader {},
    });

    // Agent cannot rate it without being enabled
    program
        .request()
        .accounts(sync_contract::accounts::RateData {
            data_submission,
            agent_config,
            signer: agent.pubkey(),
        })
        .args(sync_contract::instruction::RateData {
            data_link: data_link.to_string(),
            passed: true,
        })
        .payer(&agent)
        .send().expect_err("disabled agent must not be able to rate some data");


    program
        .request()
        .accounts(sync_contract::accounts::AllowAgent {
            program_state,
            agent_config,
            signer: payer.pubkey(),
        })
        .args(sync_contract::instruction::AllowAgent {
            agent_key: agent.pubkey()
        })
        .send().expect("admin must be able to call allow agent");



    let agent_config_data: AgentConfig = program
        .account(agent_config.clone())
        .expect("Agent config must exists after creation");
    assert_eq!(agent_config_data, AgentConfig {
        version: CURRENT_VERSION,
        is_enabled: true,
    });

    program
        .request()
        .accounts(sync_contract::accounts::RateData {
            data_submission,
            agent_config,
            signer: agent.pubkey(),
        })
        .args(sync_contract::instruction::RateData {
            data_link: data_link.to_string(),
            passed: true,
        })
        .payer(&agent)
        .send().expect("enabled agent must be able to rate some data");


    let data_submission_content: Datasubmission = program
        .account(data_submission.clone())
        .expect("Data submission must exists after creation");
    assert_eq!(data_submission_content, Datasubmission {
        version: CURRENT_VERSION,
        data_link: data_link_bytes,
        agent_response: Some(AgentResponse {
            agent_key: agent.pubkey(),
            response: true,
        }),
        data_header: DataHeader {},
    });

    program
        .request()
        .accounts(sync_contract::accounts::RateData {
            data_submission,
            agent_config,
            signer: agent.pubkey(),
        })
        .args(sync_contract::instruction::RateData {
            data_link: data_link.to_string(),
            passed: true,
        })
        .payer(&agent)
        .send().expect_err("enabled agent cannot rate data again");
}
