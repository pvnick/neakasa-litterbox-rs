use serde_json::Value;

pub type Result<T> = std::result::Result<T, Error>;

/// Errors deliberately exclude response bodies, URLs with queries, and credentials.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("not authenticated")]
    NotAuthenticated,
    #[error("session expired (code {0})")]
    SessionExpired(i64),
    #[error("invalid credentials (code {0})")]
    InvalidCredentials(i64),
    #[error("authentication failed (code {0})")]
    Authentication(i64),
    #[error("API rejected request (code {0})")]
    Api(i64),
    #[error("HTTP request failed (status {0})")]
    Http(u16),
    #[error("network request failed")]
    Transport,
    #[error("invalid response format")]
    InvalidResponse,
    #[error("cryptographic operation failed")]
    Crypto,
    #[error("invalid input: {0}")]
    InvalidInput(&'static str),
    #[error("state file operation failed")]
    State,
    #[error("MQTT connection failed")]
    Mqtt,
    #[error("operation timed out")]
    Timeout,
}
impl Error {
    pub fn code(&self) -> Option<i64> {
        match self {
            Self::SessionExpired(c)
            | Self::InvalidCredentials(c)
            | Self::Authentication(c)
            | Self::Api(c) => Some(*c),
            _ => None,
        }
    }
    pub fn kind(&self) -> &'static str {
        match self {
            Self::SessionExpired(_) => "SessionExpiredError",
            Self::InvalidCredentials(_) => "InvalidCredentialsError",
            Self::Authentication(_) => "AuthenticationError",
            Self::Api(_) => "ApiError",
            Self::Http(_) | Self::Transport | Self::Timeout | Self::Mqtt => "TransportError",
            Self::NotAuthenticated => "NeakasaError",
            Self::State => "StateError",
            Self::Crypto => "CryptoError",
            Self::InvalidInput(_) => "ValidationError",
            Self::InvalidResponse => "InvalidResponseError",
        }
    }
}
impl From<serde_json::Error> for Error {
    fn from(_: serde_json::Error) -> Self {
        Self::InvalidResponse
    }
}
impl From<std::io::Error> for Error {
    fn from(_: std::io::Error) -> Self {
        Self::State
    }
}
pub(crate) fn unwrap_rest(value: Value) -> Result<Value> {
    unwrap_rest_context(value, true)
}
pub(crate) fn unwrap_rest_context(value: Value, auth: bool) -> Result<Value> {
    let code = value["code"].as_i64().unwrap_or(-1);
    if code != 0 && !auth {
        return Err(Error::Api(code));
    }
    match code {
        0 => Ok(value["data"].clone()),
        1007 | 3026 | 3027 => Err(Error::SessionExpired(code)),
        3019 | 10060 | 10061 | 10192 => Err(Error::InvalidCredentials(code)),
        _ => Err(Error::Authentication(code)),
    }
}
pub(crate) fn unwrap_iot(value: Value) -> Result<Value> {
    match value["code"].as_i64().unwrap_or(-1) {
        200 => Ok(value["data"].clone()),
        401 => Err(Error::SessionExpired(401)),
        code => Err(Error::Api(code)),
    }
}
