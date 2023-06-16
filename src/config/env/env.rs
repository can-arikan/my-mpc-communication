#![allow(non_snake_case)]
use dotenv::dotenv;
use mpc_rocket::client::client_params::ClientParams;
use mpc_vault::vault::vault_params::VaultParams;

use crate::communication_client::communication_params::MultiPartyCommunicationParams;

#[derive(Clone)]
pub struct Environment {
    pub vault_params: VaultParams,
    pub communication_params: ClientParams,
}

impl Environment {
    pub fn LoadEnv() -> Self {
        dotenv().ok();
        let vault_params = VaultParams::loadEnv();
        let communication_params = MultiPartyCommunicationParams::loadEnv();
        return Self {
            vault_params,
            communication_params
        };
    }
}
