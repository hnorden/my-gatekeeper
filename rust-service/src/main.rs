#![feature(proc_macro_hygiene, decl_macro)]

#![warn(rust_2018_idioms)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate serde_derive;

use rocket::http::Status;
use rocket::request::{self, FromRequest, Request};
use rocket::Outcome;
use rocket_contrib::json::{Json, JsonValue};



#[derive(Default, Debug, Serialize, Deserialize)]
struct AuthHeader {
    authorization: String,
    roles: Vec<String>,
    subject: String,
    token: String,
    username: String,
}

#[derive(Debug)]
enum AuthHeaderError {
    Missing,
}

impl<'a, 'r> FromRequest<'a, 'r> for AuthHeader {
    type Error = AuthHeaderError;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        let authorization = request.headers().get("Authorization").last();
        let roles: Vec<_> = request.headers().get("X-Auth-Roles").collect::<Vec<&str>>();
        let subject = request.headers().get("X-Auth-Subject").last();
        let token = request.headers().get("X-Auth-Token").last();
        let username = request.headers().get("X-Auth-Username").last();

        let mut result = AuthHeader::default();
        if let Some(x) = authorization {
            result.authorization = x.to_string();
        }
        if roles.len() > 1 {
            result.roles = roles.into_iter().map(|x| x.to_string()).collect();
        }
        if let Some(x) = subject {
            result.subject = x.to_string();
        }
        if let Some(x) = token {
            result.token = x.to_string();
        }
        if let Some(x) = username {
            result.username = x.to_string();
        }

        match result.authorization.len() {
            0 => {
                println!("Failure: {:?}", result);
                Outcome::Failure((Status::Unauthorized, AuthHeaderError::Missing))
            }
            _ => {
                println!("Success: {:?}", result);
                Outcome::Success(result)
            }
        }
    }
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/health?<next>")]
fn health(next: Option<String>, auth: AuthHeader) -> JsonValue {
    match next {
        Some(url) => {
            let remote_response = call_next(&auth, &url);
            match remote_response {
                Ok(remote) => json!({
                    "status": "OK",
                    "username": auth.username,
                    "next": {
                        "url": url,
                        "remote": remote.1
                    }
                }),
                Err(remote) => {
                    json!({
                        "status": "failure",
                        "username": auth.username,
                        "next": {
                            "url": url,
                            "remote": remote.to_string()
                        }
                    })
                }
            }
        },
        None => json!({
            "status": "OK",
            "username": auth.username
        })
    }
}

#[get("/authorization-header")]
fn auth_header(auth: AuthHeader) -> Json<AuthHeader> {
    // format!("Authorization header: {:?}", auth)
    Json(auth)
}

#[get("/authorization-header/<key>")]
fn auth_header_by_key(key: String, auth: AuthHeader) -> Option<JsonValue> {
    let json: JsonValue = json!(auth);
    let value = json.get(key);
    match value {
        Some(v) => Some(json!(v)),
        _ => None,
    }
}

#[tokio::main]
async fn call_next(auth: &AuthHeader, url: &String) -> Result<(reqwest::StatusCode, JsonValue), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let resp = client.get(url)
        .header("Authorization", &auth.authorization)
        .header("X-Auth-Username", &auth.username)
        .send()
        .await?;

    let status = resp.status();
    match resp.status() {
        s if s.is_success() => {
            let json = resp.json()
                .await?;
            println!("Response: {:?}", &json);
            Ok((status, json))
        },
        s => Err(Box::from(s.to_string()))
    }
}

fn main() {
    rocket::ignite()
        .mount("/", routes![index, health, auth_header, auth_header_by_key])
        .launch();
}

// externing crate for test-only use
// https://doc.rust-lang.org/rust-by-example/testing/dev_dependencies.html
#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_header() {
        let response = auth_header(AuthHeader {
            authorization: String::from("my bearer token"),
            roles: vec![String::from("admin")],
            subject: String::from("1234-1234-1234-1234"),
            token: String::from("my bearer token"),
            username: String::from("myUsername")
        });
        assert_eq!(
            response.authorization,
            "my bearer  token"
        );
    }
    
    #[test]
    fn test_auth_header_by_key() {
        let response = auth_header_by_key(String::from("username"), AuthHeader {
            authorization: String::from("my bearer token"),
            roles: vec![String::from("admin")],
            subject: String::from("1234-1234-1234-1234"),
            token: String::from("my bearer token"),
            username: String::from("myUsername")
        });
        assert!(
            response.is_some()
        );
        assert_eq!(
            response.unwrap().0,
            String::from("myUsername")
        );
    }
}
