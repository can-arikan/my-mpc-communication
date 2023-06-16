use std::{ sync::{ Arc, RwLock }, thread, time::Duration };
use communication_client::communication_client::MultiPartyCommunicationClient;
use config::env::env::Environment;
use log::info;
use mpc_vault::vault::vault_service::{VaultService, tokenRenewalCycle, tokenRenewalAbortion};
use rocket::futures::executor::block_on;

pub mod config;
pub mod communication_client;
pub mod error;

#[rocket::main]
async fn main() {
    let env = Environment::LoadEnv();

    env_logger::init();

    let mut vault_service = VaultService::new(env.clone().vault_params).await;

    for _ in 0..env.vault_params.vault_retry_count {
        if vault_service.is_err() {
            log::warn!("could not create vault retrying to login");
            thread::sleep(Duration::from_secs(1));
            vault_service = VaultService::new(env.clone().vault_params).await;
        } else {
            break;
        }
    }

    if vault_service.is_err() {
        panic!("{}", vault_service.unwrap_err());
    }

    let vault_service = vault_service.unwrap();

    let vault_service = Arc::new(RwLock::new(vault_service));

    let cloned_vault_service: Arc<RwLock<VaultService>> = Arc::clone(&vault_service);

    let communication_service = MultiPartyCommunicationClient::new(env.clone(), cloned_vault_service);

    for i in 0..env.vault_params.vault_retry_count {
        match block_on(vault_service.write().unwrap().to_owned().setupHealtcheckFile()) {
            Ok(_) => {
                break;
            }
            Err(err) => {
                log::warn!("could not insert health check file");
                thread::sleep(Duration::from_secs(1));
                if i == env.vault_params.vault_retry_count - 1 {
                    panic!("{}", err);
                }
            }
        }
    }

    let cloned_vault_service: Arc<RwLock<VaultService>> = Arc::clone(&vault_service);

    let token_renewal_handler = tokenRenewalCycle(cloned_vault_service).await;

    let communication_service_handler = tokio::spawn(communication_service.start());

    // END STATE
    match communication_service_handler.await {
        Ok(_) => {}
        Err(err) => {
            log::error!("{}", err);
        }
    }

    match block_on(vault_service.read().unwrap().to_owned().clearHealthFile()) {
        Ok(_) => {
            info!("health file successfully cleared from vault");
        }
        Err(err) => {
            log::error!("{}", err);
        }
    }

    tokenRenewalAbortion(token_renewal_handler).await;
}
