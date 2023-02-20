use cosmwasm_schema::write_api;
use aurand::msg::{ExecuteMsg, QueryMsg};
use aurand::msg::InstantiateMsg;

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: ExecuteMsg,
        query: QueryMsg
    }
}
