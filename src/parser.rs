use serde_json::{Value, json};
use log::{info};
use solana_transaction_status::TransactionWithStatusMeta;
use anyhow::{Result};
use solana_sdk::{message::v0::LoadedAddresses, instruction::{Instruction, AccountMeta, CompiledInstruction}};
use crate::utils::{raydium_parser::*, pumpfun_parser::*};
use crate::filter::{RaydiumType, PumpType};

pub fn decode_raydium_txn(tx: &TransactionWithStatusMeta) -> Result<Value> {
    let allIxs = flatten_transaction_response(&tx.clone())?;

    let raydiumIxs = allIxs.into_iter().filter(|ix| 
        ix.program_id == RAYDIUM_AMM_V4_PROGRAM_ID
    );

    let mut decodedIxs = vec![];
    for ix in raydiumIxs {
        decodedIxs.push(RaydiumAmmParser::parse_instruction(&ix)?);
    }

    let raydiumType = RaydiumType::get_type(&tx.clone())?;
    info!("RAYDIUM {:?} DECODED {:#?}", raydiumType, decodedIxs);

    Ok(json!({}))
}

pub fn decode_pumpfun_txn(tx: &TransactionWithStatusMeta) -> Result<Value> {
    let allIxs = flatten_transaction_response(tx)?;

    let pumpfunIxs = allIxs.into_iter().filter(|ix| 
        ix.program_id == PUMPFUN_PROGRAM_ID
    );

    let mut decodedIxs = vec![];
    for ix in pumpfunIxs {
        decodedIxs.push(PumpfunParser::parse_instruction(&ix)?);
    }

    let pumpType = PumpType::get_type(&decodedIxs.clone())?;
    info!("PUMP DECODED {:#?}", pumpType);

    Ok(json!({}))
}

pub fn flatten_transaction_response(tx: &TransactionWithStatusMeta) -> Result<Vec<Instruction>> {
    let mut result = vec![];
    
    let (message, mut meta) = match tx.clone() {
        TransactionWithStatusMeta::Complete(tx_inner) => {
            (tx_inner.transaction.message, Some(tx_inner.meta))
        }
        TransactionWithStatusMeta::MissingMetadata(tx_inner) => {
            return Err(anyhow::anyhow!("MissingMetadata"));
        }
    };

    let compiled_instructions = match &message {
        solana_sdk::message::VersionedMessage::Legacy(message) => &message.instructions,
        solana_sdk::message::VersionedMessage::V0(message) => &message.instructions,
    };

    let (loaded_addresses, mut inner_instructions) = match &mut meta {
        Some(meta) => {
            match &mut meta.inner_instructions {
                Some(ixs) => {
                    (Some(&meta.loaded_addresses), Some(ixs))
                }
                None => {
                    (Some(&meta.loaded_addresses), None)
                }
            }
        }
        None => (None, None)
    };

    let accounts_meta = parse_transaction_accounts(
        &message,
        loaded_addresses
    );
    /*const orderedCII = (transaction?.meta?.innerInstructions || []).sort(
      (a, b) => a.index - b.index,
    );*/
    // TODO: sort inner instructions
    let ordered_cii = match &mut inner_instructions { 
        Some(ixs) => {
            ixs.sort_by_key(|cii| cii.index);
            ixs
        },
        None => &vec![]
    };

    //info!("ordered_cii: {:#?}", ordered_cii);

    let total_calls = ordered_cii.iter().fold(compiled_instructions.len(), |acc, cii| acc + cii.instructions.len());
    let mut last_pushed_ix = 0;
    let mut call_index = 0;

    for cii in ordered_cii {
        while last_pushed_ix != cii.index {
            call_index += 1;
            result.push(
                compiled_instruction_to_instruction(&compiled_instructions[last_pushed_ix as usize], &accounts_meta)?
            );
            last_pushed_ix += 1;
        }
        for cii_entry in &cii.instructions {
            call_index += 1;
            result.push(
                compiled_instruction_to_instruction(&cii_entry.instruction, &accounts_meta)?
            );
        }
    }
    while call_index < total_calls {
        call_index += 1;
        result.push(
            compiled_instruction_to_instruction(&compiled_instructions[last_pushed_ix as usize], &accounts_meta)?
        );
        last_pushed_ix += 1;
    }
        
    Ok(result)
}

fn parse_transaction_accounts(
    message: &solana_sdk::message::VersionedMessage,
    loaded_addresses: Option<&LoadedAddresses>,
) -> Vec<AccountMeta>
{   
    let accounts = match &message {
        solana_sdk::message::VersionedMessage::Legacy(message) => &message.account_keys,
        solana_sdk::message::VersionedMessage::V0(message) => &message.account_keys,
    };
    let readonly_signed_accounts_count = message.header().num_readonly_signed_accounts as usize;
    let readonly_unsigned_accounts_count = message.header().num_readonly_unsigned_accounts as usize;
    let required_signatures_accounts_count = message.header().num_required_signatures as usize;
    let total_accounts = accounts.len();

    let mut parsed_accounts: Vec<AccountMeta> = accounts
        .iter()
        .enumerate()
        .map(|(idx, account)| {
            let is_writable = idx < required_signatures_accounts_count - readonly_signed_accounts_count
                || (idx >= required_signatures_accounts_count
                    && idx < total_accounts - readonly_unsigned_accounts_count);

            AccountMeta {
                is_signer: idx < required_signatures_accounts_count,
                is_writable,
                pubkey: account.clone(),
            }
        })
        .collect();

    if let Some(loaded_addresses) = loaded_addresses {
        parsed_accounts.extend(loaded_addresses.writable.iter().map(|pubkey| AccountMeta {
            is_signer: false,
            is_writable: true,
            pubkey: pubkey.clone(),
        }));
        parsed_accounts.extend(loaded_addresses.readonly.iter().map(|pubkey| AccountMeta {
            is_signer: false,
            is_writable: false,
            pubkey: pubkey.clone(),
        }));
    }

    //info!("Parsed accounts: {:#?}", parsed_accounts);
    parsed_accounts
}

fn compiled_instruction_to_instruction(
    compiled_instruction: &CompiledInstruction,
    parsed_accounts: &[AccountMeta],
) -> Result<Instruction> {
    let program_id = match parsed_accounts.get(compiled_instruction.program_id_index as usize) {
            Some(meta) => meta.pubkey,
            None => return Err(anyhow::anyhow!("Program ID account out of bounds")),
    };

    let data = compiled_instruction.data.clone();

    let accounts = compiled_instruction.accounts
        .iter()
        .map(|idx| {
            parsed_accounts.get(*idx as usize)
                .ok_or_else(|| anyhow::anyhow!("Account index out of bounds"))
                .cloned()
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Instruction {
        program_id,
        accounts,
        data,
    })
}