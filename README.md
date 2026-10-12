# Rawchunks blockchain

Song registration, remix licenses and license-payment settlement on EVM and Solana.

Each implementation has three parts: **Factory** records songs and selling terms, **Remix** handles license purchases and licensed variations, and **Royalties** settles the license payment between the creator and treasury.

| Implementation | Stack | Song representation |
| --- | --- | --- |
| [contracts](contracts/README.md) | Solidity, OpenZeppelin, Foundry | ERC-721 song tokens. |
| [programs](programs/README.md) | Rust, Pinocchio, LiteSVM, Mollusk | Program-owned song accounts. |

[Web3 documentation](https://docs.rawchunks.space/web3/overview) · [Publishing integration](https://docs.rawchunks.space/dev/guides/publish/mint)

Contract addresses and program IDs vary by network. Check [deployed contracts](https://docs.rawchunks.space/web3/contracts/deployed-contracts) and [deployed programs](https://docs.rawchunks.space/web3/programs/deployed-programs) for current availability before connecting a wallet.
