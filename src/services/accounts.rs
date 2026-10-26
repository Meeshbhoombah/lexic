use sqlx::PgPool;

use lexe::{
    config::WalletEnvConfig,
    types::auth::{ClientCredentials, CredentialsRef},
    wallet::LexeWallet,
};

use crate::{errors::AppError, models::account::CreateAccountResponse, repositories};

pub async fn create(db: &PgPool, credential: &str) -> Result<CreateAccountResponse, AppError> {
    if credential.is_empty() || credential.len() > 16_384 {
        return Err(AppError::BadRequest);
    }

    let client_credentials =
        ClientCredentials::from_string(credential).map_err(|_| AppError::Unauthorized)?;

    let credentials = CredentialsRef::from(&client_credentials);

    let wallet = LexeWallet::load_or_fresh(WalletEnvConfig::mainnet(), credentials, None)
        .map_err(|_| AppError::Unauthorized)?;

    // Authenticate against Lexe using the supplied scoped credential.
    // The browser's claim that authorization succeeded is not trusted.
    wallet
        .node_info()
        .await
        .map_err(|_| AppError::Unauthorized)?;

    tracing::info!("Lexe credential verified; creating Lexic account");

    repositories::accounts::create(db).await
}
