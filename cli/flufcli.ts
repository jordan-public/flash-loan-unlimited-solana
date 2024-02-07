#!/usr/bin/env ts-node
// SPDX-License-Identifier: BUSL-1.1
import { AccountLayout } from '@solana/spl-token';
import {Connection, Keypair, PublicKey, SystemProgram, Transaction, sendAndConfirmTransaction} from '@solana/web3.js';
import yargs from 'yargs';
import { hideBin } from 'yargs/helpers';
import os from 'os';
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { expect } from 'chai';
import { createMint, getOrCreateAssociatedTokenAccount, mintTo, transfer, createSetAuthorityInstruction, createTransferInstruction, AuthorityType, getAccount, getMint, TOKEN_PROGRAM_ID, MintLayout } from "@solana/spl-token";
import { Fluf } from "../target/types/fluf";
import { BorrowerSample } from "../target/types/borrower_sample";
import BN from 'bn.js';
import * as fs from 'fs';

const run = async() => {

    const PROGRAM_ID = new PublicKey("D9KddH1TpYtNCcZ5igZLrXbreJVHGEmMqfzbkhST9MwS");
    const PROGRAM_BORROWER_ID = new PublicKey("BiBiaMTWecRB3cbz6oMfrKq3F1VKCRLCxKgS3NYLTMCK");

    // Parse the command
    const argv = yargs(hideBin(process.argv))
        .option('url', {
            alias: 'u',
            description: 'RPC URL of the cluster',
            type: 'string',
            default: 'https://api.devnet.solana.com',
        })
        .option('payer', {
            alias: ['p', 'wallet', 'w'],
            description: 'Payer account',
            type: 'string',
            default: os.homedir()+'/.config/solana/id.json'
        })
        .help()
        .alias('help', 'h')
        .parseSync(); // Ensure to use parseSync() for correct typing
        //.argv;

    // console.log('argv:', argv);

    // Convert shorthands for the RPC URL to the full URL
    if (argv.url === 'mainnet-beta' || argv.url === 'm') {
        argv.url = 'https://api.mainnet-beta.solana.com';
    } else if (argv.url === 'testnet' || argv.url === 't') {
        argv.url = 'https://api.testnet.solana.com';
    } else if (argv.url === 'devnet' || argv.url === 'd') {
        argv.url = 'https://api.devnet.solana.com';
    } else if (argv.url === 'localnet' || argv.url === 'l') {
        argv.url = 'http://localhost:8899';
    }

    // The first argument is the subcommand with possible choices of: create, deposit, withdraw, balance, run, feesbalance, feeswithdraw, initialize
    const subcommand = argv._[0];
    switch (subcommand) {
        case 'create':
            // Create a new pool for the token mint in the second argument
            if (argv._.length < 2) {
                yargs.showHelp();
                process.exit(1);
            }
            // Read the wallet file
            let payer;
            try {
                payer = anchor.web3.Keypair.fromSecretKey(new Uint8Array(require(argv.payer)));
                //payer = Keypair.fromSecretKey(new Uint8Array(require(argv.payer)));
                console.log('Payer:', payer.publicKey.toBase58());
            } catch (error) {
                console.error("Invalid Solana address");
                yargs.showHelp();
                process.exit(1);
            }
            let connection;
            try {
                connection = new anchor.web3.Connection(argv.url, 'processed');
                //connection = new Connection(argv.url, 'confirmed');
            } catch (error) {
                console.error("Connection URL is invalid, or there is no network connection");
                yargs.showHelp();
                process.exit(1);
            }
            // Parse the second argument as the token mint
            let tokenMint = new PublicKey("9jEQkoG2vX3ohCr9JzJMuXjydUGxFEjR9phJQsjkHZMP");
            let tokenDecimals = 0;
            try {
                tokenMint = new PublicKey(argv._[1]);
                
                const mintInfo = await getMint(connection, tokenMint);
                tokenDecimals = mintInfo.decimals;
            } catch (error) {
                console.error("Invalid Solana token mint address");
                yargs.showHelp();
                process.exit(1);
            }
            // Create the pool
            const provider = new anchor.AnchorProvider(connection, new anchor.Wallet(payer), { commitment: 'processed', preflightCommitment: 'processed' });
            anchor.setProvider(provider);

            const program = anchor.workspace.Fluf as Program<Fluf>;
            
            // Prepare the accounts for the create_pool function
            const user = payer;
            const pool = (await PublicKey.findProgramAddress(
            [Buffer.from("pool"), tokenMint.toBuffer()], 
            PROGRAM_ID
            ))[0];
            const pool_mint = tokenMint;
            const pool_account =(await PublicKey.findProgramAddress(
            [Buffer.from("pool_account"), tokenMint.toBuffer()], 
            PROGRAM_ID
            ))[0];
            const fluf_mint = (await PublicKey.findProgramAddress(
            [Buffer.from("fluf_mint"), tokenMint.toBuffer()], 
            PROGRAM_ID
            ))[0];
            const pool_fluf_account =(await PublicKey.findProgramAddress(
            [Buffer.from("pool_fluf_account"), tokenMint.toBuffer()], 
            PROGRAM_ID
            ))[0];
            const rent = anchor.web3.SYSVAR_RENT_PUBKEY;
            const system_program = SystemProgram.programId;
            const token_program = TOKEN_PROGRAM_ID;

            console.log("tokenMint", tokenMint.toBase58());
            console.log("tokenDecimals", tokenDecimals);
            // Call the create_pool function of the program
            const createPoolTx = await program.methods.createPool(tokenDecimals).accounts({
                user: user.publicKey.toBase58(),
                pool: pool,
                poolMint: pool_mint,
                poolAccount: pool_account,
                flufMint: fluf_mint,
                poolFlufAccount: pool_fluf_account,
                rent: rent,
                systemProgram: system_program,
                tokenProgram: token_program,
            }).signers([user]).rpc();
            console.log("create_pool transaction signature", createPoolTx);
            console.log("Pool created. Fluf token: ", fluf_mint.toBase58());
            
            break;
        case 'deposit':
            console.log('deposit');
            break;
        case 'withdraw':
            console.log('withdraw');
            break;
        case 'balance':
            console.log('balance');
            break;
        case 'run':
            console.log('run');
            break;
        case 'feesbalance':
            console.log('feesbalance');
            break;
        case 'feeswithdraw':
            console.log('feeswithdraw');
            break;
        case 'initialize':
            console.log('initialize');
            break;
        default:
            console.log('Unknown subcommand');
    }

    // Write the arguments to the console
    // console.log('url:', argv);

    // async function main() {
    //     const connection = new Connection("https://api.devnet.solana.com", "confirmed");
    //     const payer = Keypair.generate();

    //     // Sample transaction: create a new account
    //     const newAccount = Keypair.generate();
    //     const transaction = new Transaction().add(
    //         SystemProgram.createAccount({
    //             fromPubkey: payer.publicKey,
    //             newAccountPubkey: newAccount.publicKey,
    //             lamports: await connection.getMinimumBalanceForRentExemption(0),
    //             space: 0,
    //             programId: SystemProgram.programId,
    //         })
    //     );

    //     await sendAndConfirmTransaction(connection, transaction, [payer, newAccount]);
    //     console.log(`New account created: ${newAccount.publicKey.toBase58()}`);
    // }

    // main().catch(err => console.error(err));

};

run();