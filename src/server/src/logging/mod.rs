use tracing_subscriber::layer::SubscriberExt as _;

pub mod logging;

pub async fn init_components() {
    // Setup logging+tracing layers.
    let (route_layer, _) = tracing_subscriber::reload::Layer::new(logging::default_log_route_layer());
    let (stderr_layer, _) = tracing_subscriber::reload::Layer::new(logging::default_log_stderr_layer());
    let level_layer = logging::default_log_levels_layer();

    let subscriber = tracing_subscriber::registry()
        .with(route_layer)
        .with(stderr_layer)
        .with(level_layer);

    // set the subscriber as the default for the application
    tracing::subscriber::set_global_default(subscriber).unwrap();
}
