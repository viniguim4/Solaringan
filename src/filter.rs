use solana_transaction_status::TransactionWithStatusMeta;
use anyhow::Result;

const RD_AUTHORITY : &'static str = "5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1";
const WSOL : &'static str = "So11111111111111111111111111111111111111112";

#[derive(Debug)]
pub enum RaydiumType {
    Buy,
    Sell,
    AddLiquidity,
    RemoveLiquidity,
    Unknown
}

impl RaydiumType {
    pub fn get_type(tx : &TransactionWithStatusMeta)-> Result<Self>{
        let meta = match tx.clone() {
            TransactionWithStatusMeta::Complete(tx_inner) => {
                tx_inner.meta
            }
            TransactionWithStatusMeta::MissingMetadata(tx_inner) => {
                return Err(anyhow::anyhow!("MissingMetadata"));
            }
        };

        let pre_balances = match meta.pre_token_balances {
            Some(pre_balances) => pre_balances,
            None => return Err(anyhow::anyhow!("Pre balances not found"))
        };
        let post_balances = match meta.post_token_balances {
            Some(post_balances) => post_balances,
            None => return Err(anyhow::anyhow!("Post balances not found"))
        };

        let mut mint_target_token = "";
        let (mut post_sol_balances, mut pre_sol_balances) = (0, 0);
        let (mut post_token_balances, mut pre_token_balances) = (0, 0);
        for account in &pre_balances{
            if mint_target_token != "" && pre_sol_balances != 0 && pre_token_balances != 0 {
                break;
            }
            if account.owner == RD_AUTHORITY && account.mint != WSOL {
                mint_target_token = &account.mint;
            }
            if account.owner == RD_AUTHORITY && account.mint == WSOL {
                let amount = account.ui_token_amount.amount.parse::<u64>()
                    .map_err(|_| anyhow::anyhow!("Invalid amount format"))?;
                pre_sol_balances = amount;
            }
            if account.owner == RD_AUTHORITY && account.mint != WSOL {
                let amount = account.ui_token_amount.amount.parse::<u64>()
                    .map_err(|_| anyhow::anyhow!("Invalid amount format"))?;
                pre_token_balances = amount;
            }
        }

        for account in post_balances{
            if post_sol_balances != 0 && post_token_balances != 0 {
                break;
            }
            if account.owner == RD_AUTHORITY && account.mint != WSOL {
                if account.mint != mint_target_token {
                    return Err(anyhow::anyhow!("Mint not equal"));
                }
            }
            if account.owner == RD_AUTHORITY && account.mint == WSOL {
                let amount = account.ui_token_amount.amount.parse::<u64>()
                    .map_err(|_| anyhow::anyhow!("Invalid amount format"))?;
                post_sol_balances = amount;
            }
            if account.owner == RD_AUTHORITY && account.mint != WSOL {
                let amount = account.ui_token_amount.amount.parse::<u64>()
                    .map_err(|_| anyhow::anyhow!("Invalid amount format"))?;
                post_token_balances = amount;
            }
        }

        if mint_target_token   == "" ||
           pre_sol_balances    == 0  ||
           pre_token_balances  == 0  ||
           post_sol_balances   == 0  ||
           post_token_balances == 0 {
            return Err(anyhow::anyhow!("Failed to indentify transaction type"));
        }

        if (post_sol_balances > pre_sol_balances) && (post_token_balances > pre_token_balances) {
            return Ok(RaydiumType::AddLiquidity);
        } else if (post_sol_balances < pre_sol_balances) && (post_token_balances < pre_token_balances) {
            return Ok(RaydiumType::RemoveLiquidity);
        } else if (post_sol_balances > pre_sol_balances) && (post_token_balances < pre_token_balances) {
            return Ok(RaydiumType::Buy);
        } else if (post_sol_balances < pre_sol_balances) && (post_token_balances > pre_token_balances) {
            return Ok(RaydiumType::Sell);
        } else {
            return Ok(RaydiumType::Unknown);
        }
    }
}