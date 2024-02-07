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

    // Check if the subcommand (second argument) is missing
    if (argv._.length < 2) {
        yargs.showHelp();
        process.exit(1);
    }
    // The first argument is the subcommand with possible choices of: create, deposit, withdraw, balance, run, feesbalance, feeswithdraw, initialize
    const subcommand = argv._[0];

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
    const placeholder = new PublicKey("9jEQkoG2vX3ohCr9JzJMuXjydUGxFEjR9phJQsjkHZMP");
    let tokenMint = placeholder;
    let tokenDecimals = 0;
    let pool = placeholder;
    let pool_account = placeholder;
    let fluf_mint = placeholder;
    let pool_fluf_account = placeholder;
    let fee_account = placeholder;
    if (subcommand !== 'initialize') {
        try {
            tokenMint = new PublicKey(argv._[1]);
            
            const mintInfo = await getMint(connection, tokenMint);
            tokenDecimals = mintInfo.decimals;
        } catch (error) {
            console.error("Invalid Solana token mint address");
            yargs.showHelp();
            process.exit(1);
        }
        // PDA derivation
        pool = (await PublicKey.findProgramAddress(
            [Buffer.from("pool"), tokenMint.toBuffer()], 
            PROGRAM_ID
        ))[0];
        pool_account =(await PublicKey.findProgramAddress(
            [Buffer.from("pool_account"), tokenMint.toBuffer()], 
            PROGRAM_ID
        ))[0];
        fluf_mint = (await PublicKey.findProgramAddress(
            [Buffer.from("fluf_mint"), tokenMint.toBuffer()], 
            PROGRAM_ID
        ))[0];
        pool_fluf_account =(await PublicKey.findProgramAddress(
            [Buffer.from("pool_fluf_account"), tokenMint.toBuffer()], 
            PROGRAM_ID
        ))[0];
        fee_account = (await PublicKey.findProgramAddress(
            [Buffer.from("fee_account"), tokenMint.toBuffer()], 
            PROGRAM_ID
        ))[0];
    }
    const state = (await PublicKey.findProgramAddress(
        [Buffer.from("program_state")], 
        PROGRAM_ID
    ))[0];


    const provider = new anchor.AnchorProvider(connection, new anchor.Wallet(payer), { commitment: 'processed', preflightCommitment: 'processed' });
    anchor.setProvider(provider);

    const program = anchor.workspace.Fluf as Program<Fluf>;
    // const programBorrower = anchor.workspace.BorrowerSample as Program<BorrowerSample>;

    const rent = anchor.web3.SYSVAR_RENT_PUBKEY;
    const system_program = SystemProgram.programId;
    const token_program = TOKEN_PROGRAM_ID;

    switch (subcommand) {
        case 'create':
            // Create a new pool for the token mint in the second argument
            console.log("tokenDecimals", tokenDecimals);
            // Call the create_pool function of the program
            const createPoolTx = await program.methods.createPool(tokenDecimals).accounts({
                user: payer.publicKey.toBase58(),
                pool: pool,
                poolMint: tokenMint,
                poolAccount: pool_account,
                flufMint: fluf_mint,
                poolFlufAccount: pool_fluf_account,
                rent: rent,
                systemProgram: system_program,
                tokenProgram: token_program,
            }).signers([payer]).rpc();
            console.log("create_pool transaction signature", createPoolTx);
            console.log("Pool created. Fluf token: ", fluf_mint.toBase58());  
            break;
        case 'deposit':
            // Deposit token_mint amount to the pool
            // Check if for missing arguments
            if (argv._.length < 5) {
                yargs.showHelp();
                process.exit(1);
            }
            const depositTx = await program.methods.deposit(new BN(argv._[2])).accounts({
                user: payer.publicKey.toBase58(),
                pool: pool,
                poolMint: tokenMint,
                poolAccount: pool_account,
                userAccount: new PublicKey(argv._[3]),
                flufMint: fluf_mint,
                poolFlufAccount: pool_fluf_account,
                userFlufAccount: new PublicKey(argv._[4]),
                rent: rent,
                systemProgram: system_program,
                tokenProgram: token_program,
            }).signers([payer]).rpc();
            console.log("deposit transaction signature", depositTx);
            break;
        case 'withdraw':
            // Withdraw token_mint amount to the pool
            // Check if for missing arguments
            if (argv._.length < 4) {
                yargs.showHelp();
                process.exit(1);
            }
            const withdrawTx = await program.methods.withdraw().accounts({
                user: payer.publicKey.toBase58(),
                pool: pool,
                poolMint: tokenMint,
                poolAccount: pool_account,
                userAccount: new PublicKey(argv._[3]),
                flufMint: fluf_mint,
                poolFlufAccount: pool_fluf_account,
                userFlufAccount: new PublicKey(argv._[4]),
                rent: rent,
                systemProgram: system_program,
                tokenProgram: token_program,
            }).signers([payer]).rpc();
            console.log("withdraw transaction signature", withdrawTx);
            break;
        case 'run':
            // Check if for missing arguments
            if (argv._.length < 6) {
                yargs.showHelp();
                process.exit(1);
            }
            const lendAndCallTx = await program.methods.lendAndCall(new BN(argv._[2])).accounts({
                user: payer.publicKey.toBase58(),
                pool: pool,
                poolMint: tokenMint,
                poolAccount: pool_account,
                flufMint: fluf_mint,
                poolFlufAccount: pool_fluf_account,
                borrowerFlufAccount: new PublicKey(argv._[3]),
                userFlufAccount: new PublicKey(argv._[4]),
                feeAccount: fee_account,
                rent: rent,
                systemProgram: system_program,
                tokenProgram: token_program,
                borrowerProgram: new PublicKey(argv._[5]),
            }).signers([payer]).rpc();
            console.log("lendAndCall transaction signature", lendAndCallTx);
            break;
        case 'feesbalance':
            // Get the balance of token_mint in the fee_account
            const ferAccountInfo = await getAccount(
                provider.connection,
                fee_account
            )
            console.log("fee_balance", ferAccountInfo.amount);
            break;
        case 'feeswithdraw':
            // Check if the collector account (third argument) is missing
            if (argv._.length < 3) {
                yargs.showHelp();
                process.exit(1);
            }            
            const collector_account = new PublicKey(argv._[2]);
            const withdrawFeesTx = await program.methods.withdrawFees().accounts({
                user: payer.publicKey.toBase58(),
                state: state,
                pool: pool_account,
                poolMint: tokenMint,
                flufMint: fluf_mint,
                feeAccount: pool_fluf_account,
                collectorAccount: collector_account,
                rent: rent,
                systemProgram: system_program,
                tokenProgram: token_program,
            }).signers([payer]).rpc();
            console.log("withdraw fees transaction signature", withdrawFeesTx);
            break;
        case 'initialize':
            // Record the deployer of the program (for administative purposes)
            const initializeTx = await program.methods.initialize().accounts({
                deployer: payer.publicKey.toBase58(),
                state: state,
                systemProgram: system_program,
            }).signers([payer]).rpc();
            console.log("initialize transaction signature", initializeTx);
            console.log("Protocol initialized for fee collector: ", payer.publicKey.toBase58());
            break;
        default:
            console.log('Unknown subcommand');
    }

};

run();