//  Copyright 2020 Two Sigma Investments, LP.
//
//  Licensed under the Apache License, Version 2.0 (the "License");
//  you may not use this file except in compliance with the License.
//  You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
//  Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.

// Google platform auth token getter.
// If the GOOGLE_APPLICATION_CREDENTIALS env var is provided, we use the service account
// to authenticate via OAuth. Otherwise, we fetch the compute metadata token.

use serde::{Serialize, Deserialize};
use anyhow::{anyhow, Result};
use ureq::{json, Error};
use std::{fs, time::SystemTime};
use super::ureq_request;

const SERVICE_ACCOUNT_ENV_VAR: &str = "GOOGLE_APPLICATION_CREDENTIALS";
const OAUTH_TOKEN_URL: &str = "https://www.googleapis.com/oauth2/v4/token";
const OAUTH_SCOPE_BASE_URL: &str = "https://www.googleapis.com/auth";

const COMPUTE_METADATA_TOKEN_URL: &str =
  "http://metadata.google.internal/computeMetadata/v1/instance/service-accounts/default/token";

#[derive(Serialize)]
struct Claims<'a> {
    iss: &'a str,
    scope: &'a str,
    aud: &'a str,
    exp: u64,
    iat: u64,
}

/// A deserialized `service-account-********.json`-file.
#[derive(Deserialize, Debug)]
pub struct ServiceAccount {
    #[serde(rename = "type")]
    pub sa_type: String,
    pub project_id: String,
    pub private_key_id: String,
    pub private_key: String,
    pub client_email: String,
    pub client_id: String,
    pub auth_uri: String,
    pub token_uri: String,
    pub auth_provider_x509_cert_url: String,
    pub client_x509_cert_url: String,
}

#[derive(Deserialize, Debug)]
struct TokenResponse {
    access_token: String,
    expires_in: usize,
    token_type: String,
}

// TODO Upstream this to ureq
fn json_response<T: serde::de::DeserializeOwned>(res: ureq::Response) -> serde_json::Result<T> {
    serde_json::from_reader(res.into_reader())
}

impl ServiceAccount {
    pub fn read_from(path: &str) -> Self {
        let file = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("{} file not found: {}", &path, e));

        let account: Self = serde_json::from_str(&file)
            .unwrap_or_else(|e| panic!("{} is not a valid service account file: {}", &path, e));

        assert!(account.sa_type == "service_account",
                "The service account file {} is invalid \
                `type` is '{}' but should be 'service_account'", &path, account.sa_type);

        account
    }
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

// Code inspired from the cloud-storage crate, licensed under the MIT license.
// It is available at https://github.com/ThouCheese/cloud-storage-rs.git
pub fn get_oauth_token(service_account: &ServiceAccount, scope: &str) -> Result<String> {
    const ONE_HOUR_IN_SECS: u64 = 3600;

    let jwt = {
        let now = now();
        let exp = now + ONE_HOUR_IN_SECS;

        let header = jsonwebtoken::Header {
            alg: jsonwebtoken::Algorithm::RS256,
            ..Default::default()
        };
        let claims = Claims {
            iss: &service_account.client_email,
            scope: &scope,
            aud: &OAUTH_TOKEN_URL,
            exp,
            iat: now,
        };
        let private_key = jsonwebtoken::EncodingKey::from_rsa_pem(
            service_account.private_key.as_bytes()
        )?;
        jsonwebtoken::encode(&header, &claims, &private_key)?
    };

    let res = ureq_request("POST", OAUTH_TOKEN_URL)
        .send_json(json!({
            "grant_type": "urn:ietf:params:oauth:grant-type:jwt-bearer",
            "assertion": &jwt
        }));

    if res.ok() {
        Ok(json_response::<TokenResponse>(res)?.access_token)
    } else {
        Err(anyhow!("Failed to authenticate: {}", res.into_string().unwrap()))
    }
}

fn get_compute_metadata_token() -> Result<Option<String>> {
    let res = ureq_request("GET", COMPUTE_METADATA_TOKEN_URL)
        .set("Metadata-Flavor", "Google")
        .call();

    if let Some(Error::DnsFailed(_)) = res.synthetic_error() {
        // We are most likely not running on google platform. We don't need to report an error.
        return Ok(None);
    }

    if res.ok() {
        Ok(Some(json_response::<TokenResponse>(res)?.access_token))
    } else {
        Err(anyhow!("Failed get compute metadata token: {}", res.into_string().unwrap()))
    }
}

pub fn get_auth(oauth_scope: &str) -> Result<String> {
    let token = match std::env::var(SERVICE_ACCOUNT_ENV_VAR).ok() {
        Some(path) => {
            let service_account = ServiceAccount::read_from(&path);
            let oauth_scope_url = format!("{}/{}", OAUTH_SCOPE_BASE_URL, oauth_scope);
            get_oauth_token(&service_account, &oauth_scope_url)?
        }
        None => get_compute_metadata_token()?.unwrap_or_else(|| panic!(
            "{} env var must be set to the service-account.json path", SERVICE_ACCOUNT_ENV_VAR))
    };

    Ok(format!("Bearer {}", token))
}
