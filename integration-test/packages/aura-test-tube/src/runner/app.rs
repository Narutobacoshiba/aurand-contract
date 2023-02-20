use cosmwasm_std::Coin;

use test_tube::account::SigningAccount;

use test_tube::runner::result::{RunnerExecuteResult, RunnerResult};
use test_tube::runner::Runner;
use test_tube::BaseApp;

const FEE_DENOM: &str = "ueaura";
const CHAIN_ID: &str = "euphoria-2";
const DEFAULT_GAS_ADJUSTMENT: f64 = 1.2;

#[derive(Debug, PartialEq)]
pub struct AuraTestApp {
    inner: BaseApp,
}

impl Default for AuraTestApp {
    fn default() -> Self {
        AuraTestApp::new()
    }
}

impl AuraTestApp {
    pub fn new() -> Self {
        Self {
            inner: BaseApp::new(FEE_DENOM, CHAIN_ID, DEFAULT_GAS_ADJUSTMENT),
        }
    }

    /// Initialize account with initial balance of any coins.
    /// This function mints new coins and send to newly created account
    pub fn init_account(&self, coins: &[Coin]) -> RunnerResult<SigningAccount> {
        self.inner.init_account(coins)
    }
    /// Convinience function to create multiple accounts with the same
    /// Initial coins balance
    pub fn init_accounts(&self, coins: &[Coin], count: u64) -> RunnerResult<Vec<SigningAccount>> {
        self.inner.init_accounts(coins, count)
    }

    /// Simulate transaction execution and return gas info
    pub fn simulate_tx<I>(
        &self,
        msgs: I,
        signer: &SigningAccount,
    ) -> RunnerResult<cosmrs::proto::cosmos::base::abci::v1beta1::GasInfo>
    where
        I: IntoIterator<Item = cosmrs::Any>,
    {
        self.inner.simulate_tx(msgs, signer)
    }
}

impl<'a> Runner<'a> for AuraTestApp {
    fn execute_multiple<M, R>(
        &self,
        msgs: &[(M, &str)],
        signer: &SigningAccount,
    ) -> RunnerExecuteResult<R>
    where
        M: ::prost::Message,
        R: ::prost::Message + Default,
    {
        self.inner.execute_multiple(msgs, signer)
    }

    fn query<Q, R>(&self, path: &str, q: &Q) -> RunnerResult<R>
    where
        Q: ::prost::Message,
        R: ::prost::Message + Default,
    {
        self.inner.query(path, q)
    }
}

#[cfg(test)]
mod tests {
    use std::option::Option::None;
    use cosmwasm_std::{coins, Timestamp, Addr, HexBinary};
    use crate::module::Wasm;
    use crate::runner::app::AuraTestApp;
    use test_tube::account::{Account};
    use test_tube::runner::*;
    use test_tube::module::Module;
    use test_tube::account::SigningAccount;
    use test_tube::runner::result::{RunnerExecuteResult};
    use test_tube::runner::error::{RunnerError};
    use cosmrs::proto::cosmwasm::wasm::v1::{MsgExecuteContractResponse};

    use cosmwasm_schema::{cw_serde};

    use cosmrs::proto::cosmos::bank::v1beta1::{
        QueryAllBalancesRequest, QueryAllBalancesResponse
    };


    #[cw_serde]
    struct AurandInstantiateMsg {
        pub nois_proxy: String, 
        pub time_expired: u64, 
        pub time_per_block: u64, 
        pub bounty_denom: String,
        pub fee: String,
        pub nois_fee: String,
        pub callback_limit_gas: u64,
        pub max_callback: u32,
    }
    
    #[cw_serde]
    struct LottoInstantiateMsg {
        pub aurand_addr: String,
    }

    #[cw_serde]
    pub enum LottoExecuteMsg {
        RequestHexRandomness {
            request_id: String, 
        },

        ReceiveHexRandomness {
            request_id: String,
            randomness: Vec<String>,
        },
    }

    #[cw_serde]
    pub struct NoisCallback {
        job_id: String,
        randomness: HexBinary,
    }

