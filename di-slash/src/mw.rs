use crate::discord::verify_sig;
use crate::error::Error;
use axum::body::Body;
use axum::http::{HeaderName, HeaderValue};
use axum::{extract::Request, middleware::Next, response::Response};
use axum_extra::headers::Header;
use axum_extra::TypedHeader;
use http_body_util::BodyExt;
use tracing::info;

pub async fn validate_req(
    TypedHeader(timestamp): TypedHeader<SignatureTimestamp>,
    TypedHeader(signature): TypedHeader<SignatureEd25519>,
    request: Request,
    next: Next,
) -> crate::error::Result<Response> {
    let (parts, body) = request.into_parts();
    let bytes = body.collect().await.expect("to_bytes").to_bytes();

    let body_str = String::from_utf8(bytes.to_vec()).map_err(|_| Error::InvalidBody)?;
    let valid_req = verify_sig(
        body_str.as_str(),
        signature.value(),
        timestamp.value(),
        // public key
        "a5d1148be5d078b851d180a46134f24bceb1e6a02ff884c0c5bf2fc2ea85f408".to_string(),
    )?;

    if !valid_req {
        return Err(Error::BadSignature);
    }
    let request = Request::from_parts(parts, Body::from(bytes));

    let response = next.run(request).await;

    info!("Completed request!");
    info!("Response: {:?}", response);
    Ok(response)
}

static X_SIGNATURE_TIMESTAMP: HeaderName = HeaderName::from_static("x-signature-timestamp");

#[derive(Debug, Clone)]
pub struct SignatureTimestamp(String);

impl SignatureTimestamp {
    pub fn new(timestamp: String) -> Self {
        Self(timestamp)
    }
    pub fn value(&self) -> &str {
        self.0.as_str()
    }
}

impl Header for SignatureTimestamp {
    fn name() -> &'static HeaderName {
        &X_SIGNATURE_TIMESTAMP
    }
    fn encode<E: Extend<HeaderValue>>(&self, values: &mut E) {
        let value = HeaderValue::from_str(self.0.as_str());
        values.extend(value);
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, axum_extra::headers::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i HeaderValue>,
    {
        let value = values
            .next()
            .ok_or_else(axum_extra::headers::Error::invalid)?;
        let s = value
            .to_str()
            .map_err(|_| axum_extra::headers::Error::invalid())?;

        Ok(SignatureTimestamp(s.to_string()))
    }
}
static X_SIGNATURE_ED25519: HeaderName = HeaderName::from_static("x-signature-ed25519");

#[derive(Debug, Clone)]
pub struct SignatureEd25519(String);

impl SignatureEd25519 {
    pub fn new(signature: String) -> Self {
        Self(signature)
    }

    pub fn value(&self) -> &str {
        self.0.as_str()
    }
}

impl Header for SignatureEd25519 {
    fn name() -> &'static HeaderName {
        &X_SIGNATURE_ED25519
    }
    fn encode<E: Extend<HeaderValue>>(&self, values: &mut E) {
        let value = HeaderValue::from_str(self.0.as_str());
        values.extend(value);
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, axum_extra::headers::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i HeaderValue>,
    {
        let value = values
            .next()
            .ok_or_else(axum_extra::headers::Error::invalid)?;
        let s = value
            .to_str()
            .map_err(|_| axum_extra::headers::Error::invalid())?;

        Ok(SignatureEd25519(s.to_string()))
    }
}
