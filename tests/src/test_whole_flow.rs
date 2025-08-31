use crate::test_initialize::init_program;
use anchor_client::anchor_lang::prelude::{Pubkey, System};
use anchor_client::anchor_lang::{AnchorDeserialize, AnchorSerialize, Id};
use anchor_client::solana_client::rpc_client::RpcClient;
use anchor_client::solana_sdk::commitment_config::CommitmentConfig;
use anchor_client::solana_sdk::keccak;
use anchor_client::solana_sdk::native_token::LAMPORTS_PER_SOL;
use anchor_client::solana_sdk::program_pack::Pack;
use anchor_client::solana_sdk::signature::Keypair;
use anchor_client::solana_sdk::signer::Signer;
use anchor_client::solana_sdk::system_instruction;
use anchor_client::{Client, Cluster};
use anchor_spl::associated_token::spl_associated_token_account;
use anchor_spl::token_2022::spl_token_2022;
use spl_token_2022::instruction::initialize_mint;
use spl_token_2022::state::Mint;
use std::str::FromStr;
use sync_contract::types::{
    AgentConfig, AgentResponseV2, DataHeader, DataSubmissionV2, Datasubmission, UserConfig,
    CURRENT_VERSION, DATAHEADER_RESERVED_SIZE, DATA_LINK_SIZE, PRIMARY_CATEGORY_SIZE,
    SECONDARY_CATEGORY_SIZE, USER_CONFIG_RESERVED_SIZE,
};

fn create_mint_account(
    rpc_client: &RpcClient,
    payer: &Keypair,
    mint_account: &Keypair,
    mint_authority: &Pubkey,
) {
    // Create the mint account
    let mint_account_rent = rpc_client
        .get_minimum_balance_for_rent_exemption(spl_token::state::Mint::LEN)
        .unwrap();

    let create_mint_account_ix = system_instruction::create_account(
        &payer.pubkey(),
        &mint_account.pubkey(),
        mint_account_rent * 10,
        spl_token::state::Mint::LEN as u64,
        &spl_token_2022::id(),
    );

    // Initialize the mint
    let initialize_mint_ix = initialize_mint(
        &spl_token_2022::id(),
        &mint_account.pubkey(),
        &mint_authority,
        None,
        9, // decimals
    )
    .unwrap();

    // Create and send transaction
    let recent_blockhash = rpc_client.get_latest_blockhash().unwrap();
    let create_mint_tx = anchor_client::solana_sdk::transaction::Transaction::new_signed_with_payer(
        &[create_mint_account_ix, initialize_mint_ix],
        Some(&payer.pubkey()),
        &[&payer, &mint_account],
        recent_blockhash,
    );

    rpc_client
        .send_and_confirm_transaction(&create_mint_tx)
        .unwrap();
}

fn fetch_mint_account(
    rpc_client: &RpcClient,
    mint_pubkey: &Pubkey,
) -> Result<Mint, Box<dyn std::error::Error>> {
    // Get the account data
    let account_data = rpc_client.get_account_data(mint_pubkey)?;

    // Unpack the mint data
    let mint = Mint::unpack(&account_data)?;

    Ok(mint)
}

fn print_mint_info(rpc_client: &RpcClient, mint_pubkey: &Pubkey) {
    match fetch_mint_account(rpc_client, mint_pubkey) {
        Ok(mint) => {
            println!("📊 MINT ACCOUNT INFO:");
            println!("   Address: {}", mint_pubkey);
            println!("   Mint Authority: {:?}", mint.mint_authority);
            println!("   Supply: {}", mint.supply);
            println!("   Decimals: {}", mint.decimals);
            println!("   Is Initialized: {}", mint.is_initialized);
            println!("   Freeze Authority: {:?}", mint.freeze_authority);
        }
        Err(e) => {
            println!("❌ Error fetching mint account {}: {}", mint_pubkey, e);
        }
    }
}

