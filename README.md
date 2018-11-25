# prgnome

> It's a "PR Gnome", not a "program nome"!

This is a small bot that you can install in your Github organization to prevent
you from pressing the nice green "Merge" button before the PR is truly ready.

If a PR is tagged with some variant of "Don't merge", "Work in progress",
"Blocked", or contains `fixup!`/`squash!`/"tmp" commits then this app will
produce a "Failed" check on your PR.
This makes the "Merge" button gray again until you rebase or remove the tags.

## Configuration

prgnome supports configuration via command line argument, through ENV
variables, and through `.env` files.

They will be read in this order (first one wins):

  * CLI arguments (like `--github-webhook-secret`)
  * `.env` file (like `GITHUB_WEBHOOK_SECRET`)
  * Normal environment variables (like `GITHUB_WEBHOOK_SECRET`)

A full reference can be found under the `--help` output.

## How to install

1. [Create a new Github App.](https://developer.github.com/apps/building-github-apps/creating-a-github-app/)
   * Name it something you like.
   * Determine which hostname you will deploy it under and input it for the
     URLs below.
     * Webhook URL is `<URL>/webhook`.
     * User authorization callback URL is unused but still required, so enter
       `<URL>/auth/callback`.
   * If you are using HTTPS, then also enable SSL verification!
   * Permissions:
     * Checks: Read & Write
     * Repository contents: Read-only
     * Repository metadata: Read-only
     * Pull requests: Read-only
     * Commit statuses: Read & Write
     * Everything else: No access
   * Subscribe to events:
     * Pull request
2. Download the private key and store the webhook secret somewhere.
3. Convert the private key from PEM format into DER format.
   * Use `contrib/generate_private_key.sh` from this repo, or manually run the
     commands inside that script.
4. Deploy. (See below)
5. Install the app in your organization
   * Click on the "Public Page" of your new application.
   * Then click on "Install".
   * Select which repos you want it to access.

### Deploying using Docker

Deploy `mange/prgnome` and set these environment variables:
  - `BIND` (defaults to `127.0.0.1:8002`)
  - `GITHUB_APP_ID` to the app ID of your newly created app
  - `GITHUB_WEBHOOK_SECRET` to the webhook secret you saved when you created the app
  - `LOG_LEVEL` to the level you want (`error`, `warning`, `verbose`, `debug`)

Also mount your private key (in DER format) as `private_key.der`, or set
`PRIVATE_KEY_PATH` to point to the mounted key if you use some other path.

**Example docker-compose**

```yaml
version: "3.3"

services:
  prgnome:
    image: mange/prgnome:latest
    environment:
      BIND: 0.0.0.0:8989
      GITHUB_APP_ID: 12345
      GITHUB_WEBHOOK_SECRET: REDACTED
    volumes:
      - "/path/to/private_key.der:private_key.der"
    restart_policy:
      condition: on-failure
```

Then place some web server in front to proxy to the `$BIND` address and
terminate SSL. If the web server is not running inside Docker, don't forget to
place a `ports` configuration there to expose your port on the Docker host.

### Deploying using Systemd

Use the units under `contrib/systemd` as a base and build your own setup. The
default setup if you make no changes is this:

  * `prgnome.service` can be enabled to start automatically on boot
  * `prgnome.socket` can be enabled to start the daemon on-demand on first
    connection
  * If both are enabled, the daemon will boot when system is ready but sockets
    will start to get accepted earlier to reduce risks of dropped connections.
  * Environment variables will be read from `/etc/xdg/prgnome/settings.env`,
    which is where you can put things like `LOG_LEVEL`, `GITHUB_APP_ID`, and
    `GITHUB_WEBHOOK_SECRET`.
  * The private key will be read from `/etc/xdg/prgnome/private_key.der`,
    unless overridden by the `settings.env` file.

You can read more about socket units here: [Systemd for Developers
I](http://0pointer.de/blog/projects/socket-activation.html)

### Deploying process directly

Run `prgnome --help` to see which options are accepted. Process will run in the
foreground, so it is up to you to daemonize it and restart it on crashes.

It is recommended to run in Docker or Systemd if possible.

## Development

Install Rust via Rustup. Then run the tests or finished binary using `cargo`:
`cargo test` or `cargo run -- --help`

### Running the bot locally

Create an Github App like under the installation instructions, but for
development. Download the private key to the root of this directory and run
`./contrib/generate_private_key.sh`.

Copy `example.env` into `.env` and edit it so it contains the correct details
from your newly created example app.

You can now run `./dev-server.sh` which will watch for code changes, run tests,
and start the server. This dev server will bind to `127.0.0.1:4567`, so you can
easily run something like `ngrok http 4567` to proxy this over the internet.

Running `ngrok 4567` will give you an URL to use for your app. Edit the Github
app to point to this new hostname, and then install the App in one of your
repositories.

Webhooks triggered on this repo will now be sent to your local process and you
should be able to see the debug output that is written as it happens.

## License

Released under the MIT license. See `LICENSE` file.

Copyright (c) 2018 Magnus Bergmark
