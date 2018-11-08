use super::prelude::*;

pub fn handle_index(_req: &HttpRequest<Arc<ServerState>>) -> &'static str {
    "Hello world"
}
