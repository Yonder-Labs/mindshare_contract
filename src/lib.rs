use constants::*;
use dcap_qvl::{verify, QuoteCollateralV3};
use hex::{decode, encode};
use near_sdk::{
    env,
    env::block_timestamp,
    ext_contract, log, near, require,
    store::{IterableMap, IterableSet},
    AccountId, CryptoHash, Gas, PanicOnDefault, Promise, PublicKey,
};

mod collateral;
mod constants;

#[ext_contract(mpc)]
trait MPC {
    fn sign(&mut self, request: SignRequest) -> Promise;

    #[handle_result]
    fn derived_public_key(
        &self,
        path: String,
        predecessor: Option<AccountId>,
    ) -> Result<PublicKey, std::fmt::Error>;
}

#[near(serializers = [json, borsh])]
#[derive(Clone)]
pub struct SignRequest {
    pub payload: [u8; 32],
    pub path: String,
    pub key_version: u32,
}

#[near(serializers = [json, borsh])]
#[derive(Clone)]
pub struct Worker {
    checksum: String,
    codehash: String,
}

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct Contract {
    owner_id: AccountId,
    derivation_path: Option<String>,
    approved_codehashes: IterableSet<String>,
    worker_by_account_id: IterableMap<AccountId, Worker>,
}

#[near]
impl Contract {
    #[init]
    #[private]
    pub fn init(owner_id: AccountId) -> Self {
        Self {
            owner_id,
            derivation_path: None,
            approved_codehashes: IterableSet::new(b"a"),
            worker_by_account_id: IterableMap::new(b"b"),
        }
    }

    pub fn register_worker(
        &mut self,
        quote_hex: String,
        collateral: String,
        checksum: String,
        tcb_info: String,
    ) -> bool {
        let collateral = collateral::get_collateral(collateral);
        let quote = decode(quote_hex).unwrap();
        let now = block_timestamp() / 1000000000;
        let result = verify::verify(&quote, &collateral, now).expect("report is not verified");
        let rtmr3 = encode(result.report.as_td10().unwrap().rt_mr3.to_vec());
        let codehash = collateral::verify_codehash(tcb_info, rtmr3);

        // uncomment this line to only allow workers to register if their codehash arg is approved
        // require!(self.approved_codehashes.contains(&codehash));

        log!("verify result: {:?}", result);

        let predecessor = env::predecessor_account_id();
        self.worker_by_account_id
            .insert(predecessor, Worker { checksum, codehash });

        true
    }

    #[payable]
    pub fn sign_trade(&mut self, quote: String) -> Promise {
        self.require_approved_codehash();

        assert!(!quote.is_empty(), "Quote cannot be empty");

        log!("Quote send to format_erc191: {}", quote);
        let formatted_quote = self.format_erc191(quote.clone());

        log!("Formatted quote: {:?}", formatted_quote);
        let payload = self.generate_payload(formatted_quote);
        log!("Payload: {:?}", payload);
        let request = SignRequest {
            payload,
            path: String::from(MPC_PATH),
            key_version: KEY_VERSION,
        };

        mpc::ext(MPC_CONTRACT_ACCOUNT_ID.parse().unwrap())
            .with_static_gas(Gas::from_tgas(100))
            .with_attached_deposit(env::attached_deposit())
            .sign(request)
    }

    fn format_erc191(&self, quote: String) -> Vec<u8> {
        log!("Formatting quote to erc191");
        [
            format!("\x19Ethereum Signed Message:\n{}", quote.len()).as_bytes(),
            quote.as_bytes(),
        ]
        .concat()
    }

    pub fn get_derivation_path(&self) -> Option<String> {
        self.derivation_path.clone()
    }

    pub fn global_settlement() {
        //TODO: Implement the global settlement logic
    }

    pub fn generate_payload(&self, data: Vec<u8>) -> CryptoHash {
        log!("Generating payload");
        env::keccak256_array(&data)
    }

    #[payable]
    pub fn get_public_key(&mut self) -> Promise {
        let path = String::from(MPC_PATH);
        let predecessor = env::predecessor_account_id();

        mpc::ext(MPC_CONTRACT_ACCOUNT_ID.parse().unwrap())
            .with_static_gas(Gas::from_tgas(100))
            .derived_public_key(path, Some(predecessor))
    }

    // access control helpers

    pub fn require_owner(&mut self) {
        require!(env::predecessor_account_id() == self.owner_id);
    }

    pub fn approve_codehash(&mut self, codehash: String) {
        // !!! UPGRADE TO YOUR METHOD OF MANAGING APPROVED WORKER AGENT CODEHASHES !!!
        self.require_owner();
        self.approved_codehashes.insert(codehash);
    }

    pub fn require_approved_codehash(&mut self) {
        let worker = self.get_worker(env::predecessor_account_id());
        require!(self.approved_codehashes.contains(&worker.codehash));
    }

    // views

    pub fn get_worker(&self, account_id: AccountId) -> Worker {
        self.worker_by_account_id
            .get(&account_id)
            .unwrap()
            .to_owned()
    }
}
