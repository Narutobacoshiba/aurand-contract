# **aurand-contract**
On-chain randomness provider for decentralized applications

Aurand using [Random Org](https://www.random.org) and [Nois](https://docs.nois.network/) as original sources of randomness to providing random number with the following features
- **Unbiasable**: No attacker, or coalition of attackers, should be able to bias the output.
- **Reliable**: No attacker should be able to prevent the protocol from producing output.
- **Verifiable**: Anybody can easily verify the protocol output and should see the same output as everybody else.
- **Unpredictable**: If the protocol produces output at time *T1*, nobody should be able to predict anything about the output before some time *T0<T1*, ideally with *T0* very close to *T1*
- **Low-delay:** low time delay between request and reception of the random number

### Storing the Aurand Contract Address
```Rust
    pub fn instantiate(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        _msg: InstantiateMsg,
    ) -> Result<Response, ContractError> {


        let aurand_addr = deps
            .api
            .addr_validate(&_msg.aurand_addr)
            .map_err(|_| ContractError::InvalidAurandAddress{})?;

        AURAND_ADDR.save(deps.storage, &aurand_addr)?;

    }
```

### Make a Random Request
* Create `ExecuteMsg` enum 
```Rust
    #[cw_serde]
    pub enum AurandExecuteMsg {
        // user request for hex randomness
        RequestHexRandomness{
            request_id: String, // id of request
            num: u32 // number of wanted randomness 
        },

        // user request for integer randomness
        RequestIntRandomness{
            request_id: String, // id of request
            min: i32, // min value of each randomness
            max: i32, // max valud of each randomness
            num: u32, // number of wanted randomness
        }
    }
```

* Use
```Rust
    fn execute_request_hex_randomness(
        _deps: DepsMut,
        _env: Env, 
        _info: MessageInfo,
        request_id: String,
    ) -> Result<Response, ContractError> {

        let aurand_addr = AURAND_ADDR.load(_deps.storage)?; // aurand address

        // create a message 
        let res = Response::new().add_message(WasmMsg::Execute {
            contract_addr: aurand_addr.to_string(),
            msg: to_binary(&AurandExecuteMsg::RequestHexRandomness { 
                            request_id: request_id.clone(),
                            num: 1,
                        })?,
            funds: _info.funds,
        });
    }
```

### Processing The Callback
* Create `ExecuteMsg` enum cases
```Rust
    #[cw_serde]
    pub enum ExecuteMsg {
        // receive list of hex randomness
        ReceiveHexRandomness {
            request_id: String,
            randomness: Vec<String>,
        },
        // receive list of int randomness
        ReceiveIntRandomness {
            request_id: String,
            randomness: Vec<i32>,
        },
    }
```

* use
```Rust
#[cfg_attr(not(feature = "library"), entry_point)]
    pub fn execute(
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        msg: ExecuteMsg,
    ) -> Result<Response, ContractError> {
        match msg {
            ExecuteMsg::ReceiveHexRandomness {
                request_id, 
                randomness,
            } => execute_receive_hex_randomness(_deps,_env,_info,request_id,randomness), // receive random from aurad via callback message
        }
    }

    // ...

    fn execute_receive_hex_randomness(
        _deps: DepsMut,
        _env: Env, 
        _info: MessageInfo,
        request_id: String,
        randomness: Vec<String>,
    ) -> Result<Response, ContractError> {

        // use randomness here
    }
```

### Unit Test
* install `grcov`
```
    cargo install grcov
```

* run `unit_test.sh`
```
    ./scripts/unit_test.sh
```

* open `target/coverage/index.html` to view report
example in `wsl`
```
    wslview ./target/coverage/index.html
```

### Integration Test

* build aurand wasm file and optimize
```
    beaker wasm build --no-wasm-opt

    docker run --rm -v "$(pwd)":/code \
    --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
    --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
    cosmwasm/rust-optimizer:0.12.9
```

* copy `artifacts/aurand.wasm` to `integration-test/packages/aura-test-tube/test_artifacts`

* run `integration_test.sh`
```
    ./scripts/integration_test.sh
```