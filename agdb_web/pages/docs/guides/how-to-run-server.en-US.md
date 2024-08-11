---
title: "How to run the server?"
description: "How to run the server, Agnesoft Graph Database"
---

# How to run the server?

The following is a guide how to run a local instance of the `agdb_server` on any platform/OS supported by Rust building from source.

<br/>1. Install git from the [officail source](https://git-scm.com/) (skip if you already have it).
<br/>

<br/>2. Install Rust toolchain from the [official source](https://www.rust-lang.org/tools/install) (mininum required version is `1.75.0`).
<br/>

<br/>3. Clone the `agdb` repository: `git clone https://github.com/agnesoft/agdb.git` (or `git@github.com:agnesoft/agdb.git` if using SSH).
<br/><br/>

<br/>4. Enter the directory with `cd agdb` and build the server:
<br/><br/>

```bash
cargo build --release -p agdb_server
```

<br/>5. The server binary will be within the repository `target/release/agdb_server[.exe]`. You can either move/copy the binary to your desired location on `PATH` (e.g. `/usr/bin/`) or you can run it directly with cargo:
<br/><br/>

When on `PATH`:

```bash
agdb_server
```

Via `cargo`:

```bash
cargo run --release -p agdb_server
```

The server upon starting will create few things in its working directory:

-   `agdb_server.yaml`: Configuration file. You can alter it as you wish. You would need to restart the server for the changes to take effect.
-   `agdb_server.agdb` (`.agdb_server.agdb`): Internal databse of the server (uses `agdb` itself) + its write ahead file (the dotfile).
-   `agdb_data_dir/`: Folder for stroing the user data. It can be changed in the configuration file (requires restart of the server).

and report where it listens at:

```
2024-01-26T17:47:30.956260Z  INFO agdb_server: Listening at localhost:3000
```

NOTE: You can prepare the configuration file before starting the server. It supports following values:

```yaml
# agdb_server.yaml
host: localhost # host address to listen on
port: 3000 # port to bind to
admin: admin # the admin user that will be created automatically for the server, the password will be the same as name (admin by default, recommended to change after startup)
data_dir: agdb_server_data # directory to store user data
```

<br/>6. The server will be available on `host:port` as per configuration (i.e. `localhost:3000` by default). The server logs every request-response as a single entry each time to `STDOUT`. You can redirect the output to a file, e.g. `agdb_server > server.log`. It is recommended to **change the admin password from the default** (same as admin username by default).
<br/><br/>

<br/>7. You can test if the server is up with `curl`:
<br/><br/>

```bash
curl -v localhost:3000/api/v1/status # should return 200 OK
```

<br/>8. It is recommended by optional to create a user to use for the database management rather than using the `admin` user (which is however still possible):
<br/><br/>

```bash
 # produce an admin API token, e.g. "bb2fc207-90d1-45dd-8110-3247c4753cd5"
token=$(curl -X POST -H 'Content-Type: application/json' localhost:3000/api/v1/user/login -d '{"username":"admin","password":"admin"}')
# using admin token to create a user
curl -X POST -H "Authorization: Bearer ${token}" localhost:3000/api/v1/admin/user/my_db_user/add -d '{"password":"password123"}'
# login as the new user and producing their token
token=$(curl -X POST -H 'Content-Type: application/json' localhost:3000/api/v1/user/login -d '{"username":"my_db_user","password":"password123"}')
```

<br/>9. To interact with the database you can either continue using `curl`, interactive OpenAPI GUI from any browser `localhost:3000/api/v1` (provided by `rapidoc`) or choose one of the [available API clients](/api-docs/openapi). The raw OpenAPI specification can be downloaded from the server at `localhost:3000/api/v1/openapi.json`.
<br/><br/>

<br/>10. The server can be shutdown with `CTRL+C` or programmatically posting to the shutdown endpoint as logged in server admin:
<br/><br/>

```bash
# this will produce an admin API token, e.g. "bb2fc207-90d1-45dd-8110-3247c4753cd5"
token=$(curl -X POST -H 'Content-Type: application/json' localhost:3000/api/v1/user/login -d '{"username":"admin","password":"admin"}')
curl -X POST -H "Authorization: Bearer ${token}" localhost:3000/api/v1/admin/shutdown
```
