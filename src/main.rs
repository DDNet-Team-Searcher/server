use ddts_server::{
    application::Application,
    config::get_config,
    telemetry::{get_subscriber, init_subscriber},
};
use dotenv::dotenv;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let default_filter_level = "debug";
    let subscriber_name = "ddts".to_string();

    dotenv().ok();
    init_subscriber(get_subscriber(
        subscriber_name,
        default_filter_level,
        std::io::stdout,
    ));
    let config = get_config().expect("failed to get config");

    let app = Application::new(config)
        .await
        .expect("failed to construct app");
    app.run()
        .await
        .expect("error occured during running the app");

    Ok(())
}
