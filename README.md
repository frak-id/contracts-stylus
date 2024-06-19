# Frak - Stylus Contracts

[![Project license](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://github.com/frak-id/contracts-stylus/LICENSE.txt)

## Bringing Web2 Anti-Cheat Mechanisms to Web3 with Arbitrum Stylus

This repository houses a Rust implementation of a Content Consumption Tracker contract, specifically designed for the [Arbitrum Stylus](https://arbitrum.io/stylus). 

This contract leverages the performance benefits of Stylus (using **WASM**) to port a Web2 anti-cheat mechanism to a blockchain environment, enhancing the security and trust of content consumption data.

**Context**

Frak is building a decentralized ecosystem for content interactions, with a strong focus on accurate and reliable consumption tracking. 

You can find the Solidity smart contracts for the broader Frak ecosystem [here](https://github.com/frak-id/contracts-v2).

This Stylus contract builds upon Frak's existing anti-cheat system, bringing key parts of it on-chain to further improve security and transparency. 

## Key Features

:sparkles: Proof-of-concept (POC) using ECDSA signatures for data verification.
:sparkles: Integrates with the Frak Content Registry for content verification. 

## Roadmap

- [x] ⚗️ POC with ECDSA signatures.
- [x] 🗃️️ Link with the **ContentRegistry** from [contracts-v2](https://github.com/frak-id/contracts-v2).
- [x] 🔨 Deployment and simple test scripts.
- [ ] 🧪 Unit tests.
- [ ] 🔒️ MerkleProof validation for channel IDs.
- [ ] 🗃️ **LZ compression** of calldata using Rust.
- [ ] 🔒️ **ZKP** for verifying consumption without content reveal.
- [ ] 🔒️ **Grooth16** sig for batched CCU submissions.

## Technologies Used

- [OpenZeppelin/rust-contracts-stylus](https://github.com/OpenZeppelin/rust-contracts-stylus): For core smart contract logic.
- [alloy-rs/alloy](https://github.com/alloy-rs/alloy): For all scripting and tooling.

Script inspired by [Renegade-fi](https://github.com/renegade-fi/renegade-contracts/tree/main/scripts).

## Contributing

Contributions are welcome! Let's build a more secure and transparent content consumption tracking system. Open an issue or submit a pull request. 

## License

GPL-V3 