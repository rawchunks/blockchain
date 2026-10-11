# EVM contracts

Solidity interfaces for Rawchunks song tokens, remix rights and license payments.

- **Factory** mints originals, stores song terms and metadata references, and supports creator updates and ERC-721 ownership.
- **Remix** sells remix licenses and mints licensed variations linked to the original.
- **Royalties** settles license payments with the configured token and creator/treasury split.

Token ownership, the recorded creator and purchased remix rights are separate. Use the [contract guides](https://docs.rawchunks.space/web3/contracts/overview) for exact calls and events.

## Build and test

Solidity 0.8.28, OpenZeppelin Contracts and Foundry. Install [Foundry](https://getfoundry.sh/introduction/installation), then run from this directory:

```sh
forge install foundry-rs/forge-std@v1.16.2
forge install OpenZeppelin/openzeppelin-contracts@v5.7.0
forge build --contracts src
forge test --contracts src
```

The source is in `src/`, so keep `--contracts src`. Public interfaces are in `src/interfaces/`; Solidity tests are in `test/`.

[Client setup](https://docs.rawchunks.space/web3/contracts/stack) · [Deployed contracts](https://docs.rawchunks.space/web3/contracts/deployed-contracts)
