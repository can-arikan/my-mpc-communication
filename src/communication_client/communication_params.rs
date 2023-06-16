use mpc_rocket::client::client_params::ClientParams;
use mpc_utils::env::util::Util;

pub struct MultiPartyCommunicationParams;

#[allow(non_snake_case)]
impl MultiPartyCommunicationParams {
    pub fn loadEnv() -> ClientParams {
        let ip_addr = Util::loadOrNone("MULTIPARTY_COMMUNICATION_ADDR");
        let port = Util::loadOrNone("MULTIPARTY_COMMUNICATION_PORT");

        if ip_addr.is_none() || port.is_none() {
            panic!("communication address and port must be set");
        }

        return ClientParams::new(ip_addr.unwrap(), port.unwrap());
    }
}
