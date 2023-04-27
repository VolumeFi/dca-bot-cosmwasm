# DCA-Bot CosmWasm smart contract on Paloma

This is a CosmWasm smart contract to send messages to a DCA-bot smart contract on EVM chain written in Vyper.

Users can deposit their token or coin into a Vyper smart contract on EVM chain.

There is a view function in the smart contract that returns swappable id, minimal out amount, and remaining trade counts that can be swapped on Uniswap V2 or a DEX that works just like it.

A scheduler or script fetch the swappable id from the Vyper smart contract and run `swap` function with the id and minimal out amount.

And then, the Vyper smart contract will swap the assets and sent them to the depositors.

## ExecuteMsg

### Swap

Run `swap` function on Vyper smart contract.

| Key            | Type    | Description                               |
|----------------|---------|-------------------------------------------|
| swap_id        | Uint256 | Swap id to swap on a Vyper smart contract |
| amount_out_min | Uint256 | Minimal amount to prevent front running   |
| number_trades  | Uint256 | Remaining count of the trade              |

## QueryMsg

### GetJobId

Get `job_id` of Paloma message to run `multiple_withdraw` function on a Vyper smart contract.

| Key | Type | Description |
|-----|------|-------------|
| -   | -    | -           |

#### Response

| Key    | Type   | Description      |
|--------|--------|------------------|
| job_id | String | Job Id on Paloma |
