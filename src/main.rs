extern crate actix_web;
extern crate dotenv;
extern crate env_logger;
extern crate listenfd;
extern crate mime;
extern crate reqwest;

#[macro_use]
extern crate log;

#[macro_use]
extern crate failure;

#[macro_use]
extern crate serde_derive;
extern crate serde;

#[macro_use]
extern crate structopt;

use actix_web::middleware::Logger;
use actix_web::{http, App, Result};
use failure::{Error, ResultExt};
use listenfd::ListenFd;
use std::sync::Arc;
use structopt::StructOpt;

mod github_api;
mod judgement;
mod options;
mod server;
mod token_store;
mod utils;
mod webhook;

use github_api::Client as GithubClient;
use options::AppOptions;
use server::ServerState;
use utils::log_error_trace;

fn main() {
    dotenv::dotenv().ok();

    let app_options = AppOptions::from_args();
    app_options.init_logger();

    match run(app_options) {
        Ok(_) => {}
        Err(err) => {
            log_error_trace(err.as_fail());
            ::std::process::exit(1);
        }
    }
}

fn run(app_options: AppOptions) -> Result<(), Error> {
    let api_client = api_client(&app_options).context("Could not initialize Github API")?;
    let state = Arc::new(ServerState::new(api_client));

    let mut listenfd = ListenFd::from_env();
    let mut server = actix_web::server::new(move || {
        App::with_state(Arc::clone(&state))
            .middleware(Logger::default())
            .resource("/", |r| r.f(server::handle_index))
            .resource("/webhook", |r| {
                r.method(http::Method::POST).with(server::handle_webhook)
            })
    });

    server = if let Some(l) = listenfd.take_tcp_listener(0).unwrap() {
        server.listen(l)
    } else {
        server.bind(&app_options.bind).unwrap()
    };

    Ok(server.run())
}

fn api_client(app_options: &AppOptions) -> Result<GithubClient, Error> {
    let private_key =
        ::std::fs::read(&app_options.private_key_path).context("Failed to load private key")?;

    Ok(GithubClient::new(app_options.github_app_id, private_key))
}
