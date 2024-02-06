# FLUF Protocol Demo

## Video

See demo video [here](./demo/README.md) and on [YouTube here]().

## Cheat Sheet

If deployed locally, solana-test-validator must be run in advance and left running in a separate terminal.

---

Get some SOL:
```
solana --url http://localhost:8899 airdrop 1000
```

---

Create a Solana token mint
```
spl-token --url http://localhost:8899 create-token
```
Remember the displayed T token mint account.

---

Mint some tokens (100)
```
spl-token --url http://localhost:8899 A87Qq3WzVUEqoKrRFobhaPEGnFBgGcwVH8ZCTL5CDqGY 100000000000
```

---

Deploy fluf program and borrower sample program:
```
solana program deploy ./target/deploy/fluf.so --url http://localhost:8899
```
```
solana program deploy ./target/deploy/borrower_sample.so --url http://localhost:8899
```

---

Create a new fluf pool:
```
./cli/flufcli.ts -u http://localhost:8899 create A87Qq3WzVUEqoKrRFobhaPEGnFBgGcwVH8ZCTL5CDqGY
```
but replace the T token mint with the one displayed above when created.

---

Execute the flash loan and the borrower action:
```
./cli/flufcli.ts -u http://localhost:8899 run A87Qq3WzVUEqoKrRFobhaPEGnFBgGcwVH8ZCTL5CDqGY 300000000000 10000000000 <borrower_program>
```
Observe the message in the solant-test-validator log that displays the **amount of borrowed fT (300)**,
which **exceeds the total fT in circulation (100)**.

---