# sweetpaste

sweetpaste is a sweet n' simple pastebin server. It's completely server-side, with *zero* client-side code.

## Configuration

The configuration will be loaded from a file named `config.toml` in the working directory.

| Config Option                       | Description                                                                                                       | Default                 |
| ----------------------------------- | ----------------------------------------------------------------------------------------------------------------- | ----------------------- |
| `address`                           | The address to bind to.                                                                                           | `127.0.0.1:8080`        |
| `site-url`                          | The base URL of the site to bind to. Should *not* contain a trailing slash!                                       | `http://127.0.0.1:8080` |
| `public`                            | Whether this instance is public or not. If this is false, the password is needed to submit pastes.                | `false`                 |
| `static-dir`                        | The directory to serve static files from. These take priority over pastes!                                        | None                    |
| `paste-limit`                       | The maximum size, in bytes, of a single paste.                                                                    | 8 MB                    |
| `cache-limit`                       | The maximum size, in bytes, of the in-memory cache, used to avoid re-rendering pastes.                            | 64 MB                   |
| `db-path`                           | The path to the SQLite database file.                                                                             | `sweetpaste.db`         |
| `password`                          | A password, used for uploading on non-public instances, and deleting *any* paste.                                 | `secret`                |
| `id-key`                            | The 32-byte encryption key used to encrypt the paste ID. sweetpaste will *refuse to start* if this is all zeroes! | `0000...`               |
| `trusted-ips`                       | A list of IP addresses which will be trusted to provide `X-Real-IP`/`X-Forwarded-For` headers                     | `["127.0.0.1", "::1"]`  |
| `syntax-highlighting.theme`         | The theme to use for syntax highlighting                                                                          | `base16-eighties.dark`  |
| `syntax-highlighting.themes-folder` | The folder to load `.tmTheme` files from                                                                          | None                    |
| `syntax-highlighting.syntax-folder` | The folder to load `.tmLanguage` files from                                                                       | None                    |


All code is licensed under the [MPLv2 License](LICENSE.md).