    #[cw_serde]
    pub enum AurandExecuteMsg {
        RegisterBot {
            hashed_api_key: String,
            moniker: String,
        },

        AddRandomness{
            random_value: String,
            signature: String 
        },

        NoisReceive {
            callback: NoisCallback 
        },
    }

    #[cw_serde]
    pub enum AurandQueryMsg {
        GetNumberOfCommitment{},
        GetPendingCommitments{limit: u32},
        GetCommitments{limit: u32},
    }

    #[cw_serde]
    pub struct NumberOfCommitmentQuery {
        pub num: u64
    }

    #[cw_serde]
    pub struct DataRequest {
        pub min: i32,
        pub max: i32,
        pub num: u32,
        pub data_type: String,
    }

    #[cw_serde]
    pub struct Commitment {
        pub id: String,
        pub request_id: String,
        pub owner: Addr,
        pub commit_time: Timestamp,
        pub expired_time: Timestamp,
        pub data_request: DataRequest,
    }

    #[cw_serde]
    pub struct PendingCommitmentsQuery {
        pub commitments: Vec<Commitment>
    }

    #[cw_serde]
    pub struct UserResponse {
        pub request_id: String,
        pub commit_time: u64,
        pub completion_time: u64,
        pub data: Vec<String>,
    }

    #[cw_serde] 
    pub enum LottoQueryMsg {
        GetResponses{}
    }

    #[cw_serde]
    pub struct GetResponsesQuery {
        pub responses: Vec<UserResponse>,
    }

    #[cw_serde]
    pub struct CommitmentsQuery {
        pub commitments: Vec<Commitment>
    }

    #[test]
    fn test_init_accounts() {
        let app = AuraTestApp::default();
        let accounts = app
            .init_accounts(&coins(100_000_000_000, "ueaura"), 3)
            .unwrap();

        assert!(accounts.get(0).is_some());
        assert!(accounts.get(1).is_some());
        assert!(accounts.get(2).is_some());
        assert!(accounts.get(3).is_none());
    }

    fn get_account_balances(app: &AuraTestApp, address: String, denom: &str) -> u128 {
        let acc_balance = app.query::<QueryAllBalancesRequest,QueryAllBalancesResponse>(
            "/cosmos.bank.v1beta1.Query/AllBalances",
            &QueryAllBalancesRequest {
                address,
                pagination: None,
            },
        )
        .unwrap()
        .balances
        .into_iter()
        .find(|c| c.denom == denom)
        .unwrap()
        .amount
        .parse::<u128>()
        .unwrap();

        return acc_balance;
    }

    #[test] 
    fn test_query() {
        let app = AuraTestApp::default();

        let acc = app.init_account(&coins(100_000_000_000, "ueaura")).unwrap();
        let addr = acc.address();

        let acc_balance = get_account_balances(&app, addr, "ueaura");

        assert_eq!(acc_balance, 100_000_000_000u128);
    }

    fn instantiate_aurand_contract(wasm: &Wasm<AuraTestApp>, msg: &AurandInstantiateMsg, signer: &SigningAccount) -> String {
        // store and instantiate aurand contract
        let aurand_byte_code = std::fs::read("./test_artifacts/aurand.wasm").unwrap(); // load contract wasm 
        let aurand_code_id = wasm
            .store_code(
                &aurand_byte_code, 
                None, 
                signer 
            )
            .unwrap()
            .data
            .code_id; 
        assert_eq!(aurand_code_id, 1);
        
        let wasm_instantiate = wasm.instantiate(
            aurand_code_id,
            msg,
            None,
            Some("aurand"), 
            &[], 
            signer, 
        ).unwrap();

        return wasm_instantiate.data.address;
    }

    fn instantiate_lotto_contract(wasm: &Wasm<AuraTestApp>, msg: &LottoInstantiateMsg, signer: &SigningAccount) -> String {
        // store and instantiate lotto game contract
        let lotto_byte_code = std::fs::read("./test_artifacts/lotto_game.wasm").unwrap(); // load contract wasm 
        let lotto_code_id = wasm
            .store_code(
                &lotto_byte_code, 
                None, 
                signer 
            )
            .unwrap()
            .data
            .code_id; 
        assert_eq!(lotto_code_id, 2);
        
        let wasm_instantiate = wasm.instantiate(
            lotto_code_id,
            msg, 
            None, 
            Some("lotto-game"), 
            &[], 
            signer, 
        ).unwrap();
        return wasm_instantiate.data.address;
    }

