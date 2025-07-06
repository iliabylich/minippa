# `minippa`

This project provides a single-binary executable that acts as a Debian package server.

### Installation

Pick the one you like:

1. `cargo install minippa`
2. Download the latest binary directly from the latest release
3. Download a DEB package from the latest release

The DEB package contains a systemd service, which should slightly simplify your setup.

### Configuration

The app reads the config file from:

1. "$PWD/config.toml" in debug mode
2. "/etc/minippa.toml" in release mode

Here's an example of what must be inside:

```toml
port = 4242
token = "sekr3t"
dir = "/data-dir"
```

Then generate a GPG key with `minippa --generate-key`. The key that it generates is, of course, random every time but the metadata is static (`email = "owner@this-repo.org"` and `name = "Owner Name"`).

Then run the binary. It will start listening for incoming requests on "localhost:4242" (so you need a reverse proxy, something like Caddy or NGINX) and by sending packages to the `/upload` endpoint you'll import them:

```sh
$ curl -F "wget-1.2.3.deb=@wget-1.2.3.deb" -H "Token: sekr3t" "https://external-name"
Package has been successfully processed
```

Where:

1. `https://external-name` is the URL of your machine
2. `-F "foo=@bar"` means "create a chunk called foo and inside it attach a file called bar"
3. `-H "Token: ..."` is where you attach a header with the token from your config file. If the token doesn't match, the server will reject your request.

### Client

On any client machine that has access to the server via HTTP(S), run **as sudo**

```sh
$ curl https://external-name/install.sh | bash
```

1. Yes, this server is not only capable of file upload and static file serving but it is also able to generate a dynamic install script. In fact, the only dynamic part there is the hostname that is taken from a request header.
2. The script installs the public GPG key that you previously generated
3. Then it creates an entry in "/etc/apt/sources.list.d/" that points to your server.

Then run `apt update` and you should see how `apt` hits your server. Now you can `apt install wget` (or any of the packages that you uploaded).
