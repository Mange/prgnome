use super::prelude::*;

pub fn handle_index(_req: &HttpRequest<Arc<ServerState>>) -> &'static str {
    "prgnome is running at this location. Post webhooks to /webhooks!"
}