    fn aurand_register_bot(wasm: &Wasm<AuraTestApp>, aurand_contract_addr: &String, hashed_api_key: String, moniker: String, signer: &SigningAccount) 
    -> RunnerExecuteResult<MsgExecuteContractResponse> {
        return wasm.execute::<AurandExecuteMsg>(
            aurand_contract_addr,
            &AurandExecuteMsg::RegisterBot {
                hashed_api_key, 
                moniker, 
            },
            &[],
            &signer,
        );
    }

    fn aurand_add_randomness(wasm: &Wasm<AuraTestApp>, aurand_contract_addr: &String, random_value: String, signature: String, signer: &SigningAccount) 
    -> RunnerExecuteResult<MsgExecuteContractResponse> {
        return wasm.execute::<AurandExecuteMsg>(
            &aurand_contract_addr,
            &AurandExecuteMsg::AddRandomness {
                random_value,
                signature, 
            },
            &[],
            &signer,
        );
    }

    fn lotto_request_randomness(wasm: &Wasm<AuraTestApp>, lotto_contract_addr: &String, request_id: String, signer: &SigningAccount) 
    -> RunnerExecuteResult<MsgExecuteContractResponse>{
        return wasm.execute::<LottoExecuteMsg>(
            lotto_contract_addr,
            &LottoExecuteMsg::RequestHexRandomness {
                request_id
            },
            &coins(600, "ueaura"),
            signer,
        );
    }

    const RANDOM_VALUE_TEST: &str = r#"{"method":"generateSignedIntegers","hashedApiKey":"elv0PecZXTaHIbs+3PvmJIz2BO9mtakvSpznFkhgfe/EtmPPCVAqpBDIT6ZeQ3TEsCvdxymXnDuSPKoqjlxZ/Q==","n":32,"min":0,"max":255,"replacement":true,"base":10,"pregeneratedRandomization":null,"data":[127,12,177,76,70,175,6,221,126,220,251,62,125,122,39,146,236,173,173,240,28,197,116,202,130,36,88,171,55,232,75,86],"license":{"type":"developer","text":"Random values licensed strictly for development and testing only","infoUrl":null},"licenseData":null,"userData":null,"ticketData":null,"completionTime":"2023-02-07 03:05:57Z","serialNumber":489}"#;
    const SIGNATURE_TEST: &str = "kITMbucgIRih+606JH/zfYDIBqOYbB4VEyjCkLJIteIqMMRMZrFRBPmP4Lm+AXNSr4pl2j5fGBXcBJJUdLb4i1p/o4yI7XMg3B3lxhxbZc0fLQ4oWfPniM7El8T6AzxSgBl+OzPU08A+628j7D88IxaGXk5nzrCOmyYhTElfJNwe7erT2SJu9ydA0bC8OypRxJvfBAq4repxhsYFOG32ZhiTQ60BrjB2cTkgTTsLtBYipvp/sTfMZtUAwZ4wrYmSnBqgAFhM9IvpasrYp/4b2wej4AOKwMD34iipg84+29JwwapRBdWizzUm/TdKMvHUMAnwfyWkGs48mMtVjQstWA6A/gWkQILC5DnWJwF0DG1xOUSWO3lc3ETCDt9kNzO6y43ybYZaTma65w3xlLmuMaJAj1tIRAgHcMIHrlC0nmy9FLKVUf/drjsF5BlKbCIG6mWFuQcG4rNCsLu+3l1DjP5QeJZul9DEREHRtbkPsLCAN/Vxe/M6jieKGEJzoE2FEqeeQZCV5n7ihYVOmcJwvO2e4rBpVuu6/giqB2qd+mNqnwyPoTRn60uZPNpyxzLA+L5VRbzNHIsukQHjAB1wZO7KFomHV0xT8WOHDsTO7QKLE8T5UaEeJZVYFLduj1Eg+b05YvqRV4dW6L6/5oVnHpDEYYsJaS+HeRXrcBS3/Lk=";

