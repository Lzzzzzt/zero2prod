use std::error::Error;

use zero2prod::{
    App,
    config::get_config,
    telemetry::{create_subscriber, setup_subscriber},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let subscriber = create_subscriber("zero2prod", "info", std::io::stdout);
    setup_subscriber(subscriber);

    let config = get_config().await?;
    App::build(config).await?.run().await?;

    Ok(())
}
