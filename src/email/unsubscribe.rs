// Signed unsubscribe links for optional (non-transactional) emails.
//
// The token is an HS256 JWT carrying the recipient's email, signed with
// CVENOM_UNSUBSCRIBE_SECRET. It has no expiry: an unsubscribe link must keep
// working for as long as the email sits in someone's inbox.
//
// Env vars:
//   CVENOM_UNSUBSCRIBE_SECRET – HMAC secret for the token (required for marketing emails)
//   CVENOM_POSTAL_ADDRESS     – sender's legal name + postal address shown in the footer
//   PUBLIC_BASE_URL           – public backend URL the link points to
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

const PURPOSE: &str = "unsubscribe";

#[derive(Serialize, Deserialize)]
struct Claims {
    sub: String,
    purpose: String,
}

fn secret() -> Option<String> {
    std::env::var("CVENOM_UNSUBSCRIBE_SECRET").ok().filter(|s| !s.is_empty())
}

pub fn postal_address() -> Option<String> {
    std::env::var("CVENOM_POSTAL_ADDRESS").ok().filter(|s| !s.trim().is_empty())
}

pub fn token(email: &str) -> Option<String> {
    let claims = Claims { sub: email.to_string(), purpose: PURPOSE.into() };
    encode(&Header::new(Algorithm::HS256), &claims, &EncodingKey::from_secret(secret()?.as_bytes())).ok()
}

/// Returns the email the token was issued for, if the signature is valid.
pub fn verify(token: &str) -> Option<String> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.required_spec_claims.clear();
    validation.validate_exp = false;
    let data = decode::<Claims>(token, &DecodingKey::from_secret(secret()?.as_bytes()), &validation).ok()?;
    (data.claims.purpose == PURPOSE).then_some(data.claims.sub)
}

/// Public URL that unsubscribes `email` from all optional emails.
pub fn url(email: &str) -> Option<String> {
    let base = std::env::var("PUBLIC_BASE_URL").unwrap_or_else(|_| "https://api.cvenom.com".to_string());
    Some(format!("{}/email/unsubscribe?token={}", base.trim_end_matches('/'), token(email)?))
}

/// Email-prefs JSON with every optional category switched off, merged over the existing prefs.
pub fn disable_all_optional(existing_prefs_json: &str) -> String {
    let mut prefs: serde_json::Map<String, serde_json::Value> =
        serde_json::from_str(existing_prefs_json).unwrap_or_default();
    for key in super::templates::OPTIONAL_KINDS {
        prefs.insert((*key).to_string(), serde_json::Value::Bool(false));
    }
    serde_json::Value::Object(prefs).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_roundtrip_and_prefs() {
        std::env::set_var("CVENOM_UNSUBSCRIBE_SECRET", "test-secret");
        let t = token("Jane@Example.com").unwrap();
        assert_eq!(verify(&t).as_deref(), Some("Jane@Example.com"));
        assert_eq!(verify(&format!("{t}x")), None);

        let prefs: serde_json::Value =
            serde_json::from_str(&disable_all_optional(r#"{"cv_ready":true,"other":1}"#)).unwrap();
        assert_eq!(prefs["cv_ready"], false);
        assert_eq!(prefs["nudge"], false);
        assert_eq!(prefs["other"], 1);

        std::env::set_var("CVENOM_POSTAL_ADDRESS", "CVenom, 1 rue Test, 75001 Paris");
        let kind = crate::email::EmailKind::WinBack { name: "Jane".into() };
        let html = kind.html_body("fr", url("jane@example.com").as_deref());
        assert!(html.contains("/email/unsubscribe?token="));
        assert!(html.contains("Se désabonner"));
        assert!(html.contains("75001 Paris"));
        assert!(!crate::email::EmailKind::AccountDeleted.html_body("en", None).contains("Unsubscribe"));
    }
}
