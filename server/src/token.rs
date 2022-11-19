use rocket::request::{Request, Outcome, FromRequest};
use rocket::http::Status;
use urlencoding::decode;

use chrono::prelude::*;
use chrono::naive::NaiveDateTime;
use sha2::{Sha256, Digest};
use base64;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub struct Token {
    user: String,
    token: String
}

#[derive(Debug)]
pub enum ApiTokenError {
    Missing,
    Invalid,
}

impl Token {
    pub fn user(&self) -> String {
        format!("{}", calculate_hash(&self.user))
    }
}

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

fn validate_and_build(token: &str, query: &str) -> Option<Token> {
    let split: Vec<&str> = token.splitn(4, ".").collect();
    if split.len() != 4 {
        print!("wrong elements amount");
        return None
    }

    let (nonce, timestamp_s, user, signature) = (split[0], split[1], split[2], split[3]);
    if nonce.len() < 10 {
        return None
    }

    let timestamp = timestamp_s.parse::<i64>().unwrap_or(0);
    if timestamp == 0 {
        return None
    }

    let native_datetime = NaiveDateTime::from_timestamp_opt(timestamp, 0).unwrap_or_default();
    if native_datetime == NaiveDateTime::default() {
        return None
    }

    let elapsed = Utc::now() - DateTime::from_utc(native_datetime, Utc);
    if elapsed < chrono::Duration::seconds(-5) || elapsed > chrono::Duration::minutes(1) {
        return None
    }

    let user_name = decode(user);
    if user_name.is_err() {
        return None
    }

    let mut hasher = Sha256::new(); // ${user}_${password}_${query}_${nonce}_${timestamp}

    hasher.update(
        user_name.unwrap().as_ref()
    );
    hasher.update("_");

    hasher.update("S_:C-u3\\i-Ts)[&m?Z[F" /*password_for_user(self.user)*/);
    hasher.update("_");

    hasher.update(query);
    hasher.update("_");

    hasher.update(nonce);
    hasher.update("_");

    hasher.update(timestamp_s);

    let result = hasher.finalize();
    let mine = base64::encode(result);
    println!("Got: {}, Sign: {}, Calc: {}",
        token,
        signature,
        mine
    );

    if mine != signature {
        return None
    }
    
    Some(Token {
        token: "reducted".to_string(),
        user: user.to_string()
    })
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Token {
    type Error = ApiTokenError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let token = req.headers().get_one("Authorization");
        let query = req.uri().query().map(|q| q.as_str()).unwrap_or("");
        
        token.map_or(
            Outcome::Failure((Status::Unauthorized, ApiTokenError::Missing)),
            |token| {
                validate_and_build(token, query)
                .map(Outcome::Success)
                .unwrap_or(Outcome::Failure((Status::Unauthorized, ApiTokenError::Invalid)))
            }
        )
    }
}
