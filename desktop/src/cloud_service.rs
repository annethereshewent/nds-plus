use std::{fs, path::Path};

use reqwest::{blocking::{Body, Client, Response}, Error, StatusCode};
use serde::{Deserialize, Serialize};
use tiny_http::Server;

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
  refresh_token: Option<String>,
  token_type: String,
  expires_in: usize,
  scope: String
}

#[derive(Serialize, Deserialize)]
struct DriveResponse {
  files: Vec<File>
}

#[derive(Serialize, Deserialize, Debug)]
struct File {
  id: String,
  name: String,
  parents: Vec<String>
}

#[derive(Clone)]
pub struct CloudService {
  access_token: String,
  refresh_token: String,
  auth_code: String,
  pub logged_in: bool,
  client: Client,
  ds_folder_id: String
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
      logged_in: access_token != "",
      client: Client::new(),
      ds_folder_id: String::new()
    }
  }

  fn refresh_login(&mut self) {
    let mut body_params: Vec<[&str; 2]> = Vec::new();

    body_params.push(["client_id", CLIENT_ID]);
    body_params.push(["client_secret", CLIENT_SECRET]);
    body_params.push(["grant_type", "refresh_token"]);
    body_params.push(["refresh_token", &self.refresh_token]);

    let params = Self::generate_params_string(body_params);

    let token_response = self.client.post(BASE_TOKEN_URL)
      .header("Content-Type", "application/x-www-form-urlencoded")
      .body(Body::from(params))
      .send()
      .unwrap();

    if token_response.status() == StatusCode::OK {
      let json: Result<TokenResponse, Error> = token_response.json();

      if json.is_ok() {
        let json = json.unwrap();

        self.access_token = json.access_token;

        fs::write("./.access_token", self.access_token.clone()).unwrap();
      } else {
        let error = json.err().unwrap();

        self.logout();

        panic!("{:?}", error);
      }
    }
  }

  pub fn check_for_ds_folder(&mut self) {
    let mut query_params: Vec<[&str; 2]> = Vec::new();

    query_params.push(["q", "mimeType = \"application/vnd.google-apps.folder\" and name=\"ds-saves\""]);
    query_params.push(["fields", "files/id,files/parents,files/name"]);

    let query_string = Self::generate_params_string(query_params);

    let url = format!("https://www.googleapis.com/drive/v3/files?{query_string}");

    // I FUCKING HATE RUST FUCK YOU
    // YOU MAKE IT ******** IMPOSSIBLE ******* TO WRITE DRY CODE
    let response = self.client
      .get(url.clone())
      .header("Authorization", format!("Bearer {}", self.access_token))
      .send()
      .unwrap();

    if response.status() == StatusCode::UNAUTHORIZED {
      self.refresh_login();

      // HERE I HAVE TO REPEAT THIS WHOLE CODE SNIPPET BECAUSE MUH MOVED BORROWED VALUES
      // EAT A FUCKING DICK, RUST
      let response = self.client
        .get(url)
        .header("Authorization", format!("Bearer {}", self.access_token))
        .send()
        .unwrap();

      if response.status() == StatusCode::OK {
       self.process_folder_response(response);
      } else {
        panic!("Could not refresh token with Google API");
      }
    } else if response.status() == StatusCode::OK {
      self.process_folder_response(response);
    } else {
      panic!("An unexpected error occurred while using Google API");
    }
  }

  fn process_folder_response(&mut self, response: Response) {
    let json: DriveResponse = response.json().unwrap();

    if let Some(folder) = json.files.get(0) {
      self.ds_folder_id = folder.id.clone();
    } else {
      // create the ds folder

      // here's another painful API request because i can't DRY it up thanks to Rust
      let url = "https://www.googleapis.com/drive/v3/files?uploadType=media";

      let response = self.client
        .post(url)
        .header("Authorization", format!("Bearer {}", self.access_token))
        .header("Content-Type", "application/vnd.google-apps.folder")
        .send()
        .unwrap();

      // here's the annoying very unDRY part....
      // TODO: **SOMEHOW** find a way to DRY this up.
      if response.status() == StatusCode::UNAUTHORIZED {
        self.refresh_login();

        let response = self.client
          .post(url)
          .header("Authorization", format!("Bearer {}", self.access_token))
          .header("Content-Type", "application/vnd.google-apps.folder")
          .send()
          .unwrap();


        if response.status() == StatusCode::OK {
          let json: DriveResponse = response.json().unwrap();

          if let Some(folder) = json.files.get(0) {
            self.ds_folder_id = folder.id.clone();
          } else {
            panic!("Could not create DS folder");
          }
        } else {
          panic!("Could not create DS folder");
        }
      } else if response.status() == StatusCode::OK {
        let json: DriveResponse = response.json().unwrap();

        if let Some(folder) = json.files.get(0) {
          self.ds_folder_id = folder.id.clone();
        } else {
          panic!("Could not create DS folder");
        }
      } else {
        panic!("Could not create DS folder");
      }
    }
  }

  pub fn get_save(&mut self, game_name: &str) -> Vec<u8> {
    self.check_for_ds_folder();

    let json = self.get_save_info(game_name);

    if let Some(file) = json.files.get(0) {
      let url = format!("https://www.googleapis.com/drive/v3/files/{}?alt=media", file.id);

      // time for some repetition! woo!
      let response = self.client
        .get(url.clone())
        .header("Authorization", format!("Bearer {}", self.access_token))
        .send()
        .unwrap();

      if response.status() == StatusCode::UNAUTHORIZED {
        self.refresh_login();

        let response = self.client
          .get(url)
          .header("Authorization", format!("Bearer {}", self.access_token))
          .send()
          .unwrap();

        if response.status() == StatusCode::OK {
          return response.bytes().unwrap().to_vec();
        }
      } else if response.status() == StatusCode::OK {
        return response.bytes().unwrap().to_vec();
      }
    }

    return Vec::new();
  }

  fn get_save_info(&mut self, game_name: &str) -> DriveResponse {
    let mut query_params: Vec<[&str; 2]> = Vec::new();

    let query = &format!("name = \"{game_name}\" and parents in \"{}\"", self.ds_folder_id);

    // rust ONCE AGAIN trying to protect me from something that will never happen but makes me write shitty code.
    let mut _useless = String::new();

    query_params.push(["q", url_escape::encode_component_to_string(query, &mut _useless)]);
    query_params.push(["fields", "files/id,files/parents,files/name"]);

    let query_string = Self::generate_params_string(query_params);

    let url = format!("https://www.googleapis.com/drive/v3/files?{query_string}");

    // brace yourself for annoying unDRY code!
    let response = self.client
      .get(url.clone())
      .header("Authorization", format!("Bearer {}", self.access_token))
      .send()
      .unwrap();

    if response.status() == StatusCode::UNAUTHORIZED {
      self.refresh_login();

      let response = self.client
        .get(url)
        .header("Authorization", format!("Bearer {}", self.access_token))
        .send()
        .unwrap();

      if response.status() == StatusCode::OK {
        return response.json::<DriveResponse>().unwrap();
      } else {
        panic!("Could not get save info");
      }
    } else if response.status() == StatusCode::OK {
      return response.json::<DriveResponse>().unwrap();
    } else {
      panic!("{:?}", response.text());
    }
  }

  pub fn generate_params_string(params: Vec<[&str; 2]>) -> String {
    let param_arr: Vec<String> = params
      .iter()
      .map(|param| format!("{}={}", param[0], param[1]))
      .collect();

    // now after doing the collect we can finally actually create the query string
    let string = param_arr.join("&");

    string
  }

  pub fn login(&mut self) {
    let mut query_params: Vec<[&str; 2]> = Vec::new();

    query_params.push(["response_type", "code"]);
    query_params.push(["client_id", CLIENT_ID]);
    query_params.push(["redirect_uri", "http://localhost:8090"]);
    query_params.push(["scope", "https://www.googleapis.com/auth/drive.file https://www.googleapis.com/auth/userinfo.email"]);

    let query_string = Self::generate_params_string(query_params);

    open::that(format!("{BASE_LOGIN_URL}?{query_string}")).unwrap();

    let server = Server::http("127.0.0.1:8090").unwrap();

    'outer: for request in server.incoming_requests() {
      if let Some(query) = request.url().split_once("?") {
        let params = query.1.split("&");

        for param in params.into_iter() {
          if let Some((key, value)) = param.split_once("=") {
            if key == "code" {
              self.auth_code = value.to_string();

              request.respond(tiny_http::Response::from_string("Successfully logged in to Google! This tab can now be closed.")).unwrap();
              break 'outer;
            }
          }
        }
      }
    }

    // make a request to google to get an auth token and refresh token
    let mut body_params: Vec<[&str; 2]> = Vec::new();

    body_params.push(["code", &self.auth_code]);
    body_params.push(["client_id", CLIENT_ID]);
    body_params.push(["client_secret", CLIENT_SECRET]);
    body_params.push(["redirect_uri", "http://localhost:8090"]);
    body_params.push(["grant_type", "authorization_code"]);

    let params = Self::generate_params_string(body_params);

    let response = self.client.post(BASE_TOKEN_URL)
      .body(
        Body::from(format!("{params}"))
      )
      .header("Content-Type", "application/x-www-form-urlencoded")
      .send()
      .unwrap();


    if response.status() == StatusCode::OK {
      let json: TokenResponse = response.json().unwrap();

      self.access_token = json.access_token;
      self.refresh_token = json.refresh_token.unwrap();

      // store these in files for use later
      fs::write("./.access_token", self.access_token.clone()).unwrap();
      fs::write("./.refresh_token", self.refresh_token.clone()).unwrap();
    }
  }

  pub fn logout(&mut self) {
    fs::remove_file("./.access_token").unwrap();
    fs::remove_file("./.refresh_token").unwrap();

    self.access_token = String::new();
    self.refresh_token = String::new();
    self.logged_in = false;
  }
}