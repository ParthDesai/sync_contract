use anchor_lang::prelude::Pubkey;
use anchor_lang::prelude::*;
use std::fmt;

pub const CURRENT_VERSION: u8 = 1;
pub const DATA_LINK_SIZE: usize = 256;
pub const PRIMARY_CATEGORY_SIZE: usize = 32;
pub const SECONDARY_CATEGORY_SIZE: usize = 32;
pub const DATAHEADER_RESERVED_SIZE: usize = 96;
pub const USER_CONFIG_RESERVED_SIZE: usize = 128;

#[account]
#[derive(Eq, PartialEq, Debug)]
pub struct AgentResponseV1 {
    pub agent_key: Pubkey,
    pub response: bool,
    pub rating: u8,
    pub calculated_credits: u128,
}

impl AgentResponseV1 {
    pub const MAX_SIZE: usize = 1 + 32 + 1 + 16;
}

#[account]
#[derive(Eq, PartialEq, Debug)]
pub struct AgentResponseV2 {
    pub agent_key: Pubkey,
    pub is_valid: bool,
    pub synthetic_data_link: Option<[u8; DATA_LINK_SIZE]>,
    pub is_seed_deleted: bool,
    pub rating: u8,
    pub calculated_credits: u128,
}

impl AgentResponseV2 {
    pub const MAX_SIZE: usize = 32 + 1 + 1 + DATA_LINK_SIZE + 1 + 1 + 16;
}

#[account]
#[derive(Eq, PartialEq, Debug)]
pub struct DataHeader {
    pub primary_category: [u8; PRIMARY_CATEGORY_SIZE],
    pub secondary_category: [u8; SECONDARY_CATEGORY_SIZE],
    pub reserved: [u8; DATAHEADER_RESERVED_SIZE],
}

impl DataHeader {
    pub const MAX_SIZE: usize =
        PRIMARY_CATEGORY_SIZE + SECONDARY_CATEGORY_SIZE + DATAHEADER_RESERVED_SIZE;
}

#[account]
#[derive(Eq, PartialEq, Debug)]
pub struct DataSubmissionV1 {
    pub data_link: [u8; DATA_LINK_SIZE],
    pub agent_response: Option<AgentResponseV1>,
    pub data_header: DataHeader,
    pub user_id: Pubkey,
}

impl DataSubmissionV1 {
    pub const MAX_SIZE: usize =
        DATA_LINK_SIZE + 1 + AgentResponseV1::MAX_SIZE + DataHeader::MAX_SIZE + 32;
}

#[account]
#[derive(Eq, PartialEq, Debug)]
pub struct DataSubmissionV2 {
    pub data_link: [u8; DATA_LINK_SIZE],
    pub agent_response: Option<AgentResponseV2>,
    pub data_header: DataHeader,
    pub user_id: Pubkey,
}

