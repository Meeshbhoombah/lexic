use chrono::Utc;
use rand::RngCore;
use sha2::{Digest, Sha256};
use sqlx::PgPool;

use crate::{
    errors::AppError,
    models::session::SessionResponse,
    repositories,
};

const TOKEN_BYTES: usize = 32;

pub struct SessionResult {
    pub session: SessionResponse,
    pub token: String,
}

fn hash_token(token: &str) -> Vec<u8> {
    Sha256::digest(token.as_bytes()).to_vec()
}

fn is_valid_token(token: &str) -> bool {
    token.len() == TOKEN_BYTES * 2
        && token.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn generate_token() -> String {
    let mut bytes = [0_u8; TOKEN_BYTES];
    rand::rng().fill_bytes(&mut bytes);

    let mut token = String::with_capacity(TOKEN_BYTES * 2);

    for byte in bytes {
        use std::fmt::Write as _;
        let _ = write!(token, "{byte:02x}");
    }

    token
}

pub async fn establish(
    db: &PgPool,
    existing_token: Option<&str>,
) -> Result<SessionResult, AppError> {
    if let Some(token) = existing_token.filter(|token| is_valid_token(token)) {
        if let Some(session) =
            repositories::sessions::find_active(db, &hash_token(token)).await?
        {
            if session.expires_at > Utc::now() {
                return Ok(SessionResult {
                    session,
                    token: token.to_owned(),
                });
            }
        }
    }

    let token = generate_token();
    let session =
        repositories::sessions::create(db, &hash_token(&token)).await?;

    Ok(SessionResult { session, token })
}
