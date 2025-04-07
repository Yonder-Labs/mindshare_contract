use near_gas::NearGas;
use near_sdk::near;
use serde_json::json;

mod constants;
use constants::*;

#[near(serializers = [json, borsh])]
#[derive(Clone)]
pub struct Worker {
    checksum: String,
    codehash: String,
}

#[tokio::test]
async fn test_register_worker() -> anyhow::Result<()> {
    println!("Starting test...");
    let sandbox = near_workspaces::sandbox().await?;
    let contract_wasm = std::fs::read(CONTRACT_WASM).expect("Contract wasm not found");
    let contract = sandbox.dev_deploy(&contract_wasm).await?;

    println!("initializing contract...");
    let result = contract
        .call("init")
        .args_json(json!({
            "owner_id": contract.id()
        }))
        .transact()
        .await?;
    println!("\nResult init: {:?}", result);
    
    //Call register_worker
    let collateral = include_str!("Samples/quote_collateral.json").to_string();
    let result = contract
        .call("register_worker")
        .args_json(json!({
            "quote_hex": QUOTE_HEX.to_string(),
            "collateral": collateral,
            "checksum": CHECKSUM.to_string(),
            "tcb_info": TCB_INFO.to_string()
        }))
        .gas(NearGas::from_tgas(300))
        .transact()
        .await?;

    println!("\nResult register_worker: {:?}", result);
    assert!(result.is_success());

    let result_get_worker = contract
        .call("get_worker")
        .args_json(json!({"account_id" : contract.id()}))
        .view()
        .await?;

    let worker: Worker = serde_json::from_slice(&result_get_worker.result).unwrap();
    println!(
        "\n [LOG] Worker: {{ checksum: {}, codehash: {} }}",
        worker.checksum, worker.codehash
    );

    // Expect that fails because don't have an approved codehash
    let result_get_codehash = contract.call("require_approved_codehash").args_json(json!({})).transact().await?;

    println!("\n[LOG] Require approved codehash: {:?}", result_get_codehash);

    // Approve codehash
    let result_approve_codehash = contract.call("approve_codehash").args_json(json!({
        "codehash": worker.codehash
    })).transact().await?;

    println!("\n[LOG] Approve codehash: {:?}", result_approve_codehash);
    
    //Expect success value because we registered the codehash
    let result_get_codehash = contract.call("require_approved_codehash").args_json(json!({})).transact().await?;

    println!("\n[LOG] Require approved codehash AFTER approve: {:?}", result_get_codehash);

    Ok(())
}
