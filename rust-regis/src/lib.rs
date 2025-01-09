mod programs;
#[cfg(test)]
mod tests {
    use std::io::{ self, BufRead };
    use solana_sdk::{
        bs58,
        message::Message,
        pubkey::Pubkey,
        signature::{ read_keypair_file, Keypair, Signer },
        system_instruction::transfer,
        system_program,
        transaction::Transaction,
    };
    use crate::programs::turbin3_prereq::{ WbaPrereqProgram, CompleteArgs };
    use solana_client::rpc_client::RpcClient;
    use std::str::FromStr;
    const RPC_URL: &str = "https://api.devnet.solana.com";
    #[test]
    fn keygen() {
        let keypair = Keypair::new();
        println!("You've generated a new keypair: {:?}", keypair.pubkey());
        println!("To save your wallet, copy and paste the following into a JSON file:");
        println!("{:?}", keypair.to_bytes());
    }
    #[test]
    fn base58_to_wallet() {
        println!("Input your private key as base58:");
        let stdin = io::stdin();
        let base58 = stdin.lock().lines().next().unwrap().unwrap();
        println!("Your wallet file is:");
        let wallet = bs58::decode(base58).into_vec().unwrap();
        println!("{:?}", wallet);
    }
    #[test]
    fn wallet_to_base58() {
        println!("Input your private key as a wallet file byte array:");
        let stdin = io::stdin();
        let wallet = stdin
            .lock()
            .lines()
            .next()
            .unwrap()
            .unwrap()
            .trim_start_matches('[')
            .trim_end_matches(']')
            .split(',')
            .map(|s| s.trim().parse::<u8>().unwrap())
            .collect::<Vec<u8>>();

        println!("Your private key is:");
        let base58 = bs58::encode(wallet).into_string();
        println!("{:?}", base58);
    }
    #[test]
    fn airdop() {
        let kp = read_keypair_file("dev-wallet.json").expect("Could not read keypair file");
        let client = RpcClient::new(RPC_URL);
        match client.request_airdrop(&kp.pubkey(), 2_000_000_000u64) {
            Ok(s) => {
                println!("Airdrop successful: {:?}", s);
                println!("https://explorer.solana.com/tx/{}?cluster=devnet", s.to_string());
            }
            Err(e) => println!("Airdrop failed: {:?}", e),
        }
    }
    #[test]
    fn transfer_sol() {
        let keypair = read_keypair_file("dev-wallet.json").expect("Couldn't find wallet file");
        let to_pubkey = Pubkey::from_str("4tMN5HYmfpsAFgcxG2Ng14pfJwoy8f4Kz2V6n8tgPyim").unwrap();
        let client: RpcClient = RpcClient::new(RPC_URL);
        let balance = client.get_balance(&keypair.pubkey()).expect("Failed to get balance");
        let recent_blockhash = client
            .get_latest_blockhash()
            .expect("Failed to get recent blockhash");
        let message = Message::new_with_blockhash(
            &[transfer(&keypair.pubkey(), &to_pubkey, balance)],
            Some(&keypair.pubkey()),
            &recent_blockhash
        );
        let fee = client.get_fee_for_message(&message).expect("Failed to get fee");
        let tx = Transaction::new_signed_with_payer(
            &[transfer(&keypair.pubkey(), &to_pubkey, balance - fee)],
            Some(&keypair.pubkey()),
            &vec![&keypair],
            recent_blockhash
        );
        let sign = client.send_and_confirm_transaction(&tx).expect("Failed to send transaction");
        println!("Success! Check out your TX here: https://explorer.solana.com/tx/{}/?cluster=devnet", sign);
    }
    #[test]
    fn enroll() {
        let rpc_client = RpcClient::new(RPC_URL);
        let signer = read_keypair_file("Turbin3-wallet.json").expect("Could not read keypair file");
        let prereq = WbaPrereqProgram::derive_program_address(
            &[b"prereq", signer.pubkey().to_bytes().as_ref()]
        );
        let args = CompleteArgs {
            github: b"dorkydhruv".to_vec(),
        };
        let blockhash = rpc_client.get_latest_blockhash().expect("Failed to get recent blockhash");
        let transaction = WbaPrereqProgram::complete(
            &[&signer.pubkey(), &prereq, &system_program::id()],
            &args,
            Some(&signer.pubkey()),
            &[&signer],

            blockhash
        );
        let signature = rpc_client
            .send_and_confirm_transaction(&transaction)
            .expect("Failed to send transaction");
        println!("Success! Check out your TX here: https://explorer.solana.com/tx/{}/?cluster=devnet", signature);
    }
}
