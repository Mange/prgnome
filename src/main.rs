extern crate actix_web;
extern crate listenfd;
extern crate mime;

#[macro_use]
extern crate failure;

#[macro_use]
extern crate serde_derive;
extern crate serde;

use actix_web::{http, server, App, HttpMessage, HttpRequest, HttpResponse, Result};
use listenfd::ListenFd;

mod webhook;

use webhook::{Event, WebhookError};

#[derive(Debug)]
struct EventName(String);

impl<S> actix_web::FromRequest<S> for EventName {
    type Config = ();
    type Result = Result<EventName>;

    fn from_request(req: &HttpRequest<S>, _cfg: &Self::Config) -> Self::Result {
        let header = req
            .headers()
            .get("Github-Event")
            .ok_or_else(|| format_err!("No Github-Event header present"))?;

        header
            .to_str()
            .map(String::from)
            .map(EventName)
            .map_err(|err| {
                actix_web::Error::from(format_err!("Cannot parse Github-Event header: {}", err))
            })
    }
}

fn index(_req: &HttpRequest) -> &'static str {
    "Hello world"
}

fn webhook(event_name: EventName, body: String) -> Result<String> {
    let event = Event::parse_json(&event_name.0, &body)?;
    println!("{:#?}", event);
    Ok(format!("OK"))
}

impl actix_web::ResponseError for WebhookError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::new(http::StatusCode::BAD_REQUEST)
    }
}

fn main() {
    let mut listenfd = ListenFd::from_env();
    let mut server = server::new(|| {
        App::new()
            .resource("/", |r| r.f(index))
            .resource("/webhook", |r| r.method(http::Method::POST).with(webhook))
    });

    server = if let Some(l) = listenfd.take_tcp_listener(0).unwrap() {
        server.listen(l)
    } else {
        server.bind("127.0.0.1:4567").unwrap()
    };

    server.run();
}
