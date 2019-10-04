// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::constants::{SAFE_AUTHD_ENDPOINT_HOST, SAFE_AUTHD_ENDPOINT_PORT};
use super::quic_client::quic_send;
use super::ResultReturn;
use log::{debug, info};
//use super::authenticator::AuthedAppsList;

// Path of authenticator endpoint for login into a SAFE account
const SAFE_AUTHD_ENDPOINT_LOGIN: &str = "login/";

// Path of authenticator endpoint for loging out from a SAFE account
const SAFE_AUTHD_ENDPOINT_LOGOUT: &str = "logout";

// Path of authenticator endpoint for creating a new SAFE account
const SAFE_AUTHD_ENDPOINT_CREATE: &str = "create/";

// Path of authenticator endpoint for fetching list of authorised apps
const SAFE_AUTHD_ENDPOINT_AUTHED_APPS: &str = "authed-apps";

// Path of authenticator endpoint for revoking applications and/or permissions
const SAFE_AUTHD_ENDPOINT_REVOKE: &str = "revoke/";

// Authd Client API
pub struct SafeAuthdClient {
    port: u16,
    //session_token: String, // TODO: keep the authd session token obtained after logging in a SAFE account
}

#[allow(dead_code)]
// TODO: we need to recognise erroneous response and return an Error
impl SafeAuthdClient {
    pub fn new(port: Option<u16>) -> Self {
        let port_number = port.unwrap_or(SAFE_AUTHD_ENDPOINT_PORT);
        Self {
            port: port_number, /*, session_token: "".to_string()*/
        }
    }

    // Send a login action request to remote authd endpoint
    pub fn log_in(&mut self, secret: &str, password: &str) -> ResultReturn<()> {
        debug!("Attempting to log in on remote authd...");
        let authd_service_url = format!(
            "{}:{}/{}{}/{}",
            SAFE_AUTHD_ENDPOINT_HOST, self.port, SAFE_AUTHD_ENDPOINT_LOGIN, secret, password
        );

        info!("Sending login action to SAFE Authenticator...");
        let authd_response = quic_send(&authd_service_url, false, None, None, false)?;

        info!("SAFE login action was successful: {}", authd_response);
        // TODO: store the authd session token, replacing an existing one
        // self.session_token = authd_response;

        Ok(())
    }

    // Sends a logout action request to the SAFE Authenticator
    pub fn log_out(&mut self) -> ResultReturn<()> {
        debug!("Dropping logged in session and logging out in remote authd...");

        let authd_service_url = format!(
            "{}:{}/{}",
            SAFE_AUTHD_ENDPOINT_HOST, self.port, SAFE_AUTHD_ENDPOINT_LOGOUT
        );

        info!("Sending logout action to SAFE Authenticator...");
        let authd_response = quic_send(&authd_service_url, false, None, None, false)?;

        info!("SAFE logout action was successful: {}", authd_response);

        // TODO: clean up the stored authd session token
        // self.session_token = "".to_string();

        Ok(())
    }

    // Sends an account creation request to the SAFE Authenticator
    pub fn create_acc(&mut self, sk: &str, secret: &str, password: &str) -> ResultReturn<()> {
        debug!("Attempting to create a SAFE account on remote authd...");
        let authd_service_url = format!(
            "{}:{}/{}{}/{}/{}",
            SAFE_AUTHD_ENDPOINT_HOST, self.port, SAFE_AUTHD_ENDPOINT_CREATE, secret, password, sk
        );

        debug!("Sending account creation request to SAFE Authenticator...");
        let authd_response = quic_send(&authd_service_url, false, None, None, false)?;

        debug!(
            "SAFE account creation action was successful: {}",
            authd_response
        );
        Ok(())
    }

    // Get the list of applications authorised from remote authd
    pub fn authed_apps(&self) -> ResultReturn</*Vec<AuthedAppsList>*/ String> {
        debug!("Attempting to fetch list of authorised apps from remote authd...");
        let authd_service_url = format!(
            "{}:{}/{}",
            SAFE_AUTHD_ENDPOINT_HOST, self.port, SAFE_AUTHD_ENDPOINT_AUTHED_APPS
        );

        debug!("Sending request request to SAFE Authenticator...");
        let authd_response = quic_send(&authd_service_url, false, None, None, false)?;

        debug!(
            "List of applications authorised successfully received: {}",
            authd_response
        );

        //let authed_apps = // TODO: deserialise string Ok(authed_apps)
        Ok(authd_response)
    }

    // Revoke all permissions from an application
    pub fn revoke_app(&self, app_id: &str) -> ResultReturn<()> {
        debug!(
            "Requesting to revoke permissions from application: {}",
            app_id
        );
        let authd_service_url = format!(
            "{}:{}/{}{}",
            SAFE_AUTHD_ENDPOINT_HOST, self.port, SAFE_AUTHD_ENDPOINT_REVOKE, app_id
        );

        debug!("Sending revoke action request to SAFE Authenticator...");
        let authd_response = quic_send(&authd_service_url, false, None, None, false)?;

        debug!(
            "Application revocation action successful: {}",
            authd_response
        );
        Ok(())
    }
}
