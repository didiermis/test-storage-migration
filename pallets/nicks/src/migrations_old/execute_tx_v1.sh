#!/bin/bash

polkadot-js-api tx.nicks.setName "DanteXRP1" --seed "//Alice"

polkadot-js-api tx.nicks.setName "AlexaXRP1" --seed "//Alice//stash"

polkadot-js-api tx.nicks.setName "MarianaXRP1" --seed "//Bob"

polkadot-js-api tx.nicks.setName "CarlosXRP1" --seed "//Bob//stash"
