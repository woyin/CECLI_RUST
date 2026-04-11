use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct LocalAuth {
    token: Arc<String>,
}

impl LocalAuth {
    pub fn generate() -> Self {
        Self {
            token: Arc::new(Uuid::new_v4().to_string()),
        }
    }

    pub fn from_token(token: String) -> Self {
        Self {
            token: Arc::new(token),
        }
    }

    pub fn token(&self) -> &str {
        &self.token
    }

    pub fn validate(&self, provided: &str) -> bool {
        provided == self.token.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_and_validate() {
        let auth = LocalAuth::generate();
        let token = auth.token().to_string();
        assert!(auth.validate(&token));
        assert!(!auth.validate("wrong-token"));
    }

    #[test]
    fn from_token_and_validate() {
        let auth = LocalAuth::from_token("my-secret-token".to_string());
        assert_eq!(auth.token(), "my-secret-token");
        assert!(auth.validate("my-secret-token"));
        assert!(!auth.validate("other-token"));
    }
}
