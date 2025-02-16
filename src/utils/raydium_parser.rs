use solana_sdk::{pubkey, pubkey::Pubkey, instruction::Instruction, instruction::AccountMeta};
use anyhow::Result;
use borsh::{BorshDeserialize, BorshSerialize};

pub const RAYDIUM_AMM_V4_PROGRAM_ID: Pubkey = pubkey!("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8");

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Copy)]
pub struct SwapBaseIn {
    pub amount_in: u64,
    pub minimum_out: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Copy)]
pub struct SwapBaseOut {
    pub max_in: u64,
    pub amount_out: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Copy)]
pub struct RaydiumInitializeArgs {
    pub nonce: u8,
    pub open_time: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Copy)]
pub struct RaydiumInitialize2Args {
    pub nonce: u8,
    pub open_time: u64,
    pub init_pc_amount: u64,
    pub init_coin_amount: u64,
}

pub struct RaydiumAmmParser;

impl RaydiumAmmParser {

    pub fn parse_instruction(instruction: &Instruction) -> Result<RaydiumInstruction> {
        let data = instruction.data.as_slice();
        let instruction_type = match data.first(){
            Some(opcode) => opcode,
            None => return Err(anyhow::anyhow!("Invalid OPCODE data {:?}", data)),
        };
        
        match instruction_type {
            0 => Self::parse_initialize(instruction),
            1 => Self::parse_initialize2(instruction),
            9 => Self::parse_swap_in(instruction),
            11 => Self::parse_swap_out(instruction),
            _ => Err(anyhow::anyhow!("Irrelevant instruction type")),
        }
    }

    fn parse_swap_in(instruction: &Instruction) -> Result<RaydiumInstruction> {
        let data = instruction.data.as_slice();
        let args = SwapBaseIn::try_from_slice(&data[1..])
            .map_err(|_| anyhow::anyhow!("Invalid SwapIn instruction data {:?}", data))?;

        Ok(RaydiumInstruction::SwapIn(args, instruction.accounts.clone()))
    }

    fn parse_swap_out(instruction: &Instruction) -> Result<RaydiumInstruction> {
        let data = instruction.data.as_slice();
        let args = SwapBaseOut::try_from_slice(&data[1..])
            .map_err(|_| anyhow::anyhow!("Invalid SwapOut instruction data {:?}", data))?;

        Ok(RaydiumInstruction::SwapOut(args, instruction.accounts.clone()))
    }

    fn parse_initialize(instruction: &Instruction) -> Result<RaydiumInstruction> {
        let data = instruction.data.as_slice();
        let args = RaydiumInitializeArgs::try_from_slice(&data[1..])
            .map_err(|_| anyhow::anyhow!("Invalid Initialize instruction data {:?}", data))?;

        Ok(RaydiumInstruction::Initialize (args, instruction.accounts.clone()))
    }

    fn parse_initialize2(instruction: &Instruction) -> Result<RaydiumInstruction> {
        let data = instruction.data.as_slice();
        let args = RaydiumInitialize2Args::try_from_slice(&data[1..])
            .map_err(|_| anyhow::anyhow!("Invalid Initialize2 instruction data {:?}", data))?;

        Ok(RaydiumInstruction::Initialize2(args, instruction.accounts.clone()))
    }
}

#[derive(Debug)]
pub enum RaydiumInstruction {
    Initialize(RaydiumInitializeArgs, Vec<AccountMeta >), 
    Initialize2(RaydiumInitialize2Args, Vec<AccountMeta >),
    SwapIn(SwapBaseIn, Vec<AccountMeta >),
    SwapOut(SwapBaseOut, Vec<AccountMeta >),
}

