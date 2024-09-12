use std::{collections::HashMap, fs, path::Path};

use reqwest::blocking::Body;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tiny_http::{Response, Server};

const CLIENT_ID: &str = "353451169812-gf5j4nmiosovv7sendcanjmmcumoq0dl.apps.googleusercontent.com";

// according to google, since you can't keep secrets in desktop apps, keeping it in the source could should be ok.
// furthermore, google treats client secrets in native apps as extensions of the client ID, and not really a secret,
// and things like incremental login will not work with desktop apps, which is what the secret is used for on web
const CLIENT_SECRET: &str = "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";

const BASE_LOGIN_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";

const BASE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";

#[derive(Serialize, Deserialize)]
struct TokenResponse {
  access_token: String,
  refresh_token: String,
  token_type: String,
  expires_in: usize,
  scope: String
}

pub struct CloudService {
  access_token: String,
  refresh_token: String,
  auth_code: String,
  pub logged_in: bool
}

impl CloudService {
  pub fn new() -> Self {
    let mut access_token = "".to_string();

    let access_token_path = Path::new("./.access_token");
    if access_token_path.is_file() {
      access_token = fs::read_to_string("./.access_token").unwrap();
    }

    let refresh_token_path = Path::new("./.refresh_token");

    let mut refresh_token = "".to_string();

    if refresh_token_path.is_file() {
      refresh_token = fs::read_to_string("./.refresh_token").unwrap();
    }


    Self {
      access_token: access_token.to_string(),
      refresh_token: refresh_token.to_string(),
      auth_code: String::new(),
      logged_in: access_token != ""
    }
  }

  pub fn login(&mut self) {
    let mut query_params: Vec<[&str; 2]> = Vec::new();

    query_params.push(["response_type", "code"]);
    query_params.push(["client_id", CLIENT_ID]);
    query_params.push(["redirect_uri", "http://localhost:8090"]);
    query_params.push(["scope", "https://www.googleapis.com/auth/drive.file https://www.googleapis.com/auth/userinfo.email"]);

    // ugh fuck you rust. have to do this in two steps
    let query_string_arr: Vec<String> = query_params
      .iter()
      .map(|param| format!("{}={}", param[0], param[1]))
      .collect();

    // now after doing the collect we can finally actually create the query string
    let query_string = query_string_arr.join("&");

    println!("opening browser window");

    open::that(format!("{BASE_LOGIN_URL}?{query_string}")).unwrap();

    let server = Server::http("127.0.0.1:8090").unwrap();

    'outer: for request in server.incoming_requests() {
      if let Some(query) = request.url().split_once("?") {
        let params = query.1.split("&");

        for param in params.into_iter() {
          if let Some((key, value)) = param.split_once("=") {
            if key == "code" {
              self.auth_code = value.to_string();

              println!("received auth code {}", self.auth_code);

              request.respond(Response::from_string("Successfully logged in to Google! This tab can now be closed.")).unwrap();
              break 'outer;
            }
          }
        }
      }
    }

    // make a request to google to get an auth token and refresh token
    let client = reqwest::blocking::Client::new();

    let mut body_params: Vec<[&str; 2]> = Vec::new();

    body_params.push(["code", &self.auth_code]);
    body_params.push(["client_id", CLIENT_ID]);
    body_params.push(["client_secret", CLIENT_SECRET]);
    body_params.push(["redirect_uri", "http://localhost:8090"]);
    body_params.push(["grant_type", "authorization_code"]);

    let params_arr: Vec<String> = body_params
      .iter()
      .map(|param| format!("{}={}", param[0], param[1]))
      .collect();

    let params = params_arr.join("&");

    let response = client.post(BASE_TOKEN_URL)
      .body(
        Body::from(format!("{params}"))
      )
      .header("Content-Type", "application/x-www-form-urlencoded")
      .send()
      .unwrap();

    let json: TokenResponse = response.json().unwrap();

    self.access_token = json.access_token;
    self.refresh_token = json.refresh_token;

    // store these in files for use later
    fs::write("./.access_token", self.access_token.clone()).unwrap();
    fs::write("./.refresh_token", self.refresh_token.clone()).unwrap();

  }

  pub fn logout(&mut self) {
    fs::remove_file("./.access_token").unwrap();
    fs::remove_file("./.refresh_token").unwrap();

    self.access_token = String::new();
    self.refresh_token = String::new();
  }
}