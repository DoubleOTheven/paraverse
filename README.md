# Paraverse

A blockchain to power Metaverse and gaming economies.

## Features
* Create custom Assets (a.k.a. Fungible "Tokens")
* Create AMM pools between arbitrary fungible token pairs
* Create NFTs
* The Pricing API can be derived from the chain state:
  * The AMM swap price is determined by the ratio of token A to token B. This can be done on the client
  * Fetch the real-world USD price from the chain that was set via an authorized pricing oracle
* As a `root` caller you can whitelist Pricing Oracles
* As a `pricing_oracle` you can set real-world values on each token. 
  * NOTE: in a production app I would create a multi-party pricing pool for each Asset ensure integrity. Outlandish prices can be culled and Pricing Oracles slashed.

## Architecture

* [AMM Dex](https://github.com/DoubleOTheven/paraverse/tree/master/pallets/template)

* [TESTS and a cool AMM calculator with integer-decimal math](https://github.com/DoubleOTheven/paraverse/blob/master/pallets/template/src/dex_pricer.rs)

* [NFT Maker](https://github.com/DoubleOTheven/paraverse/tree/master/pallets/nft_maker)
  * Allows you to create a NFT

* [NFT Marketplace](https://github.com/DoubleOTheven/paraverse/tree/master/pallets/nft_marketplace)
  * Allows you to create a SaleItem using any Asset, including LP Assets :)

* [Custom Traits](https://github.com/DoubleOTheven/paraverse/blob/master/pallets/custom_traits/src/lib.rs)
  * Used to keep business logic isolated per pallet. I would use this more if I had more time for reusable code and isolation of unit testing

* [Chain Spec](https://github.com/DoubleOTheven/paraverse/blob/master/node/src/chain_spec.rs)
  * Here I create assets for three pools. Three Assets for trading, and three LP Assets for the three pools.
    * e.g. Token AB Pool with LP_AB Assets for liquidity providers, Token BC Pool with LP_BC for liquidity providers, and a LP_AB / LP_BC pool with LLP tokens for liquidity providers.

## What I would change with more time
* More Tests!!! I unit tested the scary math in dex_pricer, but I would add more tests for state transition functions
* Change the Pool ID in pallet-node-template to use a hash of the Asset Pair IDs to ensure uniqueness. It is not a big deal ATM bc/ pools are created via `root` access.