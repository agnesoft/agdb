# agdb baseline benchmarks

Compares the agdb in the working tree (`current`) with the agdb of a git ref (`base`) on the
same embedded workload, for each storage kind (memory, mapped, file), using `agbench`. Both
versions run in one process and in turn within every sample, so the reported overhead of
`current` is a ratio that holds on a busy machine too.

```sh
cargo run -r -p agdb_benchmark -- baseline                         # against HEAD
cargo run -r -p agdb_benchmark -- baseline origin/main             # against another ref
cargo run -r -p agdb_benchmark -- baseline -- --filter select      # agbench options after --
```

`baseline` (`src/baseline.rs`) exports the ref and generates the bench manifest in
`<tmp>/agdb_baseline`, then builds it into the repository's `target`. The export uses its own
git index, so the repository's index and working tree are left alone. The copy has to be
outside the repository: Cargo would resolve the workspace fields of a copy nested in it from
the repository's own manifest, building the base against the current `agdb_derive` and
`agplatform`. The base gets the version suffix `-base`, as a lockfile cannot hold two path
packages of the same name and version. The bench (`benches/`) is not a target of
`agdb_benchmark`, so it is built only by `baseline`.

The workload (`benches/workload.rs`) is compiled once against each version and so uses only the
API both share. A change of that API is compared against a ref that has it.
