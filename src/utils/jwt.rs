// JWT工具

use jsonwebtoken::{
    Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation, decode, encode,
};

use crate::config::auth::JWT_SECRET;
use crate::errors::app_error::AppError;
use crate::schemas::auth::Claims;
use crate::unauthorized;

pub fn create_access_token(data: Claims) -> Result<String, jsonwebtoken::errors::Error> {
    // The header will automatically use HS256 as the algorithm
    let token = encode(
        &Header::default(),
        &data,
        &EncodingKey::from_secret(JWT_SECRET.as_ref()),
    )?;

    Ok(token)
}

pub fn decode_token(token: &str) -> Result<Claims, AppError> {
    let decoded: TokenData<Claims> = decode(
        token,
        &DecodingKey::from_secret(JWT_SECRET.as_ref()),
        &Validation::default(),
    )
    .map_err(|_| unauthorized!("Decode token error".to_string()))?;

    Ok(decoded.claims)
}