    #[test]
    fn test_user_request_and_receive_randomness() {
        let app = AuraTestApp::default();

        let acc = app.init_account(&coins(100_000_000_000, "ueaura")).unwrap();

        let wasm = Wasm::new(&app);
        
        // store and instantiate aurand contract
        let aurand_contract_addr = instantiate_aurand_contract(&wasm, &AurandInstantiateMsg {
            nois_proxy: String::from("cosmos14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s4hmalr"), 
            time_expired: 2000000000, 
            time_per_block: 5, 
            bounty_denom: String::from("ueaura"),
            fee: String::from("300"),
            nois_fee: String::from("300"),
            callback_limit_gas: 150000u64,
            max_callback: 5u32,
        }, &acc);

        // store and instantiate lotto game contract
        let lotto_contract_addr = instantiate_lotto_contract(&wasm, &LottoInstantiateMsg {
            aurand_addr: aurand_contract_addr.clone(), 
        }, &acc);
        
        // bot execute register
        aurand_register_bot(
            &wasm, 
            &aurand_contract_addr, 
            String::from("elv0PecZXTaHIbs+3PvmJIz2BO9mtakvSpznFkhgfe/EtmPPCVAqpBDIT6ZeQ3TEsCvdxymXnDuSPKoqjlxZ/Q=="),
            String::from("test moniker"),
            &acc
        ).unwrap();

        // execute lotto game request hex randomness
        lotto_request_randomness(
            &wasm,
            &lotto_contract_addr,
            String::from("test request 1"),
            &acc
        ).unwrap();


        // bot execute add randomness
        aurand_add_randomness(
            &wasm,
            &aurand_contract_addr,
            String::from(RANDOM_VALUE_TEST),
            String::from(SIGNATURE_TEST),
            &acc
        ).unwrap();

        // get all randomness response
        let query_msg = wasm
            .query::<LottoQueryMsg, GetResponsesQuery>(
                &lotto_contract_addr, // contract address
                &LottoQueryMsg::GetResponses {} // query defined in contract
        ).unwrap();

        assert_eq!(query_msg.responses.len(), 1);
    }

    #[test]
    fn request_and_receive_multi_randomness_success() {
        let app = AuraTestApp::default();

        let wasm = Wasm::new(&app);

        let accs = app
            .init_accounts(
                &coins(100_000_000_000, "ueaura"),
                3,
            )
            .unwrap();

        let owner = &accs[0];
        let bot = &accs[1];
        let user = &accs[2];
        
        // store and instantiate aurand contract
        let aurand_contract_addr = instantiate_aurand_contract(&wasm, &AurandInstantiateMsg {
            nois_proxy: String::from("cosmos14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s4hmalr"), 
            time_expired: 2000000000, 
            time_per_block: 5, 
            bounty_denom: String::from("ueaura"),
            fee: String::from("300"),
            nois_fee: String::from("300"),
            callback_limit_gas: 150000u64,
            max_callback: 4u32
        }, &owner);

        // store and instantiate lotto game contract
        let lotto_contract_addr = instantiate_lotto_contract(&wasm, &LottoInstantiateMsg {
            aurand_addr: aurand_contract_addr.clone(), 
        }, &user);

        // bot execute register
        aurand_register_bot(
            &wasm,
            &aurand_contract_addr,
            String::from("elv0PecZXTaHIbs+3PvmJIz2BO9mtakvSpznFkhgfe/EtmPPCVAqpBDIT6ZeQ3TEsCvdxymXnDuSPKoqjlxZ/Q=="),
            String::from("test moniker"),
            &bot
        ).unwrap();

        //request 10
        for i in 0..10 {
            lotto_request_randomness(
                &wasm,
                &lotto_contract_addr,
                String::from("test request ") + &i.to_string(),
                &user
            ).unwrap();
        }

        let user_new_balances = get_account_balances(&app, user.address().clone(), "ueaura");
        
        assert_eq!(user_new_balances, 99999994000); // fee 600, and made 10 request

        for _ in 0..2 {
            // bot execute add randomness
            aurand_add_randomness(
                &wasm,
                &aurand_contract_addr,
                String::from(RANDOM_VALUE_TEST),
                String::from(SIGNATURE_TEST),
                &bot
            ).unwrap();
        }

        let bot_new_balances = get_account_balances(&app, bot.address().clone(), "ueaura");
        
        assert_eq!(bot_new_balances, 100000002400); // reward 300, and 8 commitments has made

        // get all randomness response
        let query_msg = wasm
            .query::<LottoQueryMsg, GetResponsesQuery>(
                &lotto_contract_addr, // contract address
                &LottoQueryMsg::GetResponses {} // query defined in contract
        ).unwrap();

        assert_eq!(query_msg.responses.len(), 8);
    }   

