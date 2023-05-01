// TODO test sending a transaction to pk_AB from miner
// TODO test spending a transaction from pk_AB to bob

use std::collections::HashSet;
use std::{collections::HashMap, str::FromStr};

use curve25519_dalek::constants::ED25519_BASEPOINT_TABLE;
use curve25519_dalek::edwards::{CompressedEdwardsY, EdwardsPoint};
use curve25519_dalek::scalar::Scalar;

use monero_serai::random_scalar;
use monero_serai::ringct::generate_key_image;
use monero_serai::rpc::Rpc;
use monero_serai::transaction::{Input, Transaction};
use monero_serai::wallet::address::{AddressSpec, MoneroAddress, Network};
use monero_serai::wallet::extra::Extra;
use monero_serai::wallet::{
    Change, ReceivedOutput, Scanner, SignableTransaction, SignableTransactionBuilder,
    SpendableOutput, Timelocked, ViewPair,
};

use core::ops::Deref;
use paymo::core::utils::{generate_user_key_pair, generate_user_tag};
use rand_core::OsRng;
use sha3::{Digest, Keccak256};
use zeroize::Zeroizing;

const MINER_VIEW_PRIVATE_KEY: &str =
    "e89b33d2af4b7de82ffe63db5e70d931cc87b1f230e089506ac127408746eb0c";
const MINER_SPEND_PRIVATE_KEY: &str =
    "608b200321973bb720910233672288bf730d29b6e1a9a750829d1bebd6c90d0a";

const BOB_MAIN_ADDRESS: &str = "41pkzuEmAKDEY8jyH7oq3Yc2XAEP5Ci43b7sSYgeR91UKFTu8hv5BX5PSuq3zAw1de2Qdrv8NyA69ag38oTSwhawSWRGJyZ";

const XMR_AMOUNT_1: &str = "10 xmr";

#[tokio::main]
async fn main() {
    // ------- JOINT GENERATION ---------
    let (alice_view_secret, alice_view_public) = generate_user_key_pair();
    let (alice_spend_secret, alice_spend_public) = generate_user_key_pair();

    let (bob_view_secret, bob_view_public) = generate_user_key_pair();
    let (bob_spend_secret, bob_spend_public) = generate_user_key_pair();

    let joint_view_private = alice_view_secret + bob_view_secret;
    let joint_spend_private = alice_spend_secret + bob_spend_secret;

    let joint_view_public = alice_view_public + bob_view_public;
    let joint_spend_public = alice_spend_public + bob_spend_public;

    // TODO generate tag when spending from shared address

    println!(
        "Joint view private is: {}",
        hex::encode(joint_view_private.as_bytes())
    );
    println!(
        "Joint view public is: {}",
        hex::encode(joint_view_public.compress().as_bytes())
    );
    println!(
        "Joint spend public is: {}",
        hex::encode(joint_spend_public.compress().as_bytes())
    );

    println!();

    let joint_view_pair = ViewPair::new(joint_spend_public, Zeroizing::new(joint_view_private));
    let joint_address = joint_view_pair.address(Network::Mainnet, AddressSpec::Standard);

    println!("Joint address is: {}", joint_address.to_string());

    // ------- MINER VIEW PAIR ---------
    let miner_view_private_key: [u8; 32] = hex::decode(MINER_VIEW_PRIVATE_KEY)
        .unwrap()
        .try_into()
        .unwrap();
    let miner_view_private_key = Scalar::from_bits(miner_view_private_key);

    let miner_spend_private_key: [u8; 32] = hex::decode(MINER_SPEND_PRIVATE_KEY)
        .unwrap()
        .try_into()
        .unwrap();
    let miner_spend_private_key = Scalar::from_bits(miner_spend_private_key);
    let miner_spend_public_key = &miner_spend_private_key * &ED25519_BASEPOINT_TABLE;

    let miner_view_pair = ViewPair::new(
        miner_spend_public_key,
        Zeroizing::new(miner_view_private_key),
    );

    let miner_addr = miner_view_pair.address(Network::Mainnet, AddressSpec::Standard);

    // ------- DAEMON RPC ---------
    let daemon_rpc = Rpc::new("http://127.0.0.1:18081".to_string()).unwrap();

    // ------- MINER WALLET RPC ---------
    let wallet_rpc = monero_rpc::RpcClientBuilder::new()
        .build("http://localhost:18085")
        .unwrap()
        .wallet();

    // ------- MINER SCANNER ---------
    let mut miner_scanner = Scanner::from_view(miner_view_pair.clone(), Some(HashSet::new()));

    // ------- CREATE TRANSACTION ---------
    type Builder = SignableTransactionBuilder;

    let miner_spendable_outputs = get_miner_txs(&daemon_rpc, &mut miner_scanner).await;
    println!(
        "\nMiner has {} spendable outputs",
        miner_spendable_outputs.len()
    );

    let r_seed = Zeroizing::new(Keccak256::digest(random_scalar(&mut OsRng).to_bytes()).into());

    let mut builder = SignableTransactionBuilder::new(
        daemon_rpc.get_protocol().await.unwrap(),
        daemon_rpc.get_fee().await.unwrap(),
        Some(Change::new(&miner_view_pair, false)),
    )
    .set_r_seed(r_seed);

    let sign = |tx: SignableTransaction| {
        let daemon_rpc = daemon_rpc.clone();

        async move {
            tx.sign(
                &mut OsRng,
                &daemon_rpc,
                &Zeroizing::new(miner_spend_private_key),
            )
            .await
            .unwrap()
        }
    };

    let amount = monero::Amount::from_str(XMR_AMOUNT_1).unwrap();
    let amount = amount.as_pico();

    builder.add_payment(joint_address, amount);

    let selected_inputs: Vec<SpendableOutput> = select_inputs(
        &daemon_rpc,
        &miner_spend_private_key,
        &miner_spendable_outputs,
        amount,
    )
    .await;
    println!("\nSelected {} inputs", selected_inputs.len());
    builder.add_inputs(&selected_inputs);

    let tx = builder.build().unwrap();
    let signed_tx = sign(tx).await;

    daemon_rpc.publish_transaction(&signed_tx).await.unwrap();
}

