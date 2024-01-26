# Troubleshooting

1. Getting cross-origin (CORS) errors when connecting to the `agdb_server`.

CORS error can happen even locally when running the server and connecting to it using raw IP address (e.g. `127.0.0.1`). Try running the server and binding it to `localhost` (default) or another DNS name instead.