    #[test]
    fn request_randomness_with_insufficient_balances() {
        let app = AuraTestApp::default();

        let wasm = Wasm::new(&app);

        let accs = app
            .init_accounts(
                &coins(700, "ueaura"),
                2,
            )
            .unwrap();

        let owner = &accs[0];
        let user = &accs[1];
        
        // store and instantiate aurand contract
        let aurand_contract_addr = instantiate_aurand_contract(&wasm, &AurandInstantiateMsg {
            nois_proxy: String::from("cosmos14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s4hmalr"), 
            time_expired: 2000000000, 
            time_per_block: 5, 
            bounty_denom: String::from("ueaura"),
            fee: String::from("300"),
            nois_fee: String::from("300"),
            callback_limit_gas: 150000u64,
            max_callback: 4u32
        }, &owner);

        // store and instantiate lotto game contract
        let lotto_contract_addr = instantiate_lotto_contract(&wasm, &LottoInstantiateMsg {
            aurand_addr: aurand_contract_addr.clone(), 
        }, &user);

        //request 1
        lotto_request_randomness(
            &wasm,
            &lotto_contract_addr,
            String::from("test request 1"),
            &user
        ).unwrap();

        let user_new_balances = get_account_balances(&app, user.address().clone(), "ueaura");
        
        assert_eq!(user_new_balances, 100); // fee 600, and made 1 request so current balances is 100

        // request with 100 amount of ueaura, must return err
        let execute_response = lotto_request_randomness(
            &wasm,
            &lotto_contract_addr,
            String::from("test request 2"),
            &user
        ); 
        
        match execute_response {
            Err(RunnerError::ExecuteError{msg}) => {
                assert_eq!(msg, String::from("failed to execute message; message index: 0: 100ueaura is smaller than 600ueaura: insufficient funds"));    
            },
            _ => panic!(),
        }

        // get number of commitments, must be 1
        let query_msg = wasm.query::<AurandQueryMsg, NumberOfCommitmentQuery>(
                &aurand_contract_addr, 
                &AurandQueryMsg::GetNumberOfCommitment {}
        ).unwrap();

        assert_eq!(query_msg.num, 1);
    }

