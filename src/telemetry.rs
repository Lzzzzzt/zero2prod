use axum::Router;
use axum::http::{Request, header::HeaderName};
use tower_http::request_id::{MakeRequestUuid, SetRequestIdLayer};
use tower_http::trace::TraceLayer;
use tracing::{Subscriber, error, info_span, subscriber::set_global_default};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{EnvFilter, Registry, fmt::MakeWriter, layer::SubscriberExt};

pub fn create_subscriber<S>(
    name: impl Into<String>,
    level: impl Into<String>,
    out: S,
) -> impl Subscriber + Sync + Send
where
    S: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level.into()));
    let formatting_layer = BunyanFormattingLayer::new(name.into(), out);

    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

pub fn setup_subscriber(subscriber: impl Subscriber + Sync + Send) {
    set_global_default(subscriber).expect("Failed to set subscriber");
}

pub fn with_request_id(router: Router) -> Router {
    let x_request_id = HeaderName::from_static("x-request-id");

    let service = tower::ServiceBuilder::new()
        .layer(SetRequestIdLayer::new(
            x_request_id.clone(),
            MakeRequestUuid,
        ))
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &Request<_>| {
                // Log the request id as generated.}
                let request_id = request.headers().get("x-request-id");

                match request_id {
                    Some(request_id) => info_span!(
                        "http_request",
                        request_id = ?request_id,
                    ),
                    None => {
                        error!("could not extract request_id");
                        info_span!("http_request")
                    }
                }
            }),
        );

    router.layer(service)
}
