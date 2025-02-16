use solana_transaction_status::TransactionWithStatusMeta;
use anyhow::Result;

const RD_AUTHORITY : &'static str = "5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1";
const WSOL : &'static str = "So11111111111111111111111111111111111111112";

#[derive(Debug)]
pub enum RaydiumType {
    Buy(TradeSize),
    Sell(TradeSize),
    AddLiquidity(LiquiditySize),
    RemoveLiquidity(LiquiditySize),
    Unknown
}

#[derive(Debug)]
pub struct TradeSize {
    reserve_in: u64,
    reserve_out: u64,
    amount_in: u64,
    amount_out : u64,
    price_impact: f64
}
#[derive(Debug)]
pub struct LiquiditySize {
    sol_reserve : u64,
    token_reserve : u64
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
        let sol_reserve = post_sol_balances - pre_sol_balances;
        let token_reserve = post_token_balances - pre_token_balances;
        if (post_sol_balances > pre_sol_balances) && (post_token_balances > pre_token_balances) {
            return Ok(RaydiumType::AddLiquidity(LiquiditySize{sol_reserve, token_reserve}));
        } else if (post_sol_balances < pre_sol_balances) && (post_token_balances < pre_token_balances) {
            return Ok(RaydiumType::RemoveLiquidity(LiquiditySize{sol_reserve, token_reserve}));
        } else if (post_sol_balances > pre_sol_balances) && (post_token_balances < pre_token_balances) {
            let price_impact_sqrt = post_sol_balances as f64/pre_sol_balances as f64;
            let price_impact = (price_impact_sqrt * price_impact_sqrt) - 1.0; 
            let amount_in = post_sol_balances - pre_sol_balances;
            let amount_out = pre_token_balances - post_token_balances;
            return Ok(RaydiumType::Buy(TradeSize{reserve_in : post_sol_balances, reserve_out :post_token_balances, amount_in, amount_out, price_impact}));
        } else if (post_sol_balances < pre_sol_balances) && (post_token_balances > pre_token_balances) {
            let price_impact_sqrt = pre_token_balances as f64/ post_token_balances as f64;
            let price_impact = 1.0 - (price_impact_sqrt * price_impact_sqrt);
            let amount_in = post_token_balances - pre_token_balances;
            let amount_out = pre_sol_balances - post_sol_balances;
            return Ok(RaydiumType::Sell(TradeSize{reserve_in : post_token_balances, reserve_out : post_sol_balances, amount_in, amount_out, price_impact}));
        } else {
            return Ok(RaydiumType::Unknown);
        }
    }
}

#[derive(Debug)]
pub enum PumpType {
    Buy(TradeSizeWithVirtual),
    Sell(TradeSizeWithVirtual),
}

#[derive(Debug)]
pub struct TradeSizeWithVirtual {
    reserve_in_virtual: u64,
    reserve_out_virtual: u64,
    reserve_in: u64,
    reserve_out: u64,
    amount_in: u64,
    amount_out : u64,
    price_impact: f64
}

impl PumpType {
// TODO unfineshed
    pub fn get_type(ixs : &Vec<PumpfunInstruction>) -> PumpType {
        let decodedBuySells = vec![];
        let decodedCpi = vec![];
        for ix in ixs {
            match ix {
                PumpfunInstruction::Buy => decodedBuySells.push(ix),
                PumpfunInstruction::Sell => decodedBuySells.push(ix),
                PumpfunInstruction::Cpi => decodedCpi.push(ix),
            }
        }
    }
}