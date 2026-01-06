pub fn init_subscriber(service_name: &str) {
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_target(true)
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_level(true)
        .with_ansi(true)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set global default subscriber");

    tracing::info!(service = %service_name, "Telemetry initialized");
}