async fn get_miner_txs(rpc: &Rpc, scanner: &mut Scanner) -> Vec<SpendableOutput> {
    let highest = rpc.get_height().await.unwrap();

    let mut spendable_outputs = vec![];

    for i in 1..highest {
        let block = rpc.get_block_by_number(i).await.unwrap();

        let outputs = scanner.scan(rpc, &block).await.unwrap();
        let unlocked_outputs: Vec<SpendableOutput> = outputs
            .into_iter()
            // .inspect(|o| println!("Found output: {:?}", o.timelock()))
            .flat_map(|o| o.ignore_timelock())
            .collect();

        spendable_outputs.extend(unlocked_outputs);
    }

    spendable_outputs
}

async fn select_inputs(
    rpc: &Rpc,
    spend: &Scalar,
    spendable_outs: &[SpendableOutput],
    amount: u64,
) -> Vec<SpendableOutput> {
    // for the purposes of testing, add 0.1 XMR to account for possible fees
    let epsilon = monero::Amount::from_str("0.1 XMR").unwrap().as_pico();

    let mut selected_outputs = vec![];

    let mut total = 0;

    // also, terrible way to select inputs; but this is only for testing
    for out in spendable_outs {
        let input_spend = Zeroizing::new(out.key_offset() + spend);
        let image = generate_key_image(&input_spend).compress();

        if rpc.is_key_image_spent(image.as_bytes()).await.unwrap() {
            println!("Skipping spent output");
            continue;
        }

        println!(
            "Selecting output to be used as input; has key {:?}, and has image: {:?}",
            hex::encode(out.key().compress().as_bytes()),
            hex::encode(image.as_bytes()),
        );

        selected_outputs.push(out.clone());
        total += out.commitment().amount;

        if total >= amount + epsilon {
            break;
        }
    }

    if total < amount + epsilon {
        panic!("Not enough funds");
    }

    selected_outputs
}
