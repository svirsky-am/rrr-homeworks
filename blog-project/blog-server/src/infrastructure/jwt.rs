use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use serde::{Serialize, Deserialize};
use chrono::{Utc, Duration};
use crate::domain::DomainError;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub user_id: i64,
    pub username: String,
    pub exp: i64,
}

#[derive(Clone)]
pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl JwtService {
    pub fn new(secret: &str) -> Self {
        let key = secret.as_bytes();
        Self {
            encoding_key: EncodingKey::from_secret(key),
            decoding_key: DecodingKey::from_secret(key),
        }
    }

    pub fn generate_token(&self, user_id: i64, username: String) -> Result<String, DomainError> {
        let exp = Utc::now() + Duration::hours(24);
        let claims = Claims {
            user_id,
            username,
            exp: exp.timestamp(),
        };
        Ok(encode(&Header::default(), &claims, &self.encoding_key)?)
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims, DomainError> {
        let token_data = decode::<Claims>(
            token,
            &self.decoding_key,
            &Validation::new(jsonwebtoken::Algorithm::HS256),
        )?;
        Ok(token_data.claims)
    }
}