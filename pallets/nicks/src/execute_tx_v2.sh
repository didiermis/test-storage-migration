#!/bin/bash

polkadot-js-api tx.nicks.setName "DanteXRP1" "MaximunXRP1" --seed "//Alice"

polkadot-js-api tx.nicks.setName "AlexaXRP1" "SolarisRP1" --seed "//Alice//stash"

polkadot-js-api tx.nicks.setName "MarianaXRP1" "SolanaXRP1" --seed "//Bob"

polkadot-js-api tx.nicks.setName "CarlosXRP1" "CalciumXRP1" --seed "//Bob//stash"
