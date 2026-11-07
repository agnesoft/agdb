# description

The `agci` utility helps automate the release process. It determines the current version from the latest `v*` git tag and reads the new version from the root `Cargo.toml` (`[workspace.package].version`). It then updates version strings in workspace packages (e.g. TypeScript `package.json` files) and regenerates OpenAPI/client artifacts and test queries. For TypeScript projects it also runs `pnpm install`, which updates `pnpm-lock.yaml`.

- agdb
- agdb_derive
- agdb_api
- @agnesoft/agdb_api

# usage

NOTE: Requires Rust toolchain and `pnpm` to be installed. It is recommended to use [`rustup`](https://www.rust-lang.org/tools/install) to get Rust and [NodeJS](https://nodejs.org/en) to get `npm` with which you can install the pnpm: `npm i -g pnpm`.

```
cargo run --release -p agci
```

# release

In order to release a new version of the packages in the repository:

1. Create an issue for the new release, e.g. `[ci] Release 1.2.3`.

2. Create a branch for the issue, e.g. `999-ci-release-1-2-3`.

3. Update the version in the root `Cargo.toml` (the `[workspace.package].version` value).

4. Run `agci` (NOTE: the pull request pipeline contains a validation that will fail if `Cargo.toml` changed but generated packages/artifacts do not match it, i.e. `agci` was not run.)

```
cargo run --release -p agci
```

5. Commit the result.

6. Open a pull request from the release branch to `main`.

7. After merge the release will happen automatically.
