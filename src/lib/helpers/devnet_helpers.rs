use ellipsis_client::EllipsisClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signer::Signer;

pub mod devnet_token_faucet {
    use borsh::{BorshDeserialize, BorshSerialize};
    use solana_sdk::{
        declare_id,
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
    };

    const CREAT_MINT_DISCRIMINATOR: [u8; 8] = [69, 44, 215, 132, 253, 214, 41, 45];
    const AIRDROP_SPL_DISCRIMINATOR: [u8; 8] = [133, 44, 125, 96, 172, 219, 228, 51];

    declare_id!("FF2UnZt7Lce3S65tW5cMVKz8iVAPoCS8ETavmUhsWLJB");

    #[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
    pub struct CreateMint {
        pub ticker: String,
        pub decimals: u8,
    }

    pub fn get_mint_address(ticker: &str) -> Pubkey {
        Pubkey::find_program_address(&["mint".as_bytes(), ticker.to_lowercase().as_ref()], &ID).0
    }

    pub fn get_mint_authority_address(ticker: &str) -> Pubkey {
        Pubkey::find_program_address(
            &["mint-authority".as_bytes(), ticker.to_lowercase().as_ref()],
            &ID,
        )
        .0
    }

    pub fn create_mint_ix(
        program_id: Pubkey,
        payer: Pubkey,
        ticker: String,
        decimals: u8,
    ) -> Instruction {
        let (mint, _) = Pubkey::find_program_address(
            &["mint".as_bytes(), ticker.to_lowercase().as_ref()],
            &program_id,
        );

        let (mint_authority, _) = Pubkey::find_program_address(
            &["mint-authority".as_bytes(), ticker.to_lowercase().as_ref()],
            &program_id,
        );

        let accounts_vec: Vec<AccountMeta> = vec![
            AccountMeta::new(mint, false),
            AccountMeta::new(mint_authority, false),
            AccountMeta::new(payer, true),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
            AccountMeta::new_readonly(solana_sdk::sysvar::rent::id(), false),
        ];

        let create_mint_data = CreateMint { ticker, decimals };

        let ix_data: Vec<u8> = [
            CREAT_MINT_DISCRIMINATOR,
            create_mint_data.try_to_vec().unwrap().try_into().unwrap(),
        ]
        .concat();

        Instruction {
            program_id,
            accounts: accounts_vec,
            data: ix_data,
        }
    }

    #[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
    pub struct AirdropSpl {
        pub amount: u64,
    }

    pub fn airdrop_spl_with_ticker_ix(
        program_id: &Pubkey,
        ticker: String,
        payer: &Pubkey,
        amount: u64,
    ) -> Instruction {
        let mint = get_mint_address(&ticker);
        let mint_authority = get_mint_authority_address(&ticker);

        let destination = spl_associated_token_account::get_associated_token_address(payer, &mint);

        let accounts_vec: Vec<AccountMeta> = vec![
            AccountMeta::new(mint, false),
            AccountMeta::new(mint_authority, false),
            AccountMeta::new(destination, false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ];

        let airdrop_spl_data = AirdropSpl { amount };

        let ix_data: Vec<u8> = [
            AIRDROP_SPL_DISCRIMINATOR,
            airdrop_spl_data.try_to_vec().unwrap().try_into().unwrap(),
        ]
        .concat();

        Instruction {
            program_id: *program_id,
            accounts: accounts_vec,
            data: ix_data,
        }
    }

    pub fn airdrop_spl_with_mint_pdas_ix(
        program_id: &Pubkey,
        mint: &Pubkey,
        mint_authority: &Pubkey,
        payer: &Pubkey,
        amount: u64,
    ) -> Instruction {
        let destination = spl_associated_token_account::get_associated_token_address(payer, mint);

        let accounts_vec: Vec<AccountMeta> = vec![
            AccountMeta::new(*mint, false),
            AccountMeta::new(*mint_authority, false),
            AccountMeta::new(destination, false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ];

        let airdrop_spl_data = AirdropSpl { amount };

        let ix_data: Vec<u8> = [
            AIRDROP_SPL_DISCRIMINATOR,
            airdrop_spl_data.try_to_vec().unwrap().try_into().unwrap(),
        ]
        .concat();

        Instruction {
            program_id: *program_id,
            accounts: accounts_vec,
            data: ix_data,
        }
    }
}

// Create_mint on devnet, utilizing devnet-token-faucet
pub async fn find_or_create_devnet_mint(
    client: &EllipsisClient,
    ticker: &str,
    decimals: u8,
) -> anyhow::Result<Pubkey> {
    let (mint, _) = Pubkey::find_program_address(
        &["mint".as_bytes(), ticker.to_lowercase().as_ref()],
        &devnet_token_faucet::ID,
    );
    if client.get_account(&mint).await.is_err() {
        let mint_ix = devnet_token_faucet::create_mint_ix(
            devnet_token_faucet::ID,
            client.payer.pubkey(),
            ticker.to_string(),
            decimals,
        );
        client
            .sign_send_instructions(vec![mint_ix], vec![&client.payer])
            .await?;
    }
    Ok(mint)
}
