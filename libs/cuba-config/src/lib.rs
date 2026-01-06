pub fn load_config<T: serde::de::DeserializeOwned>(
    service_name: &str,
) -> Result<T, config::ConfigError> {
    // Load `.env` if present to support local development without requiring `make`/shell exports.
    // This is a no-op if the file does not exist.
    let _ = dotenvy::dotenv();

    // Environment variables cannot contain `-`, so normalize service names like `auth-service`
    // to match `.env` patterns such as `AUTH_SERVICE__SERVER_ADDR`.
    let env_prefix = service_name.replace('-', "_").to_ascii_uppercase();

    let config = config::Config::builder()
        .add_source(config::File::with_name(&format!("config/{}", service_name)))
        .add_source(config::Environment::with_prefix(&env_prefix).separator("__"))
        .build()?;
    config.try_deserialize()
}
