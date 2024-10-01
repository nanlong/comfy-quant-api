use bon::Builder;

#[derive(Builder, Debug, Clone)]
#[builder(on(String, into))]
pub struct AccountKey {
    pub api_key: String,
    pub secret_key: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_key_builder() {
        let account = AccountKey::builder()
            .api_key("api_key")
            .secret_key("secret_key")
            .build();

        assert_eq!(account.api_key, "api_key");
        assert_eq!(account.secret_key, "secret_key");
    }
}
