# Concordium Hackathon Workshop Material

This repository contains the code used in Concordium's workshops at hackathons
in 2023. The workshop builds a voting application and showcases the identity feature of the Concordium blockchain.

It has two parts, a smart contract and a dApp.

Note: There is also a more "advanced" version of the system, which exists in the `/advanced` folder. You can read more about it [below](#advanced-version)

## Prerequisites

- [The Rust programming language](https://www.rust-lang.org/tools/install) version 1.53+
- [NodeJS and NPM](https://nodejs.org/en/)
- [Cargo
  Concordium](https://developer.concordium.software/en/mainnet/net/installation/downloads-testnet.html#cargo-concordium-v2-5-0)
  version 2.5.0
- [Concordium
  Client](https://developer.concordium.software/en/mainnet/net/installation/downloads-testnet.html#concordium-client-v5-0-2)
  version 5.0.2

## Smart contract instructions

1. Open the `smart-contract/` folder in a terminal.
2. Build the smart contract with [Cargo Concordium](https://developer.concordium.software/en/mainnet/net/installation/downloads-testnet.html#cargo-concordium-v2-5-0):
   - For example:
   ```
   cargo concordium build --schema-embed --schema-out schema.bin --out voting.wasm.v1
   ```
3. You can also run the tests with Cargo concordium: `$ cargo concordium test`.
4. Deploy the smart contract to the chain with [Concordium Client](https://developer.concordium.software/en/mainnet/net/installation/downloads-testnet.html#concordium-client-v5-0-2):
   - ```
     concordium-client --grpc-ip node.testnet.concordium.com module deploy voting.wasm.v1 --sender ACCOUNT_ADDRESS --name voting-contract-module
     ```
   - Note that concordium-cleint is communicating with the node located at `node.testnet.concordium.com`.
   - Also note that we are adding a local name to the module: `voting-contract-module`.
5. Initialize the smart contract from the module:
   - ```
     concordium-client --grpc-ip node.testnet.concordium.com contract init voting-contract-module --contract voting --parameter-json init-parameter.json --sender ACCOUNT_ADDRESS --energy 10000 --name voting-contract
     ```
   - Note that we are using the json file `init-parameter.json` as input.
   - Also note that we are adding a local name to the instance: `voting-contract`.
6. Update the contract (vote):
   - ```
     concordium-client --grpc-ip node.testnet.concordium.com contract update voting-contract --entrypoint vote --parameter-json vote-parameter.json --sender ACCOUNT_ADDRESS --energy 10000
     ```
   - Note that we are calling the `vote` entrypoint (method) on the smart
     contract instance.
   - Also note that we are using the json file `vote-parameter.json` as input.
7. View the voting results:
   - ```
     concordium-client --grpc-ip node.testnet.concordium.com contract invoke voting-contract --entrypoint view
     ```
   - Note that this is _not_ a transaction, so we do not need to provide an
     energy limit or a sender account to pay.

## More information and support

For more information, tutorials, guides etc. see our [developer documentation](https://developer.concordium.software/).
If you get stuck, reach out to us on:

- The Concordium [software support channels](https://support.concordium.software).

## dApp instructions

1. Open the `dapp/` folder in a terminal.
2. Run `npm install` to install the dependencies.
3. Run the app with `npm start`.

For using the dapp, you need the ![Concordium Wallet browser extension](https://chrome.google.com/webstore/detail/concordium-wallet/mnnkpffndmickbiakofclnpoiajlegmg).

### Using `yarn` (on unix systems)

Some of the node modules we use have Windows-type line endings (`\r\n`), instead
of unix line endings (`\n`), which causes problems when using the `yarn` package
manager.

If you see an error message similar to this, then you've run into the problem:

``` sh
env: node\r: No such file or directory
```

The issue does not occur with `npm` because it automatically fixes the line
endings on unix systems.
However, it is possible to use `yarn`, but you need to fix the line endings
before it will work.
This guide explains how to fix the line endings on macOS: https://techtalkbook.com/env-noder-no-such-file-or-directory/

## Advanced version

The advanced version living in `/advanced` includes modified versions of the dApp and smart contract along with a "verifier backend", which the dApp sends the proof to for verification.
The backend returns a signature if the verification succeeds, and this signature must then be included when calling the `vote` entrypoint on the smart contract, 
which checks that the signature is valid.
These extra steps ensure that the smart contract itself can check that the voter account is eligible for voting, i.e. that the account does not live in the country being voted for.
In the simple version, the check only occurs in the frontend, and so, you can circumvent it by calling the contract directly with e.g. concordium-client.