impl DataSubmissionV2 {
    pub const MAX_SIZE: usize =
        DATA_LINK_SIZE + 1 + AgentResponseV2::MAX_SIZE + DataHeader::MAX_SIZE + 32;
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum Datasubmission {
    V1(DataSubmissionV1),
    V2(DataSubmissionV2),
}

#[cfg(feature = "idl-build")]
impl IdlBuild for Datasubmission {
    fn create_type() -> Option<anchor_lang::idl::types::IdlTypeDef> {
        Some(anchor_lang::idl::types::IdlTypeDef {
            name: Self::get_full_path(),
            docs: Vec::new(),
            serialization: anchor_lang::idl::types::IdlSerialization::default(),
            repr: None,
            generics: Vec::new(),
            ty: anchor_lang::idl::types::IdlTypeDefTy::Enum {
                variants: vec![
                    anchor_lang::idl::types::IdlEnumVariant {
                        name: "V2".to_string(),
                        fields: Some(anchor_lang::idl::types::IdlDefinedFields::Named(
                            <[_]>::into_vec(Box::new([
                                anchor_lang::idl::types::IdlField {
                                    name: "data_link".into(),
                                    docs: Vec::new(),
                                    ty: anchor_lang::idl::types::IdlType::Array(
                                        Box::new(anchor_lang::idl::types::IdlType::U8),
                                        anchor_lang::idl::types::IdlArrayLen::Value(DATA_LINK_SIZE),
                                    ),
                                },
                                anchor_lang::idl::types::IdlField {
                                    name: "agent_response".into(),
                                    docs: Vec::new(),
                                    ty: anchor_lang::idl::types::IdlType::Option(Box::new(
                                        anchor_lang::idl::types::IdlType::Defined {
                                            name: <AgentResponseV2>::get_full_path(),
                                            generics: Vec::new(),
                                        },
                                    )),
                                },
                                anchor_lang::idl::types::IdlField {
                                    name: "data_header".into(),
                                    docs: Vec::new(),
                                    ty: anchor_lang::idl::types::IdlType::Defined {
                                        name: <DataHeader>::get_full_path(),
                                        generics: Vec::new(),
                                    },
                                },
                                anchor_lang::idl::types::IdlField {
                                    name: "user_id".into(),
                                    docs: Vec::new(),
                                    ty: anchor_lang::idl::types::IdlType::Pubkey,
                                },
                            ])),
                        )),
                    },
                    anchor_lang::idl::types::IdlEnumVariant {
                        name: "V1".to_string(),
                        fields: Some(anchor_lang::idl::types::IdlDefinedFields::Named(
                            <[_]>::into_vec(Box::new([
                                anchor_lang::idl::types::IdlField {
                                    name: "data_link".into(),
                                    docs: Vec::new(),
                                    ty: anchor_lang::idl::types::IdlType::Array(
                                        Box::new(anchor_lang::idl::types::IdlType::U8),
                                        anchor_lang::idl::types::IdlArrayLen::Value(DATA_LINK_SIZE),
                                    ),
                                },
                                anchor_lang::idl::types::IdlField {
                                    name: "agent_response".into(),
                                    docs: Vec::new(),
                                    ty: anchor_lang::idl::types::IdlType::Option(Box::new(
                                        anchor_lang::idl::types::IdlType::Defined {
                                            name: <AgentResponseV1>::get_full_path(),
                                            generics: Vec::new(),
                                        },
                                    )),
                                },
                                anchor_lang::idl::types::IdlField {
                                    name: "data_header".into(),
                                    docs: Vec::new(),
                                    ty: anchor_lang::idl::types::IdlType::Defined {
                                        name: <DataHeader>::get_full_path(),
                                        generics: Vec::new(),
                                    },
                                },
                                anchor_lang::idl::types::IdlField {
                                    name: "user_id".into(),
                                    docs: Vec::new(),
                                    ty: anchor_lang::idl::types::IdlType::Pubkey,
                                },
                            ])),
                        )),
                    },
                ],
            },
        })
    }
    fn insert_types(
        types: &mut std::collections::BTreeMap<String, anchor_lang::idl::types::IdlTypeDef>,
    ) {
        if let Some(ty) = <AgentResponseV2>::create_type() {
            types.insert(<AgentResponseV2>::get_full_path(), ty);
            <AgentResponseV2>::insert_types(types);
        }
        if let Some(ty) = <DataHeader>::create_type() {
            types.insert(<DataHeader>::get_full_path(), ty);
            <DataHeader>::insert_types(types);
        }
    }
    fn get_full_path() -> String {
        let res = fmt::format(format_args!(
            "{0}::{1}",
            "sync_contract::types", "DataSubmissionV2"
        ));
        res
    }
}

impl borsh::ser::BorshSerialize for Datasubmission {
    fn serialize<W: borsh::maybestd::io::Write>(
        &self,
        writer: &mut W,
    ) -> core::result::Result<(), borsh::maybestd::io::Error> {
        match self {
            Datasubmission::V1(data_submission) => {
                writer.write(&[1])?;
                borsh::BorshSerialize::serialize(data_submission, writer)?;
            }
            Datasubmission::V2(data_submission) => {
                writer.write(&[2])?;
                borsh::BorshSerialize::serialize(data_submission, writer)?;
            }
        }
        Ok(())
    }
}
impl borsh::de::BorshDeserialize for Datasubmission {
    fn deserialize_reader<R: borsh::maybestd::io::Read>(
        reader: &mut R,
    ) -> core::result::Result<Self, borsh::maybestd::io::Error> {
        let version_byte = u8::deserialize_reader(reader)?;
        match version_byte {
            1 => Ok(Self::V1(DataSubmissionV1::deserialize_reader(reader)?)),
            2 | 0 => Ok(Self::V2(DataSubmissionV2::deserialize_reader(reader)?)),
            _ => Err(borsh::maybestd::io::Error::new(
                borsh::maybestd::io::ErrorKind::InvalidData,
                "Invalid version",
            )),
        }
    }
}

impl AccountSerialize for Datasubmission {
    fn try_serialize<W: std::io::Write>(&self, writer: &mut W) -> Result<()> {
        if writer.write_all(Datasubmission::DISCRIMINATOR).is_err() {
            return Err(ErrorCode::AccountDidNotSerialize.into());
        }
        if AnchorSerialize::serialize(self, writer).is_err() {
            return Err(ErrorCode::AccountDidNotSerialize.into());
        }
        Ok(())
    }
}

impl AccountDeserialize for Datasubmission {
    fn try_deserialize(buf: &mut &[u8]) -> Result<Self> {
        if buf.len() < Datasubmission::DISCRIMINATOR.len() {
            return Err(ErrorCode::AccountDiscriminatorNotFound.into());
        }
        let given_disc = &buf[..Datasubmission::DISCRIMINATOR.len()];
        if Datasubmission::DISCRIMINATOR != given_disc {
            return Err(anchor_lang::error::Error::from(AnchorError {
                error_name: ErrorCode::AccountDiscriminatorMismatch.name(),
                error_code_number: ErrorCode::AccountDiscriminatorMismatch.into(),
                error_msg: ErrorCode::AccountDiscriminatorMismatch.to_string(),
                error_origin: Some(ErrorOrigin::Source(Source {
                    filename: file!(),
                    line: 1,
                })),
                compared_values: None,
            })
            .with_account_name("Datasubmission"));
        }
        Self::try_deserialize_unchecked(buf)
    }
    fn try_deserialize_unchecked(buf: &mut &[u8]) -> Result<Self> {
        let mut data: &[u8] = &buf[Datasubmission::DISCRIMINATOR.len()..];
        AnchorDeserialize::deserialize(&mut data)
            .map_err(|_| ErrorCode::AccountDidNotDeserialize.into())
    }
}

impl Discriminator for Datasubmission {
    const DISCRIMINATOR: &'static [u8] = &[80, 43, 67, 220, 153, 253, 53, 244];
}

impl Owner for Datasubmission {
    fn owner() -> Pubkey {
        crate::ID
    }
}

impl Datasubmission {
    pub const MAX_SIZE: usize = 1 + DataSubmissionV2::MAX_SIZE;

    pub fn is_rated(&self) -> bool {
        match self {
            Datasubmission::V1(data_submission) => data_submission.agent_response.is_some(),
            Datasubmission::V2(data_submission) => data_submission.agent_response.is_some(),
        }
    }

    pub fn user_id(&self) -> Pubkey {
        match self {
            Datasubmission::V1(data_submission) => data_submission.user_id,
            Datasubmission::V2(data_submission) => data_submission.user_id,
        }
    }
}

#[account]
pub struct ProgramState {
    pub version: u8,
    pub admin: Pubkey,
}

impl ProgramState {
    pub const MAX_SIZE: usize = 1 + 32;
}

#[derive(Eq, PartialEq, Debug)]
#[account]
pub struct AgentConfig {
    pub version: u8,
    pub is_enabled: bool,
}

impl AgentConfig {
    pub const MAX_SIZE: usize = 2;
}

#[derive(Eq, PartialEq, Debug)]
#[account]
pub struct UserConfig {
    pub version: u8,
    pub accumulated_credits: u128,
    pub reserved: [u8; USER_CONFIG_RESERVED_SIZE],
}

impl UserConfig {
    pub const MAX_SIZE: usize = 1 + 16 + USER_CONFIG_RESERVED_SIZE;
}
