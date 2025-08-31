use anchor_client::anchor_lang::{AnchorDeserialize, AnchorSerialize};
use anchor_client::solana_sdk::pubkey::Pubkey;
use anchor_client::solana_sdk::signature::{Keypair, Signer};
use sync_contract::types::{
    AgentResponseV1, AgentResponseV2, DataHeader, DataSubmissionV1, DataSubmissionV2,
    Datasubmission, DATAHEADER_RESERVED_SIZE, DATA_LINK_SIZE, PRIMARY_CATEGORY_SIZE,
    SECONDARY_CATEGORY_SIZE,
};

#[test]
fn test_backward_serialization() {
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

    let synthetic_data_link = "Hello World processed";
    let mut synthetic_data_link_bytes = [0u8; DATA_LINK_SIZE];
    synthetic_data_link_bytes[0..synthetic_data_link.as_bytes().len()]
        .copy_from_slice(synthetic_data_link.as_bytes());

    let agent = Keypair::new();
    let user = Keypair::new();

    let data_submission = Datasubmission::V2(DataSubmissionV2 {
        data_link: data_link_bytes,
        agent_response: Some(AgentResponseV2 {
            agent_key: agent.pubkey(),
            is_valid: false,
            synthetic_data_link: Some(synthetic_data_link_bytes),
            rating: 85,
            calculated_credits: 85,
            is_seed_deleted: false,
        }),
        data_header: DataHeader {
            primary_category: primary_category_bytes,
            secondary_category: secondary_category_bytes,
            reserved: [1u8; DATAHEADER_RESERVED_SIZE],
        },
        user_id: user.pubkey(),
    });

    let mut bytes = vec![];
    data_submission.serialize(&mut bytes).unwrap();
    assert_eq!(Datasubmission::MAX_SIZE, bytes.len());
    let deserialized_data_submission = Datasubmission::deserialize(&mut bytes.as_slice()).unwrap();
    assert_eq!(deserialized_data_submission, data_submission);

    // Sample serialized data from v1 which should be able to be decoded by
    let previous_version_of_data_submission = vec![
        1u8, 72, 101, 108, 108, 111, 32, 119, 111, 114, 108, 100, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 122, 227, 221,
        208, 128, 166, 215, 204, 117, 235, 247, 250, 245, 52, 60, 77, 108, 110, 69, 47, 41, 150,
        84, 175, 71, 150, 111, 124, 218, 144, 254, 201, 1, 85, 85, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 72, 101, 97, 108, 116, 104, 99, 97, 114, 101, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 80, 97, 116, 105, 101, 110, 116, 68, 97, 116, 97, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 244, 17, 157, 161,
        123, 98, 108, 38, 164, 47, 97, 229, 187, 72, 147, 182, 27, 98, 12, 166, 151, 127, 189, 28,
        209, 234, 251, 118, 19, 58, 49, 130,
    ];
    let backward_compatible_data_submission =
        Datasubmission::deserialize(&mut previous_version_of_data_submission.as_slice()).unwrap();
    assert_eq!(
        backward_compatible_data_submission,
        Datasubmission::V1(DataSubmissionV1 {
            data_link: [
                72, 101, 108, 108, 111, 32, 119, 111, 114, 108, 100, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
            ],
            agent_response: Some(AgentResponseV1 {
                agent_key: Pubkey::from_str_const("9GiGPk9a8QJQPptfZ4BH3zaNdCQWaxei925aAjyA8XqA"),
                response: true,
                rating: 85,
                calculated_credits: 85,
            }),
            data_header: DataHeader {
                primary_category: [
                    72, 101, 97, 108, 116, 104, 99, 97, 114, 101, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
                ],
                secondary_category: [
                    80, 97, 116, 105, 101, 110, 116, 68, 97, 116, 97, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
                ],
                reserved: [
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
                ],
            },
            user_id: Pubkey::from_str_const("HRk4M8ZBUjAf9vyqmix75v6x9cMJr3qpYxndj3gH1cjs"),
        })
    );
}
