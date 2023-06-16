use std::sync::{ Arc, RwLock };
use mpc_rocket::client::client_service::Client;
use mpc_vault::vault::vault_service::VaultService;
use rocket::{ serde::json::Json, response::status::NotFound };
use log::info;
use rocket::{ routes, Route, post, State };
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::{error::CommunicationError, config::env::env::Environment};

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct PartySignupKeygen {
    pub threshold: u16,
    pub share_count: u16,
    pub join_uuid: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct PartySignup {
    pub number: u16,
    pub uuid: String,
    pub join_uuid: String,
}

pub type Key = String;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Index {
    pub key: Key,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Entry {
    pub key: Key,
    pub value: String,
}

pub struct MultiPartyCommunicationClient {
    rocket_client: Client,
}

impl Clone for MultiPartyCommunicationClient {
    fn clone(&self) -> Self {
        Self {
            rocket_client: self.rocket_client.clone(),
        }
    }
}

impl MultiPartyCommunicationClient {
    fn endpoints() -> Vec<Route> {
        #[post("/get", format = "json", data = "<request>")]
        async fn get(
            db_mtx: &State<Arc<RwLock<VaultService>>>,
            request: Json<Index>
        ) -> Json<Result<Entry, ()>> {
            log::info!("{}", request.0.key);
            let index: Index = request.0;
            let clone = db_mtx.inner().to_owned();
            let res = tokio::task::spawn_blocking(move || {
                let locked_service = match clone.read() {
                    Ok(res) => res,
                    Err(err) => panic!("{}", err),
                };

                tokio::runtime::Runtime
                    ::new()
                    .unwrap()
                    .block_on(async {
                        locked_service
                            .read::<Entry>(&index.key).await
                            .map_err(|err| NotFound(err.to_string()))
                    })
            }).await;

            match res {
                Ok(Ok(v)) => { Json(Ok(v)) }
                Ok(_) => Json(Err(())),
                Err(_) => Json(Err(())),
            }
        }

        #[post("/set", format = "json", data = "<request>")]
        async fn set(
            db_mtx: &State<Arc<RwLock<VaultService>>>,
            request: Json<Entry>
        ) -> Json<Result<(), ()>> {
            let entry: Entry = request.0;
            let clone = db_mtx.inner().to_owned();
            tokio::task
                ::spawn_blocking(move || {
                    let locked_service = match clone.write() {
                        Ok(res) => res,
                        Err(err) => panic!("{}", err),
                    };

                    tokio::runtime::Runtime
                        ::new()
                        .unwrap()
                        .block_on(async { locked_service.insert(&entry.key, entry.clone()).await })
                }).await
                .unwrap();
            Json(Ok(()))
        }

        #[post("/initializekeygen", format = "json")]
        async fn initialize_keygen(
            db_mtx: &State<Arc<RwLock<VaultService>>>
        ) -> Json<Result<String, ()>> {
            let join_uuid = Uuid::new_v4().to_string();
            let join_uuid_clone = join_uuid.clone();
            let clone = db_mtx.inner().to_owned();
            let res = tokio::task::spawn_blocking(move || {
                let locked_service = match clone.write() {
                    Ok(res) => res,
                    Err(err) => panic!("{}", err),
                };
                let data = PartySignup {
                    number: 0,
                    uuid: Uuid::new_v4().to_string(),
                    join_uuid: join_uuid_clone,
                };
                tokio::runtime::Runtime
                    ::new()
                    .unwrap()
                    .block_on(async {
                        locked_service.insert(
                            &format!("signup-keygen-{}", data.join_uuid),
                            &data
                        ).await
                    })
            }).await;
            match res {
                Ok(_) => Json(Ok(join_uuid)),
                Err(_) => Json(Err(())),
            }
        }

        #[post("/signupkeygen", format = "json", data = "<request>")]
        async fn signup_keygen(
            db_mtx: &State<Arc<RwLock<VaultService>>>,
            request: Json<PartySignupKeygen>
        ) -> Json<Result<PartySignup, CommunicationError>> {
            let parties = request.0.threshold;
            let join_uuid = request.0.join_uuid;
            let key = format!("signup-keygen-{}", join_uuid);
            let key_clone = key.clone();

            let clone = db_mtx.inner().to_owned();

            let res: Result<Result<PartySignup, _>, tokio::task::JoinError> = Ok(
                tokio::task::spawn_blocking(move || {
                    let locked_service = match clone.read() {
                        Ok(res) => res,
                        Err(err) => panic!("{}", err),
                    };
                    let res = {
                        let client_signup: PartySignup = match
                            tokio::runtime::Runtime
                                ::new()
                                .unwrap()
                                .block_on(async {
                                    locked_service.read::<PartySignup>(&key_clone).await
                                })
                        {
                            Ok(value) => { value }
                            Err(err) => panic!("{}", err),
                        };
                        let res = if client_signup.number <= parties {
                            PartySignup {
                                number: client_signup.number + 1,
                                uuid: client_signup.uuid,
                                join_uuid: join_uuid.clone(),
                            }
                        } else {
                            PartySignup {
                                number: 1,
                                uuid: Uuid::new_v4().to_string(),
                                join_uuid: join_uuid.clone(),
                            }
                        };
                        res
                    };
                    res
                }).await
            );

            let party_signup = match res.unwrap() {
                Ok(res) => {
                    res
                },
                Err(err) => {
                    return Json(Err(CommunicationError::new(format!("{}", err))))
                }
            };
            let party_signup_clone = party_signup.clone();
            let clone = db_mtx.inner().to_owned();

            tokio::task
                ::spawn_blocking(move || {
                    let locked_service = match clone.write() {
                        Ok(res) => res,
                        Err(err) => panic!("{}", err),
                    };

                    tokio::runtime::Runtime
                        ::new()
                        .unwrap()
                        .block_on(locked_service.insert(&key, &party_signup_clone))
                }).await
                .unwrap();
            Json(Ok(party_signup))
        }

        routes![get, set, signup_keygen, initialize_keygen]
    }

    pub fn new(env: Environment, vault_service: Arc<RwLock<VaultService>>) -> Self {
        let mut rocket_client = Client::new(
            env.communication_params
                .to_owned(),
            Self::endpoints(),
            None
        );

        rocket_client = rocket_client.set_manage(env.vault_params).set_manage(vault_service);

        Self {
            rocket_client,
        }
    }

    pub async fn start(self) {
        match self.rocket_client.spawn_rocket().await.await {
            Ok(_) => { info!("service closed") }
            Err(err) => { panic!("{}", err) }
        };
    }
}
