# Solana programs

Rawchunks song registration, remix rights and license settlement on Solana. Songs are stored in program-owned accounts.

- **Factory** registers originals and records song terms and license rights.
- **Remix** purchases remix rights and registers licensed variations.
- **Royalties** settles license payments with the configured token mint and creator/treasury split.

Written in Rust with Pinocchio. LiteSVM tests transaction flows; Mollusk checks individual instructions. Shared account types are in `crates/common/`.

## Build and test

Install the [Solana toolchain](https://solana.com/docs/intro/installation). Run from this directory:

```sh
cargo build-sbf --manifest-path programs/factory/Cargo.toml
cargo build-sbf --manifest-path programs/royalties/Cargo.toml
cargo build-sbf --manifest-path programs/remix/Cargo.toml
cargo test --workspace --locked
```

Build all three programs before host tests; tests read their binaries from `target/deploy/`. The locked host-test dependencies require Rust 1.97.1 or newer. SBF builds use the Solana toolchain's compiler.

[Instructions and accounts](https://docs.rawchunks.space/web3/programs/overview) · [Client setup](https://docs.rawchunks.space/web3/programs/stack) · [Deployed programs](https://docs.rawchunks.space/web3/programs/deployed-programs)