    #[test]
    fn nois_callback_success_with_one_commitment() {
        let app = AuraTestApp::default();

        let wasm = Wasm::new(&app);

        let accs = app
            .init_accounts(
                &coins(1000, "ueaura"),
                3,
            )
            .unwrap();

        let owner = &accs[0];
        let user = &accs[1];
        let nois_proxy = &accs[2];

        // store and instantiate aurand contract
        let aurand_contract_addr = instantiate_aurand_contract(&wasm, &AurandInstantiateMsg {
            nois_proxy: nois_proxy.address(), 
            time_expired: 5, 
            time_per_block: 5, 
            bounty_denom: String::from("ueaura"),
            fee: String::from("300"),
            nois_fee: String::from("300"),
            callback_limit_gas: 150000u64,
            max_callback: 4u32
        }, &owner);

        // store and instantiate lotto game contract
        let lotto_contract_addr = instantiate_lotto_contract(&wasm, &LottoInstantiateMsg {
            aurand_addr: aurand_contract_addr.clone(), 
        }, &user);

        //request 1
        lotto_request_randomness(
            &wasm,
            &lotto_contract_addr,
            String::from("test request 1"),
            &user
        ).unwrap();

        wasm.execute::<AurandExecuteMsg>(
            &aurand_contract_addr,
            &AurandExecuteMsg::NoisReceive {
                callback: NoisCallback {
                    job_id: String::from("d0c1afb49942d01c7402c77757949dd21ee14f480dd3d7a5fa55295d6ad1e5ed"),
                    randomness: HexBinary::from(&[0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00]),
                }
            },
            &[],
            nois_proxy,
        ).unwrap();

        let user_proxy_balances = get_account_balances(&app, nois_proxy.address().clone(), "ueaura");
        
        assert_eq!(user_proxy_balances, 1000); // if commitment made by nois proxy, reward will pay to onwer, so proxy balances still equals 1000ueaura

        let user_onwer_balances = get_account_balances(&app, owner.address().clone(), "ueaura");
        
        assert_eq!(user_onwer_balances, 1300);

        // get all randomness response
        let query_msg = wasm
            .query::<LottoQueryMsg, GetResponsesQuery>(
                &lotto_contract_addr, // contract address
                &LottoQueryMsg::GetResponses {} // query defined in contract
        ).unwrap();

        assert_eq!(query_msg.responses.len(), 1);

    }

    #[test]
    fn aurand_query_succes_with_large_number_of_request_randomness() {
        let app = AuraTestApp::default();

        let wasm = Wasm::new(&app);

        let accs = app
            .init_accounts(
                &coins(100_000_000_000, "ueaura"),
                3,
            )
            .unwrap();

        let owner = &accs[0];
        let bot = &accs[1];
        let user = &accs[2];
        
        // store and instantiate aurand contract
        let aurand_contract_addr = instantiate_aurand_contract(&wasm, &AurandInstantiateMsg {
            nois_proxy: String::from("cosmos14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s4hmalr"), 
            time_expired: 2000000000, 
            time_per_block: 5, 
            bounty_denom: String::from("ueaura"),
            fee: String::from("300"),
            nois_fee: String::from("300"),
            callback_limit_gas: 150000u64,
            max_callback: 5u32
        }, &owner);

        // store and instantiate lotto game contract
        let lotto_contract_addr = instantiate_lotto_contract(&wasm, &LottoInstantiateMsg {
            aurand_addr: aurand_contract_addr.clone(), 
        }, &user);

        // bot execute register
        aurand_register_bot(
            &wasm,
            &aurand_contract_addr,
            String::from("elv0PecZXTaHIbs+3PvmJIz2BO9mtakvSpznFkhgfe/EtmPPCVAqpBDIT6ZeQ3TEsCvdxymXnDuSPKoqjlxZ/Q=="),
            String::from("test moniker"),
            &bot
        ).unwrap();

        //request 5000 time
        for i in 0..5000 {
            lotto_request_randomness(
                &wasm,
                &lotto_contract_addr,
                String::from("test request ") + &i.to_string(),
                &user
            ).unwrap();
        }

        // get number of commitments, must be 5000
        let query_number_msg = wasm.query::<AurandQueryMsg, NumberOfCommitmentQuery>(
                &aurand_contract_addr, 
                &AurandQueryMsg::GetNumberOfCommitment {}
        ).unwrap();

        assert_eq!(query_number_msg.num, 5000);
        
        // get list of pendings commitments, must be 10
        let query_pending_commitments_msg = wasm.query::<AurandQueryMsg, PendingCommitmentsQuery>(
                &aurand_contract_addr, 
                &AurandQueryMsg::GetPendingCommitments {
                    limit: 10
                }
        ).unwrap();

        assert_eq!(query_pending_commitments_msg.commitments.len(), 10);

        // get list of commitments, must be 10
        let query_commitments_msg = wasm.query::<AurandQueryMsg, CommitmentsQuery>(
                &aurand_contract_addr, 
                &AurandQueryMsg::GetCommitments {
                    limit: 10
                }
        ).unwrap();

        assert_eq!(query_commitments_msg.commitments.len(), 10);

    }
}