#[test]
fn test_whole_flow() {
    let program_id = "HkUDiDMSDntG1p4CgEaT9nhVrZ6MpPTVyL8rYiX3nvxy";
    let anchor_rpc_client =
        RpcClient::new_with_commitment(Cluster::Localnet.url(), CommitmentConfig::finalized());

    let payer = Keypair::new();
    println!("Payer Pubkey: {:?}", payer.pubkey());
    anchor_rpc_client
        .request_airdrop(&payer.pubkey(), 100 * LAMPORTS_PER_SOL)
        .unwrap();

    let agent = Keypair::new();
    anchor_rpc_client
        .request_airdrop(&agent.pubkey(), 100 * LAMPORTS_PER_SOL)
        .unwrap();

    let user = Keypair::from_bytes(&[
        111, 232, 119, 3, 204, 103, 148, 61, 101, 39, 172, 229, 84, 19, 249, 25, 241, 122, 92, 144,
        15, 24, 230, 118, 44, 108, 127, 132, 201, 169, 81, 153, 244, 17, 157, 161, 123, 98, 108,
        38, 164, 47, 97, 229, 187, 72, 147, 182, 27, 98, 12, 166, 151, 127, 189, 28, 209, 234, 251,
        118, 19, 58, 49, 130,
    ])
    .unwrap();
    println!("User Pubkey: {:?}", user.pubkey());
    anchor_rpc_client
        .request_airdrop(&user.pubkey(), 100 * LAMPORTS_PER_SOL)
        .unwrap();

    std::thread::sleep(std::time::Duration::from_secs(10));

    let client = Client::new_with_options(Cluster::Localnet, &payer, CommitmentConfig::confirmed());
    let program_id = Pubkey::from_str(program_id).unwrap();
    let program = client.program(program_id).unwrap();

    let (program_state_pubkey, _) = init_program(&program, &payer);

    let (agent_config, _) = Pubkey::find_program_address(
        &[
            b"sync_program".as_ref(),
            b"agent_config".as_ref(),
            agent.pubkey().as_ref(),
        ],
        &program.id(),
    );

    let mint_account = Keypair::from_bytes(&[
        138, 100, 129, 3, 92, 243, 121, 32, 39, 179, 165, 170, 49, 143, 72, 19, 159, 154, 32, 251,
        35, 136, 100, 65, 165, 143, 177, 235, 131, 247, 52, 210, 118, 112, 118, 167, 92, 207, 142,
        251, 48, 187, 70, 117, 187, 189, 234, 102, 241, 57, 230, 62, 89, 76, 35, 214, 132, 18, 54,
        144, 105, 28, 253, 91,
    ])
    .unwrap();
    println!("Mint account Pubkey: {:?}", mint_account.pubkey());
    create_mint_account(
        &anchor_rpc_client,
        &payer,
        &mint_account,
        &program_state_pubkey,
    );

    // Fetch and display mint account information
    print_mint_info(&anchor_rpc_client, &mint_account.pubkey());

    program
        .request()
        .accounts(sync_contract::accounts::CreateAgent {
            agent_config,
            signer: agent.pubkey(),
            system_program: System::id(),
        })
        .args(sync_contract::instruction::CreateAgent {})
        .payer(&agent)
        .send()
        .expect("Valid create agent request must succeed.");

    let agent_config_data: AgentConfig = program
        .account(agent_config.clone())
        .expect("Agent config must exists after creation");
    assert_eq!(
        agent_config_data,
        AgentConfig {
            version: CURRENT_VERSION,
            is_enabled: false,
        }
    );

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
            _agent_key: agent.pubkey(),
        })
        .payer(&agent)
        .send()
        .expect_err("non admin must not be able to call allow agent");

    // Not enabled agent cannot rate a submission

    let synthetic_data_link = "Hello World processed";
    let mut synthetic_data_link_bytes = [0u8; DATA_LINK_SIZE];
    synthetic_data_link_bytes[0..synthetic_data_link.as_bytes().len()]
        .copy_from_slice(synthetic_data_link.as_bytes());

    // Submitting some data
    let data_link = "Hello world";
    let mut data_link_bytes = [0u8; DATA_LINK_SIZE];
    data_link_bytes[0..data_link.as_bytes().len()].copy_from_slice(data_link.as_bytes());

    let primary_category = "Healthcare";
    let mut primary_category_bytes = [0u8; PRIMARY_CATEGORY_SIZE];
    primary_category_bytes[0..primary_category.as_bytes().len()]
        .copy_from_slice(primary_category.as_bytes());

    let secondary_category = "PatientData";
    let mut secondary_category_bytes = [0u8; SECONDARY_CATEGORY_SIZE];
    secondary_category_bytes[0..secondary_category.as_bytes().len()]
        .copy_from_slice(secondary_category.as_bytes());

    let (data_submission, _) = Pubkey::find_program_address(
        &[
            b"sync_program".as_ref(),
            b"data_submission".as_ref(),
            keccak::hash(data_link.as_bytes()).as_ref(),
        ],
        &program.id(),
    );

    let (user_config_key, _) = Pubkey::find_program_address(
        &[
            b"sync_program".as_ref(),
            b"user_config".as_ref(),
            user.pubkey().as_ref(),
        ],
        &program.id(),
    );

    program
        .request()
        .accounts(sync_contract::accounts::SubmitData {
            data_submission,
            user_config: user_config_key,
            signer: user.pubkey(),
            system_program: System::id(),
        })
        .args(sync_contract::instruction::SubmitData {
            data_link: data_link.to_string(),
            primary_category: primary_category.to_string(),
            secondary_category: secondary_category.to_string(),
        })
        .payer(&user)
        .send()
        .expect("user should be able to submit some data");

    let data_submission_content: Datasubmission = program
        .account(data_submission.clone())
        .expect("Data submission must exists after creation");
    assert_eq!(
        data_submission_content,
        Datasubmission::V2(DataSubmissionV2 {
            data_link: data_link_bytes,
            agent_response: None,
            data_header: DataHeader {
                primary_category: primary_category_bytes,
                secondary_category: secondary_category_bytes,
                reserved: [0u8; DATAHEADER_RESERVED_SIZE],
            },
            user_id: user.pubkey(),
        })
    );

    // Agent cannot rate it without being enabled
    program
        .request()
        .accounts(sync_contract::accounts::RateData {
            data_submission,
            agent_config,
            user_config: user_config_key,
            signer: agent.pubkey(),
        })
        .args(sync_contract::instruction::RateData {
            _data_link: data_link.to_string(),
            is_seed_deleted: true,
            rating: 100,
            synthetic_data_link: None,
        })
        .payer(&agent)
        .send()
        .expect_err("disabled agent must not be able to rate some data");

    program
        .request()
        .accounts(sync_contract::accounts::AllowAgent {
            program_state,
            agent_config,
            signer: payer.pubkey(),
        })
        .args(sync_contract::instruction::AllowAgent {
            _agent_key: agent.pubkey(),
        })
        .send()
        .expect("admin must be able to call allow agent");

    let agent_config_data: AgentConfig = program
        .account(agent_config.clone())
        .expect("Agent config must exists after creation");
    assert_eq!(
        agent_config_data,
        AgentConfig {
            version: CURRENT_VERSION,
            is_enabled: true,
        }
    );

    // On rating agent must have deleted the file
    program
        .request()
        .accounts(sync_contract::accounts::RateData {
            data_submission,
            agent_config,
            user_config: user_config_key,
            signer: agent.pubkey(),
        })
        .args(sync_contract::instruction::RateData {
            _data_link: data_link.to_string(),
            is_seed_deleted: false,
            rating: 85,
            synthetic_data_link: Some(synthetic_data_link.to_string()),
        })
        .payer(&agent)
        .send()
        .expect_err("enabled agent must be deleting data");

    // Valid rate data instruction must be successful
    program
        .request()
        .accounts(sync_contract::accounts::RateData {
            data_submission,
            agent_config,
            user_config: user_config_key,
            signer: agent.pubkey(),
        })
        .args(sync_contract::instruction::RateData {
            _data_link: data_link.to_string(),
            is_seed_deleted: true,
            rating: 85,
            synthetic_data_link: Some(synthetic_data_link.to_string()),
        })
        .payer(&agent)
        .send()
        .expect("enabled agent must be able to rate some data");

    let data_submission_content: Datasubmission = program
        .account(data_submission.clone())
        .expect("Data submission must exists after creation");
    assert_eq!(
        data_submission_content,
        Datasubmission::V2(DataSubmissionV2 {
            data_link: data_link_bytes,
            agent_response: Some(AgentResponseV2 {
                agent_key: agent.pubkey(),
                is_valid: true,
                synthetic_data_link: Some(synthetic_data_link_bytes),
                rating: 85,
                calculated_credits: 85,
                is_seed_deleted: true,
            }),
            data_header: DataHeader {
                primary_category: primary_category_bytes,
                secondary_category: secondary_category_bytes,
                reserved: [0u8; DATAHEADER_RESERVED_SIZE],
            },
            user_id: user.pubkey(),
        })
    );

    let user_config_content: UserConfig = program
        .account(user_config_key.clone())
        .expect("User config must exists after creation");

    assert_eq!(
        user_config_content,
        UserConfig {
            version: 1,
            accumulated_credits: 85,
            reserved: [0; USER_CONFIG_RESERVED_SIZE],
        }
    );

    program
        .request()
        .accounts(sync_contract::accounts::RateData {
            data_submission,
            agent_config,
            user_config: user_config_key,
            signer: agent.pubkey(),
        })
        .args(sync_contract::instruction::RateData {
            _data_link: data_link.to_string(),
            is_seed_deleted: false,
            rating: 100,
            synthetic_data_link: None,
        })
        .payer(&agent)
        .send()
        .expect_err("enabled agent cannot rate data again");

    // We create another data link
    let data_link = "Hello world 1";
    let mut data_link_bytes = [0u8; DATA_LINK_SIZE];
    data_link_bytes[0..data_link.as_bytes().len()].copy_from_slice(data_link.as_bytes());

    let primary_category = "Healthcare";
    let mut primary_category_bytes = [0u8; PRIMARY_CATEGORY_SIZE];
    primary_category_bytes[0..primary_category.as_bytes().len()]
        .copy_from_slice(primary_category.as_bytes());

    let secondary_category = "PatientData";
    let mut secondary_category_bytes = [0u8; SECONDARY_CATEGORY_SIZE];
    secondary_category_bytes[0..secondary_category.as_bytes().len()]
        .copy_from_slice(secondary_category.as_bytes());

    let (data_submission, _) = Pubkey::find_program_address(
        &[
            b"sync_program".as_ref(),
            b"data_submission".as_ref(),
            keccak::hash(data_link.as_bytes()).as_ref(),
        ],
        &program.id(),
    );

    program
        .request()
        .accounts(sync_contract::accounts::SubmitData {
            data_submission,
            user_config: user_config_key,
            signer: user.pubkey(),
            system_program: System::id(),
        })
        .args(sync_contract::instruction::SubmitData {
            data_link: data_link.to_string(),
            primary_category: primary_category.to_string(),
            secondary_category: secondary_category.to_string(),
        })
        .payer(&user)
        .send()
        .expect("user should be able to submit some data");

    let data_submission_content: Datasubmission = program
        .account(data_submission.clone())
        .expect("Data submission must exists after creation");
    assert_eq!(
        data_submission_content,
        Datasubmission::V2(DataSubmissionV2 {
            data_link: data_link_bytes,
            agent_response: None,
            data_header: DataHeader {
                primary_category: primary_category_bytes,
                secondary_category: secondary_category_bytes,
                reserved: [0u8; DATAHEADER_RESERVED_SIZE],
            },
            user_id: user.pubkey(),
        })
    );

    // Agent rates data once again
    program
        .request()
        .accounts(sync_contract::accounts::RateData {
            data_submission,
            agent_config,
            user_config: user_config_key,
            signer: agent.pubkey(),
        })
        .args(sync_contract::instruction::RateData {
            _data_link: data_link.to_string(),
            is_seed_deleted: true,
            rating: 82,
            synthetic_data_link: None,
        })
        .payer(&agent)
        .send()
        .expect("enabled agent must be able to rate some data");

    let data_submission_content: Datasubmission = program
        .account(data_submission.clone())
        .expect("Data submission must exists after creation");
    assert_eq!(
        data_submission_content,
        Datasubmission::V2(DataSubmissionV2 {
            data_link: data_link_bytes,
            agent_response: Some(AgentResponseV2 {
                agent_key: agent.pubkey(),
                is_valid: false,
                synthetic_data_link: None,
                rating: 82,
                calculated_credits: 0,
                is_seed_deleted: true,
            }),
            data_header: DataHeader {
                primary_category: primary_category_bytes,
                secondary_category: secondary_category_bytes,
                reserved: [0u8; DATAHEADER_RESERVED_SIZE],
            },
            user_id: user.pubkey(),
        })
    );

    let user_config_content: UserConfig = program
        .account(user_config_key.clone())
        .expect("User config must exists after creation");

    assert_eq!(
        user_config_content,
        UserConfig {
            version: 1,
            accumulated_credits: 85,
            reserved: [0; USER_CONFIG_RESERVED_SIZE],
        }
    );

    // Rate data one more time
    let data_link = "Hello world 2";
    let mut data_link_bytes = [0u8; DATA_LINK_SIZE];
    data_link_bytes[0..data_link.as_bytes().len()].copy_from_slice(data_link.as_bytes());

    let primary_category = "Healthcare";
    let mut primary_category_bytes = [0u8; PRIMARY_CATEGORY_SIZE];
    primary_category_bytes[0..primary_category.as_bytes().len()]
        .copy_from_slice(primary_category.as_bytes());

    let secondary_category = "PatientData";
    let mut secondary_category_bytes = [0u8; SECONDARY_CATEGORY_SIZE];
    secondary_category_bytes[0..secondary_category.as_bytes().len()]
        .copy_from_slice(secondary_category.as_bytes());

    let (data_submission, _) = Pubkey::find_program_address(
        &[
            b"sync_program".as_ref(),
            b"data_submission".as_ref(),
            keccak::hash(data_link.as_bytes()).as_ref(),
        ],
        &program.id(),
    );

    program
        .request()
        .accounts(sync_contract::accounts::SubmitData {
            data_submission,
            user_config: user_config_key,
            signer: user.pubkey(),
            system_program: System::id(),
        })
        .args(sync_contract::instruction::SubmitData {
            data_link: data_link.to_string(),
            primary_category: primary_category.to_string(),
            secondary_category: secondary_category.to_string(),
        })
        .payer(&user)
        .send()
        .expect("user should be able to submit some data");

    let data_submission_content: Datasubmission = program
        .account(data_submission.clone())
        .expect("Data submission must exists after creation");
    assert_eq!(
        data_submission_content,
        Datasubmission::V2(DataSubmissionV2 {
            data_link: data_link_bytes,
            agent_response: None,
            data_header: DataHeader {
                primary_category: primary_category_bytes,
                secondary_category: secondary_category_bytes,
                reserved: [0u8; DATAHEADER_RESERVED_SIZE],
            },
            user_id: user.pubkey(),
        })
    );

    // Agent rates data once again
    program
        .request()
        .accounts(sync_contract::accounts::RateData {
            data_submission,
            agent_config,
            user_config: user_config_key,
            signer: agent.pubkey(),
        })
        .args(sync_contract::instruction::RateData {
            _data_link: data_link.to_string(),
            is_seed_deleted: true,
            rating: 20,
            synthetic_data_link: Some(synthetic_data_link.to_string()),
        })
        .payer(&agent)
        .send()
        .expect("enabled agent must be able to rate some data");

    let data_submission_content: Datasubmission = program
        .account(data_submission.clone())
        .expect("Data submission must exists after creation");
    assert_eq!(
        data_submission_content,
        Datasubmission::V2(DataSubmissionV2 {
            data_link: data_link_bytes,
            agent_response: Some(AgentResponseV2 {
                agent_key: agent.pubkey(),
                is_valid: true,
                synthetic_data_link: Some(synthetic_data_link_bytes),
                rating: 20,
                calculated_credits: 20,
                is_seed_deleted: true,
            }),
            data_header: DataHeader {
                primary_category: primary_category_bytes,
                secondary_category: secondary_category_bytes,
                reserved: [0u8; DATAHEADER_RESERVED_SIZE],
            },
            user_id: user.pubkey(),
        })
    );

    let user_config_content: UserConfig = program
        .account(user_config_key.clone())
        .expect("User config must exists after creation");

    assert_eq!(
        user_config_content,
        UserConfig {
            version: 1,
            accumulated_credits: 105,
            reserved: [0; USER_CONFIG_RESERVED_SIZE],
        }
    );

    let associated_token_account =
        spl_associated_token_account::get_associated_token_address_with_program_id(
            &user.pubkey(),
            &mint_account.pubkey(),
            &spl_token_2022::id(),
        );

    println!("Program state: {}", program_state);
    println!("User config: {}", user_config_key);
    println!("Mint account: {}", mint_account.pubkey());
    println!("Associated token account: {}", associated_token_account);
    println!("User: {}", user.pubkey());
    println!("System program: {}", System::id());
    println!("Token program: {}", spl_token_2022::id());
    println!(
        "Associated token program: {}",
        spl_associated_token_account::id()
    );

    // Now claim the credits
    program
        .request()
        .accounts(sync_contract::accounts::ClaimCredits {
            program_state,
            user_config: user_config_key,
            mint: mint_account.pubkey(),
            token_account: associated_token_account,
            signer: user.pubkey(),
            system_program: System::id(),
            token_program: spl_token_2022::id(),
            associated_token_program: spl_associated_token_account::id(),
        })
        .args(sync_contract::instruction::ClaimCredits {})
        .payer(&user)
        .send()
        .expect("user should be able to claim credits");

    std::thread::sleep(std::time::Duration::from_secs(20));

    let token_account_info = anchor_rpc_client
        .get_token_account(&associated_token_account)
        .unwrap();

    let mint_account_contents =
        fetch_mint_account(&anchor_rpc_client, &mint_account.pubkey()).unwrap();

    assert_eq!(
        token_account_info.unwrap().token_amount.amount,
        (105 * 10u64.pow(mint_account_contents.decimals as u32)).to_string()
    );

    // User credit should be 0
    let user_config_content: UserConfig = program
        .account(user_config_key.clone())
        .expect("User config must exists after creation");
    assert_eq!(
        user_config_content,
        UserConfig {
            version: 1,
            accumulated_credits: 0,
            reserved: [0; USER_CONFIG_RESERVED_SIZE],
        }
    );

    // Pass the mint authority to the new address can only be done by the admin
    program
        .request()
        .accounts(sync_contract::accounts::TransferMintAuthority {
            program_state,
            signer: user.pubkey(),
            system_program: System::id(),
            mint: mint_account.pubkey(),
            token_program: spl_token_2022::id(),
        })
        .args(sync_contract::instruction::TransferMintAuthority {
            new_authority: payer.pubkey(),
        })
        .payer(&user)
        .send()
        .expect_err("user must not be able to transfer mint authority");

    // Now transfer the mint authority to the admin
    program
        .request()
        .accounts(sync_contract::accounts::TransferMintAuthority {
            program_state,
            signer: payer.pubkey(),
            system_program: System::id(),
            mint: mint_account.pubkey(),
            token_program: spl_token_2022::id(),
        })
        .args(sync_contract::instruction::TransferMintAuthority {
            new_authority: payer.pubkey(),
        })
        .send()
        .expect("admin must be able to transfer mint authority");

    std::thread::sleep(std::time::Duration::from_secs(20));

    // Verify the mint authority has changed
    println!("\n🔄 After transferring mint authority:");
    print_mint_info(&anchor_rpc_client, &mint_account.pubkey());

    // You can also use fetch_mint_account directly for programmatic checks
    let mint_info = fetch_mint_account(&anchor_rpc_client, &mint_account.pubkey())
        .expect("Should be able to fetch mint account");
    assert_eq!(mint_info.mint_authority, Some(payer.pubkey()).into());

    // Now claim credits cannot be done by user
    program.request().accounts(sync_contract::accounts::ClaimCredits {
        program_state,
        user_config: user_config_key,
        mint: mint_account.pubkey(),
        token_account: associated_token_account,
        signer: user.pubkey(),
        system_program: System::id(),
        token_program: spl_token_2022::id(),
        associated_token_program: spl_associated_token_account::id(),
    }).args(sync_contract::instruction::ClaimCredits {
    }).payer(&user).send().expect_err("user should not be able to claim credits as program state is not mint authority anymore");
}
