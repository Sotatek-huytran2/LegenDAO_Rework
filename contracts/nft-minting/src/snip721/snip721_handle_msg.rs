use cosmwasm_std::HumanAddr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::snip721::metadata::Metadata;
use crate::state::{TokenType};
use super::royalties::RoyaltyInfo;

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    EnableReveal {},
    /// mint new token
    MintNft {
        /// optional token id. if omitted, use current token index
        token_id: Option<String>,
        /// optional owner address. if omitted, owned by the message sender
        owner: Option<HumanAddr>,
        /// optional public metadata that can be seen by everyone
        public_metadata: Option<Metadata>,
        /// optional private metadata that can only be seen by the owner and whitelist
        private_metadata: Option<Metadata>,
        /// optional serial number for this token
        serial_number: Option<SerialNumber>,
        /// optional royalty information for this token
        royalty_info: Option<RoyaltyInfo>,
        /// optional token type
        token_type: Option<u8>,
        /// optional memo for the tx
        memo: Option<String>,
        /// optional message length padding
        padding: Option<String>,
    },
    /// Mint multiple tokens
    BatchMintNft {
        /// list of mint operations to perform
        mints: Vec<Mint>,
        /// optional message length padding
        padding: Option<String>,
    },
}

/// Serial number to give an NFT when minting
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct SerialNumber {
    /// optional number of the mint run this token will be minted in.  A mint run represents a
    /// batch of NFTs released at the same time.  So if a creator decided to make 100 copies
    /// of an NFT, they would all be part of mint run number 1.  If they sold quickly, and
    /// the creator wanted to rerelease that NFT, he could make 100 more copies which would all
    /// be part of mint run number 2.
    pub mint_run: Option<u32>,
    /// serial number (in this mint run).  This is used to serialize
    /// identical NFTs
    pub serial_number: u32,
    /// optional total number of NFTs minted on this run.  This is used to
    /// represent that this token is number m of n
    pub quantity_minted_this_run: Option<u32>,
}

/// token mint info used when doing a BatchMint
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct Mint {
    /// optional token id, if omitted, use current token index
    pub token_id: Option<String>,
    /// optional owner address, owned by the minter otherwise
    pub owner: Option<HumanAddr>,
    /// optional public metadata that can be seen by everyone
    pub public_metadata: Option<Metadata>,
    /// optional private metadata that can only be seen by owner and whitelist
    pub private_metadata: Option<Metadata>,
    /// optional serial number for this token
    pub serial_number: Option<SerialNumber>,
    /// optional royalty info for this token
    pub royalty_info: Option<RoyaltyInfo>,
    /// optional token type
    pub token_type: Option<u8>,
    /// optional memo for the tx
    pub memo: Option<String>,
}
