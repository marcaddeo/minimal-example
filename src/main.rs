use std::net::SocketAddr;

use axum::{
    extract::State,
    middleware::map_response_with_state,
    response::{IntoResponse, Redirect, Response},
    routing::get,
    Router,
};
use axum_messages::{Messages, MessagesManagerLayer};
use tower_sessions::{MemoryStore, SessionManagerLayer};

async fn set_messages_handler(messages: Messages) -> impl IntoResponse {
    messages
        .info("Hello, world!")
        .debug("This is a debug message.");

    Redirect::to("/read-messages")
}

async fn read_messages_handler(messages: Messages) -> impl IntoResponse {
    // Messages always empty here because they've been consumed by the response middlewares
    let messages = messages
        .into_iter()
        .map(|message| format!("{}: {}", message.level, message))
        .collect::<Vec<_>>()
        .join(", ");

    if messages.is_empty() {
        "No messages yet!".to_string()
    } else {
        messages
    }
}

struct MyExt {}

async fn error_page<B>(
    State(_state): State<MyState>,
    messages: Messages,
    response: Response<B>,
) -> Response<B> {
    // Messages is always empty here because the render_page middleware runs first.
    if let Some(_ext) = response.extensions().get::<MyExt>() {
        let _messages = messages.into_iter().collect::<Vec<_>>();

        // Create an error page that contains messages, alter response, etc.
        // Makes a call to render_page(), passing the messages along

        response
    } else {
        response
    }
}

struct MyOtherExt {}

async fn render_page<B>(
    State(_state): State<MyState>,
    messages: Messages,
    response: Response<B>,
) -> Response<B> {
    // Going to / adds a message, and redirects to /read-messages but messages will be consumed
    // here on the response, and /read-messages will read "No messages yet!".
    //
    // Messages are consumed here even if I don't actually use them because this condition isn't
    // met.
    if let Some(_ext) = response.extensions().get::<MyOtherExt>() {
        let _messages = messages.into_iter().collect::<Vec<_>>();

        // Create a page that contains messages, alter response, etc.

        response
    } else {
        response
    }
}

#[derive(Clone)]
struct MyState {}

#[tokio::main]
async fn main() {
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store).with_secure(false);

    let state = MyState {};

    let app = Router::new()
        .route("/", get(set_messages_handler))
        .route("/read-messages", get(read_messages_handler))
        .layer(map_response_with_state(state.clone(), render_page))
        .layer(map_response_with_state(state.clone(), error_page))
        .layer(MessagesManagerLayer)
        .layer(session_layer)
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
