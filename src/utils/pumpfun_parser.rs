use solana_sdk::{pubkey, pubkey::Pubkey, instruction::Instruction, instruction::AccountMeta};
use borsh::{BorshDeserialize, BorshSerialize};
use anyhow::Result;

pub const PUMPFUN_PROGRAM_ID: Pubkey = pubkey!("6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P");

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct InitializeArgs {}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct SetParamsArgs {
    pub fee_recipient: [u8; 32],
    pub initial_virtual_token_reserves: u64,
    pub initial_virtual_sol_reserves: u64,
    pub initial_real_token_reserves: u64,
    pub token_total_supply: u64,
    pub fee_basis_points: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct CreateArgs {
    pub name: String,
    pub symbol: String,
    pub uri: String,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct BuyArgs {
    pub amount: u64,
    pub max_sol_cost: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct SellArgs {
    pub amount: u64,
    pub min_sol_output: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Default)]
pub struct CPILog {
    pub blob : u64,      //8
    pub mint_address: [u8; 32],  //32 
    pub user_sol: u64,  //4
    pub user_token: u64, //4 
    pub is_buy: bool,   // 1
    pub user_addrs: [u8; 32],  // 32
    pub timestamp: i64,  //4
    pub virtual_sol_reserves: u64,  //4
    pub virtual_token_reserves: u64, //4
    pub real_sol_reserves: u64,   //4
    pub real_token_reserves: u64,   //4
}

#[derive(Debug, Clone)]
pub enum PumpfunInstruction {
    Initialize,
    SetParams(SetParamsArgs, Vec<AccountMeta>),
    Create(CreateArgs, Vec<AccountMeta>),
    Buy(BuyArgs, Vec<AccountMeta>),
    Sell(SellArgs, Vec<AccountMeta>),
    Withdraw,
    CPILog(CPILog, Vec<AccountMeta>),
}

pub struct PumpfunParser;

impl PumpfunParser {
        pub fn parse_instruction(ix: &Instruction) -> Result<PumpfunInstruction> {
        let discriminator = match ix.data.get(0..8) {
            Some(opcode) => opcode,
            None => return Err(anyhow::anyhow!("Invalid instruction data")),
        };


        match discriminator {
            [175, 175, 109, 31, 13, 152, 155, 237] => Ok(PumpfunInstruction::Initialize),
            [165, 31, 134, 53, 189, 180, 130, 255] => {
                let args = SetParamsArgs::try_from_slice(&ix.data[8..])
                    .map_err(|_| anyhow::anyhow!("Invalid SetParams instruction data {:?}", ix.data))?;
                Ok(PumpfunInstruction::SetParams(args, ix.accounts.clone()))
            },
            [24, 30, 200, 40, 5, 28, 7, 119] => {
                let args = CreateArgs::try_from_slice(&ix.data[8..])
                    .map_err(|_| anyhow::anyhow!("Invalid Create instruction data {:?}", ix.data))?;
                Ok(PumpfunInstruction::Create(args, ix.accounts.clone()))
            },
            [102, 6, 61, 18, 1, 218, 235, 234] => {
                let args = BuyArgs::try_from_slice(&ix.data[8..])
                    .map_err(|_| anyhow::anyhow!("Invalid Buy instruction data {:?}", ix.data))?;
                Ok(PumpfunInstruction::Buy(args, ix.accounts.clone()))
            },
            [51, 230, 133, 164, 1, 127, 131, 173] => {
                let args = SellArgs::try_from_slice(&ix.data[8..])
                    .map_err(|_| anyhow::anyhow!("Invalid Sell instruction data {:?}", ix.data))?;
                Ok(PumpfunInstruction::Sell(args, ix.accounts.clone()))
            },
            [183, 18, 70, 156, 148, 109, 161, 34] => Ok(PumpfunInstruction::Withdraw),

            [228, 69, 165, 46, 81, 203, 154, 29] => {
                let args = CPILog::try_from_slice(&ix.data[8..])
                    .map_err(|_| anyhow::anyhow!("Invalid CPI instruction data {:?}", ix.data))?;
                Ok(PumpfunInstruction::CPILog(args, ix.accounts.clone()))
            }
            _ => Err(anyhow::anyhow!("Unknown instruction type: {:?}", discriminator)),
        }
    }

}
