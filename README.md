# prgnome

> It's a "PR Gnome", not a "program nome"!

This is a small bot that you can install in your Github organization to prevent
you from pressing the nice green "Merge" button before the PR is truly ready.

If a PR is tagged with some variant of "Don't merge", "Work in progress",
"Blocked", or contains `fixup!`/`squash!`/"tmp" commits then this app will
produce a "Failed" check on your PR.
This makes the "Merge" button gray again until you rebase or remove the tags.

## How to install

1. Create a new Github App.
   * Name it something you like and input URLs for where you intend to deploy this.
2. Download the private key and store the webhook secret somewhere.
3. Convert the private key from PEM format into DER format.
  * Use `generate_private_key.sh` from this repo, or manually run the commands
    inside the script.
4. Deploy. See below.
5. Update the app to the URLs used by the shipped app, if you guessed wrong.
6. Install the app in your organization.

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
```

Then place some web server in front to proxy to the `$BIND` address and
terminate SSL.

### Deploying process directly

Run `prgnome --help` to see which options are accepted.

Systemd units will be written for this program later, which you could use.

## License

Released under the MIT license. See `LICENSE` file.

Copyright (c) 2018
