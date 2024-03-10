# Vitalicals-cli
a client cli for vitalicals

## 1. Create a btc wallet

```bash
./target/release/vitalicals-cli --indexer http://localhost:9944  -n  regtest -e 10.1.1.84:50002 wallet create
mnemonic: garbage notice other combine frost tornado nominee mechanic jacket match hammer virtual
recv desc (pub key): "tr([0670c99a/86'/1'/0']tpubDDZps2fBuMesuiuXc6GfBXzFWXrFkPV8uAQ7zruqviUUtqsZrRgNY8nHM4pwUh2N7ycLniV1ny5fetHWvgzuUJjVj6pQahXVumyNNsfKZya/0/*)#0390358l"
chng desc (pub key): "tr([0670c99a/86'/1'/0']tpubDDZps2fBuMesuiuXc6GfBXzFWXrFkPV8uAQ7zruqviUUtqsZrRgNY8nHM4pwUh2N7ycLniV1ny5fetHWvgzuUJjVj6pQahXVumyNNsfKZya/1/*)#79qwvph8"
```

it will create a new mnemonic, and storage wallet file in ./.vitalicals-cli

also can import a mnemonic, but you need make sure it is safety

```bash
 ./target/release/vitalicals-cli --indexer http://localhost:9944  -n  regtest -e 10.1.1.84:50002 wallet import 'garbage notice other combine frost tornado nominee mechanic jacket match hammer virtual'
mnemonic: garbage notice other combine frost tornado nominee mechanic jacket match hammer virtual
recv desc (pub key): "tr([0670c99a/86'/1'/0']tpubDDZps2fBuMesuiuXc6GfBXzFWXrFkPV8uAQ7zruqviUUtqsZrRgNY8nHM4pwUh2N7ycLniV1ny5fetHWvgzuUJjVj6pQahXVumyNNsfKZya/0/*)#0390358l"
chng desc (pub key): "tr([0670c99a/86'/1'/0']tpubDDZps2fBuMesuiuXc6GfBXzFWXrFkPV8uAQ7zruqviUUtqsZrRgNY8nHM4pwUh2N7ycLniV1ny5fetHWvgzuUJjVj6pQahXVumyNNsfKZya/1/*)#79qwvph8"
```

you should give this address some btc, first, we can got the address by wallet:

```bash
./target/release/vitalicals-cli --indexer http://localhost:9944  -n  regtest -e 10.1.1.84:50002 wallet address
address: bcrt1pr04tzkr2eysslnk5afvry9fp2ujy5czgeqxgpt092fyv930q79js67hj48
```

use btc cli to give this address some btc, then we can got balance:

```bash
./target/release/vitalicals-cli --indexer http://localhost:9944  -n  regtest -e 10.1.1.84:50002 wallet balance
balance: { immature: 0, trusted_pending: 0, untrusted_pending: 0, confirmed: 10000000000 }
```

> Note we support cli use different wallet by name, in default, the cli will use `default` wallet, we can use other, can see details in next.

Now we can use cli.

## 2. Mint a name for deploy VRC20

```bash
./target/release/vitalicals-cli --indexer http://localhost:9944  -n  regtest -e 10.1.1.84:50002 mint name vital
```

it mint `vital` name, can query:

```bash
./target/release/vitalicals-cli --indexer http://localhost:9944  -n  regtest -e 10.1.1.84:50002 query resources
find 1 resources
0. find pending 6dae6e6ed1ba7245fdde59c3ae1b5031eb8529de1d781ba7bbdd86998d9671b1:0 contain with resource name(vital)
```

Note the name is pending to into btc block, we need wait for btc blocks. then we can got:

```bash
./target/release/vitalicals-cli --indexer http://localhost:9944  -n  regtest -e 10.1.1.84:50002 query resources
find 1 resources
0. find 6dae6e6ed1ba7245fdde59c3ae1b5031eb8529de1d781ba7bbdd86998d9671b1:0 contain with resource name(vital)
```

We can use this name to deploy a vrc20 token:

usage:

`vitalicals-cli deploy vrc20 <NAME> <MINT_AMOUNT> <MAX_MINTS>`

```bash
 ./target/release/vitalicals-cli --indexer http://localhost:9944  -n  regtest -e 10.1.1.84:50002 deploy vrc20 vital 100000 100000000
```

wait btc block, we can got vrc20 metadata:

```bash
./target/release/vitalicals-cli --indexer http://localhost:9944  -n  regtest -e 10.1.1.84:50002 query vrc20-metadata vital
metadata: {
  "mint_count": 0,
  "meta": {
    "decimals": 5,
    "nonce": 0,
    "bworkc": 0,
    "mint": {
      "mint_amount": 100000,
      "mint_height": 0,
      "max_mints": 100000000
    },
    "meta": null
  }
}
```

## 3. Mint vrc20

```bash
./target/release/vitalicals-cli --indexer http://localhost:9944  -n  regtest -e 10.1.1.84:50002 mint vrc20 vital
```

then we can got pending vrc20:

```bash
./target/release/vitalicals-cli --indexer http://localhost:9944  -n  regtest -e 10.1.1.84:50002 query resources           
find 1 resources
0. find 0f803eac974dd779666d867352d654dae4dac2ebbfe3d7bb4b2ff589b0aec916:0 contain with resource vrc20([vital,100000])
```

we can mint many times in a block:

```bash
./target/release/vitalicals-cli --indexer http://localhost:9944  -n  regtest -e 10.1.1.84:50002 query resources 
find 3 resources
0. find 0f803eac974dd779666d867352d654dae4dac2ebbfe3d7bb4b2ff589b0aec916:0 contain with resource vrc20([vital,100000])
1. find pending bb68a54e4c3a82f7332432705253530e0f4386ee2f16cbeaad01dc896763e8ad:0 contain with resource vrc20([vital,100000])
2. find pending 6dd8ba49afbc90e995ec90fd1dfe2f9d90d8e3d7742014693032347835a7bbe0:0 contain with resource vrc20([vital,100000])
```

after blocks we can got all vrc20 is not pending:

```bash
./target/release/vitalicals-cli --indexer http://localhost:9944  -n  regtest -e 10.1.1.84:50002 query resources
find 3 resources
0. find 0f803eac974dd779666d867352d654dae4dac2ebbfe3d7bb4b2ff589b0aec916:0 contain with resource vrc20([vital,100000])
1. find bb68a54e4c3a82f7332432705253530e0f4386ee2f16cbeaad01dc896763e8ad:0 contain with resource vrc20([vital,100000])
2. find 6dd8ba49afbc90e995ec90fd1dfe2f9d90d8e3d7742014693032347835a7bbe0:0 contain with resource vrc20([vital,100000])
```

## 4. Transfer vrc20

before:

```bash
./target/release/vitalicals-cli --indexer http://localhost:9944  -n  regtest -e 10.1.1.84:50002 query resources
find 3 resources
0. find 0f803eac974dd779666d867352d654dae4dac2ebbfe3d7bb4b2ff589b0aec916:0 contain with resource vrc20([vital,100000])
1. find bb68a54e4c3a82f7332432705253530e0f4386ee2f16cbeaad01dc896763e8ad:0 contain with resource vrc20([vital,100000])
2. find 6dd8ba49afbc90e995ec90fd1dfe2f9d90d8e3d7742014693032347835a7bbe0:0 contain with resource vrc20([vital,100000])
```

move to self:

```bash
./target/release/vitalicals-cli --indexer http://localhost:9944  -n  regtest -e 10.1.1.84:50002 move vrc20 vital 2000
```

we can got:

```bash
./target/release/vitalicals-cli --indexer http://localhost:9944  -n  regtest -e 10.1.1.84:50002 query resources      
find 4 resources
0. find bb68a54e4c3a82f7332432705253530e0f4386ee2f16cbeaad01dc896763e8ad:0 contain with resource vrc20([vital,100000])
1. find 6dd8ba49afbc90e995ec90fd1dfe2f9d90d8e3d7742014693032347835a7bbe0:0 contain with resource vrc20([vital,100000])
2. find pending 769e9901de9f6ab121dfca17689f1a4ab098f0049e061e0b30fa1141157a6648:0 contain with resource vrc20([vital,2000])
3. find pending 769e9901de9f6ab121dfca17689f1a4ab098f0049e061e0b30fa1141157a6648:1 contain with resource vrc20([vital,98000])
```

move to other address :

```bash
./target/release/vitalicals-cli --indexer http://localhost:9944  -n  regtest -e 10.1.1.84:50002 --to bcrt1py4zy879dj5d36xzjsl4yuvzgxss8u3ha7wkxvlkctp50xqppykhs7k0ezw move vrc20 vital 210000
```

then we can got:

```bash
./target/release/vitalicals-cli --indexer http://localhost:9944  -n  regtest -e 10.1.1.84:50002 query resources                                                                              
find 1 resources
0. find 482d8d933842d70eeebf342e20fa165ddec930342f6e9be10ddb5cbbe15339ab:1 contain with resource vrc20([vital,90000])
```

## 5. use different wallet

We can create different wallet by name `test_wallet`:

```bash
./target/release/vitalicals-cli --indexer http://localhost:9944  -n  regtest -e 10.1.1.84:50002 wallet create test_wallet
```

So we can create a wallet named "test". We can use other cmds like:

```bash
./target/release/vitalicals-cli --indexer http://localhost:9944  -n  regtest -e 10.1.1.84:50002 wallet address --wallet test_wallet
address: bcrt1p5hs3s9kpa3ncmwy95jkeduqntmlhyedk9huk5965gj83lggru85stsv4sd

./target/release/vitalicals-cli --indexer http://localhost:9944  -n  regtest -e 10.1.1.84:50002 wallet balance test_wallet
balance: { immature: 0, trusted_pending: 0, untrusted_pending: 0, confirmed: 100000000 }
```

We can mint by:

```bash
./target/release/vitalicals-cli --indexer http://localhost:9944  -n regtest -e 10.1.1.84:50002 --wallet test_wallet mint vrc20 vital
```

We also can search all wallet by:

```bash
./target/release/vitalicals-cli -n regtest wallet list
```
