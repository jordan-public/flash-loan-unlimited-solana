import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { expect } from 'chai';
import { Transaction, PublicKey, SystemProgram } from "@solana/web3.js";
import { createMint, getOrCreateAssociatedTokenAccount, mintTo, transfer, createSetAuthorityInstruction, createTransferInstruction, AuthorityType, getAccount, getMint, Token, TOKEN_PROGRAM_ID, MintLayout } from "@solana/spl-token";
import { Fluf } from "../target/types/fluf";
import { BorrowerSample } from "../target/types/borrower_sample";
import BN from 'bn.js';

const PROGRAM_ID = new PublicKey("7Crsw9yaDiT5jMZ8yWJgkdVeWpLirh9G5hJZCp9G1Aiy");
const PROGRAM_BORROWER_ID = new PublicKey("5N7gCufd5hEVkcHVSwtUmAKaHvNNagkq7T4qcUYzJ91y");

const TOKEN_DECIMALS = 9;

async function createToken(connection: anchor.Provider.Connection, creator: anchor.web3.Keypair, recipient: andchor.web3.Keypair, decimals: number, amount: number) {
  // Create a new mint
  const mint = await createMint(
    connection,
    creator,
    creator.publicKey,
    null,
    decimals
  );
  // Create an account to hold tokens of this new type
  const associatedTokenAccount = await getOrCreateAssociatedTokenAccount(
    connection,
    creator,
    mint,
    recipient.publicKey
  );
  // Mint only one token into the account
  await mintTo(
    connection,
    creator,
    mint,
    associatedTokenAccount.address,
    creator.publicKey,
    amount
  );
  // Disable future minting
  let transaction = new Transaction()
  .add(createSetAuthorityInstruction(
    mint,
    creator.publicKey,
    AuthorityType.MintTokens,
    null
  ));
  // const accountInfo = await getAccount(connection, associatedTokenAccount.address);
  // See result
  // const mintInfo = await getMint(
  //     connection,
  //     mint
  //   );
  // console.log("Quantity:", accountInfo.amount);
  // console.log("Mint info:", mintInfo);
  // Return the mint
  return mint
}

describe("fluf", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.Fluf as Program<Fluf>;
  const programBorrowerSample = anchor.workspace.BorrowerSample as Program<BorrowerSample>;

  it("Is initialized!", async () => {
    // Assuming AnchorProvider is set up with a funded wallet
    const provider = anchor.AnchorProvider.env();

    const state = (await PublicKey.findProgramAddress(
      [Buffer.from("program_state")], 
      PROGRAM_ID
    ))[0];
    const tx = await program.methods.initialize().accounts({
      deployer: provider.wallet.publicKey,
      state: state,
      systemProgram: SystemProgram.programId,
    }).rpc();
    console.log("Your transaction signature", tx);
  });

  it("Initializes pool", async () => {
    // Assuming AnchorProvider is set up with a funded wallet
    const provider = anchor.AnchorProvider.env();

    // Generate keypairs for the NFT holder and the swap offerer
    const tokenAuthority = anchor.web3.Keypair.generate();
    const flashLoanInitiator = anchor.web3.Keypair.generate();
    const poolInvestor1 = anchor.web3.Keypair.generate();
    const poolInvestor2 = anchor.web3.Keypair.generate();
    // Airdrop 5 sol into each of the above accounts
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(tokenAuthority.publicKey, 5e9),
      "confirmed"
    );
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(flashLoanInitiator.publicKey, 5e9),
      "confirmed"
    );
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(poolInvestor1.publicKey, 5e9),
      "confirmed"
    );
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(poolInvestor2.publicKey, 5e9),
      "confirmed"
    );
    
    // Check the balances of the accounts
    console.log("tokenAuthority balance", await provider.connection.getBalance(tokenAuthority.publicKey));
    console.log("flashLoanInitiator balance", await provider.connection.getBalance(flashLoanInitiator.publicKey));
    console.log("poolInvestor1 balance", await provider.connection.getBalance(poolInvestor1.publicKey));
    console.log("poolInvestor2 balance", await provider.connection.getBalance(poolInvestor2.publicKey));
  
    // Mint 1000 tokens for the pool investors
    const tokenMint = await createToken(provider.connection, tokenAuthority, tokenAuthority, TOKEN_DECIMALS, 1_000_000_000_000); // 1000 tokens
    console.log("Token mint", tokenMint);

    const tokenAuthorityTokenAccount = (await getOrCreateAssociatedTokenAccount(provider.connection, tokenAuthority, tokenMint, tokenAuthority.publicKey)).address;
    const flashLoanInitiatorTokenAccount = (await getOrCreateAssociatedTokenAccount(provider.connection, flashLoanInitiator, tokenMint, flashLoanInitiator.publicKey)).address;
    const poolInvestor1TokenAccount = (await getOrCreateAssociatedTokenAccount(provider.connection, poolInvestor1, tokenMint, poolInvestor1.publicKey)).address;
    const poolInvestor2TokenAccount = (await getOrCreateAssociatedTokenAccount(provider.connection, poolInvestor2, tokenMint, poolInvestor2.publicKey)).address;

    // Transfer 100 tokens to the pool investor 1
    await transfer (
      provider.connection,
      tokenAuthority,
      tokenAuthorityTokenAccount,
      poolInvestor1TokenAccount,
      tokenAuthority.publicKey,
      100_000_000_000
    );
    {
      const tokenAccountInfo = await getAccount(
        provider.connection,
        poolInvestor1TokenAccount
      )
      console.log("poolInvestor1TokenAccount token balance", tokenAccountInfo.amount);
    }

    // Transfer 200 tokens to the pool investor 2
    await transfer (
      provider.connection,
      tokenAuthority,
      tokenAuthorityTokenAccount,
      poolInvestor2TokenAccount,
      tokenAuthority.publicKey,
      200_000_000_000
    );
    {
      const tokenAccountInfo = await getAccount(
        provider.connection,
        poolInvestor2TokenAccount
      )
      console.log("poolInvestor2TokenAccount token balance", tokenAccountInfo.amount);
    }
    
    // Prepare the accounts for the create_pool function
    const user = poolInvestor1.publicKey;
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
  
    // Call the create_pool function of the program
    const createPoolTx = await program.methods.createPool(TOKEN_DECIMALS).accounts({
      // List of accounts:
      user: user,
      pool: pool,
      poolMint: pool_mint,
      poolAccount: pool_account,
      flufMint: fluf_mint,
      poolFlufAccount: pool_fluf_account,
      rent: rent,
      systemProgram: system_program,
      tokenProgram: token_program,
    }).signers([poolInvestor1]).rpc();
    console.log("create_pool transaction signature", createPoolTx);
  });

  it("Deposits and withdraws", async () => {
    // Assuming AnchorProvider is set up with a funded wallet
    const provider = anchor.AnchorProvider.env();

    // Generate keypairs for the NFT holder and the swap offerer
    const tokenAuthority = anchor.web3.Keypair.generate();
    const flashLoanInitiator = anchor.web3.Keypair.generate();
    const poolInvestor1 = anchor.web3.Keypair.generate();
    const poolInvestor2 = anchor.web3.Keypair.generate();
    // Airdrop 5 sol into each of the above accounts
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(tokenAuthority.publicKey, 5e9),
      "confirmed"
    );
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(flashLoanInitiator.publicKey, 5e9),
      "confirmed"
    );
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(poolInvestor1.publicKey, 5e9),
      "confirmed"
    );
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(poolInvestor2.publicKey, 5e9),
      "confirmed"
    );
    
    // Check the balances of the accounts
    console.log("tokenAuthority balance", await provider.connection.getBalance(tokenAuthority.publicKey));
    console.log("flashLoanInitiator balance", await provider.connection.getBalance(flashLoanInitiator.publicKey));
    console.log("poolInvestor1 balance", await provider.connection.getBalance(poolInvestor1.publicKey));
    console.log("poolInvestor2 balance", await provider.connection.getBalance(poolInvestor2.publicKey));
  
    // Mint 1000 tokens for the pool investors
    const tokenMint = await createToken(provider.connection, tokenAuthority, tokenAuthority, TOKEN_DECIMALS, 1_000_000_000_000); // 1000 tokens
    console.log("Token mint", tokenMint);

    const tokenAuthorityTokenAccount = (await getOrCreateAssociatedTokenAccount(provider.connection, tokenAuthority, tokenMint, tokenAuthority.publicKey)).address;
    const flashLoanInitiatorTokenAccount = (await getOrCreateAssociatedTokenAccount(provider.connection, flashLoanInitiator, tokenMint, flashLoanInitiator.publicKey)).address;
    const poolInvestor1TokenAccount = (await getOrCreateAssociatedTokenAccount(provider.connection, poolInvestor1, tokenMint, poolInvestor1.publicKey)).address;
    const poolInvestor2TokenAccount = (await getOrCreateAssociatedTokenAccount(provider.connection, poolInvestor2, tokenMint, poolInvestor2.publicKey)).address;

    // Transfer 100 tokens to the pool investor 1
    await transfer (
      provider.connection,
      tokenAuthority,
      tokenAuthorityTokenAccount,
      poolInvestor1TokenAccount,
      tokenAuthority.publicKey,
      100_000_000_000
    );
    {
      const tokenAccountInfo = await getAccount(
        provider.connection,
        poolInvestor1TokenAccount
      )
      console.log("poolInvestor1TokenAccount token balance", tokenAccountInfo.amount);
    }

    // Transfer 200 tokens to the pool investor 2
    await transfer (
      provider.connection,
      tokenAuthority,
      tokenAuthorityTokenAccount,
      poolInvestor2TokenAccount,
      tokenAuthority.publicKey,
      200_000_000_000
    );
    {
      const tokenAccountInfo = await getAccount(
        provider.connection,
        poolInvestor2TokenAccount
      )
      console.log("poolInvestor2TokenAccount token balance", tokenAccountInfo.amount);
    }
    
    // Prepare the accounts for the create_pool function
    const user = poolInvestor1.publicKey;
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
  
    // Call the create_pool function of the program
    const createPoolTx = await program.methods.createPool(TOKEN_DECIMALS).accounts({
      user: user,
      pool: pool,
      poolMint: pool_mint,
      poolAccount: pool_account,
      flufMint: fluf_mint,
      poolFlufAccount: pool_fluf_account,
      rent: rent,
      systemProgram: system_program,
      tokenProgram: token_program,
    }).signers([poolInvestor1]).rpc();
    console.log("create_pool transaction signature", createPoolTx);

    const tokenAuthorityFlufTokenAccount = (await getOrCreateAssociatedTokenAccount(provider.connection, tokenAuthority, fluf_mint, tokenAuthority.publicKey)).address;
    const flashLoanInitiatorFlufTokenAccount = (await getOrCreateAssociatedTokenAccount(provider.connection, flashLoanInitiator, fluf_mint, flashLoanInitiator.publicKey)).address;
    const poolInvestor1FlufTokenAccount = (await getOrCreateAssociatedTokenAccount(provider.connection, poolInvestor1, fluf_mint, poolInvestor1.publicKey)).address;
    const poolInvestor2FlufTokenAccount = (await getOrCreateAssociatedTokenAccount(provider.connection, poolInvestor2, fluf_mint, poolInvestor2.publicKey)).address;

    // const user_fluf_account = (await PublicKey.findProgramAddress(
    //   [Buffer.from("user_fluf_account"), tokenMint.toBuffer(), poolInvestor1.publicKey.toBuffer()], 
    //   PROGRAM_ID
    // ))[0];

    // Deposit 50 tokens T into the pool to get 50 fT tokens
    const depositTx = await program.methods.deposit(new BN(50_000_000_000)).accounts({
      user: poolInvestor1.publicKey,
      pool: pool,
      poolMint: pool_mint,
      poolAccount: pool_account,
      userAccount: poolInvestor1TokenAccount,
      flufMint: fluf_mint,
      poolFlufAccount: pool_fluf_account,
      userFlufAccount: poolInvestor1FlufTokenAccount,
      rent: rent,
      systemProgram: system_program,
      tokenProgram: token_program,
    }).signers([poolInvestor1]).rpc();  
    console.log("deposit transaction signature", depositTx);

    // Get the pool token balance of the user_fluf_account
    const userAccountInfo = await getAccount(
      provider.connection,
      poolInvestor1TokenAccount
    );
    console.log("poolInvestor1TokenAccount token balance", userAccountInfo.amount);
    // Get the fluf_mint token balance of the user_fluf_account
    const userFlufAccountInfo = await getAccount(
      provider.connection,
      poolInvestor1FlufTokenAccount
    );
    console.log("poolInvestor1FlufTokenAccount token balance", userFlufAccountInfo.amount);
  
    // Withdraw all 50 tokens fT from the pool to get 50 T tokens (initially same value)
    const withdrawTx = await program.methods.withdraw().accounts({
      user: poolInvestor1.publicKey,
      pool: pool,
      poolMint: pool_mint,
      poolAccount: pool_account,
      userAccount: poolInvestor1TokenAccount,
      flufMint: fluf_mint,
      poolFlufAccount: pool_fluf_account,
      userFlufAccount: poolInvestor1FlufTokenAccount,
      rent: rent,
      systemProgram: system_program,
      tokenProgram: token_program,
    }).signers([poolInvestor1]).rpc();

    // Get the pool token balance of the user_fluf_account
    {
    const userAccountInfo = await getAccount(
      provider.connection,
      poolInvestor1TokenAccount
    );
    console.log("poolInvestor1TokenAccount token balance", userAccountInfo.amount);
    // Get the fluf_mint token balance of the user_fluf_account
    const userFlufAccountInfo = await getAccount(
      provider.connection,
      poolInvestor1FlufTokenAccount
    );
    console.log("poolInvestor1FlufTokenAccount token balance", userFlufAccountInfo.amount);
    }
    
  });

  it("Flash-lends and executes", async () => {
    // Assuming AnchorProvider is set up with a funded wallet
    const provider = anchor.AnchorProvider.env();

    // Generate keypairs for the NFT holder and the swap offerer
    const tokenAuthority = anchor.web3.Keypair.generate();
    const flashLoanInitiator = anchor.web3.Keypair.generate();
    const poolInvestor1 = anchor.web3.Keypair.generate();
    const poolInvestor2 = anchor.web3.Keypair.generate();
    // Airdrop 5 sol into each of the above accounts
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(tokenAuthority.publicKey, 5e9),
      "confirmed"
    );
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(flashLoanInitiator.publicKey, 5e9),
      "confirmed"
    );
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(poolInvestor1.publicKey, 5e9),
      "confirmed"
    );
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(poolInvestor2.publicKey, 5e9),
      "confirmed"
    );
    
    // Check the balances of the accounts
    console.log("tokenAuthority balance", await provider.connection.getBalance(tokenAuthority.publicKey));
    console.log("flashLoanInitiator balance", await provider.connection.getBalance(flashLoanInitiator.publicKey));
    console.log("poolInvestor1 balance", await provider.connection.getBalance(poolInvestor1.publicKey));
    console.log("poolInvestor2 balance", await provider.connection.getBalance(poolInvestor2.publicKey));
  
    // Mint 1000 tokens for the pool investors
    const tokenMint = await createToken(provider.connection, tokenAuthority, tokenAuthority, TOKEN_DECIMALS, 1_000_000_000_000); // 1000 tokens
    console.log("Token mint", tokenMint);

    const tokenAuthorityTokenAccount = (await getOrCreateAssociatedTokenAccount(provider.connection, tokenAuthority, tokenMint, tokenAuthority.publicKey)).address;
    const flashLoanInitiatorTokenAccount = (await getOrCreateAssociatedTokenAccount(provider.connection, flashLoanInitiator, tokenMint, flashLoanInitiator.publicKey)).address;
    const poolInvestor1TokenAccount = (await getOrCreateAssociatedTokenAccount(provider.connection, poolInvestor1, tokenMint, poolInvestor1.publicKey)).address;
    const poolInvestor2TokenAccount = (await getOrCreateAssociatedTokenAccount(provider.connection, poolInvestor2, tokenMint, poolInvestor2.publicKey)).address;

    // Transfer 100 tokens to the pool investor 1
    await transfer (
      provider.connection,
      tokenAuthority,
      tokenAuthorityTokenAccount,
      poolInvestor1TokenAccount,
      tokenAuthority.publicKey,
      100_000_000_000
    );
    {
      const tokenAccountInfo = await getAccount(
        provider.connection,
        poolInvestor1TokenAccount
      )
      console.log("poolInvestor1TokenAccount token balance", tokenAccountInfo.amount);
    }

    // Transfer 200 tokens to the pool investor 2
    await transfer (
      provider.connection,
      tokenAuthority,
      tokenAuthorityTokenAccount,
      poolInvestor2TokenAccount,
      tokenAuthority.publicKey,
      200_000_000_000
    );
    {
      const tokenAccountInfo = await getAccount(
        provider.connection,
        poolInvestor2TokenAccount
      )
      console.log("poolInvestor2TokenAccount token balance", tokenAccountInfo.amount);
    }
    
    // Prepare the accounts for the create_pool function
    const user = poolInvestor1.publicKey;
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
  
    // Call the create_pool function of the program
    const createPoolTx = await program.methods.createPool(TOKEN_DECIMALS).accounts({
      user: user,
      pool: pool,
      poolMint: pool_mint,
      poolAccount: pool_account,
      flufMint: fluf_mint,
      poolFlufAccount: pool_fluf_account,
      rent: rent,
      systemProgram: system_program,
      tokenProgram: token_program,
    }).signers([poolInvestor1]).rpc();
    console.log("create_pool transaction signature", createPoolTx);

    const tokenAuthorityFlufTokenAccount = (await getOrCreateAssociatedTokenAccount(provider.connection, tokenAuthority, fluf_mint, tokenAuthority.publicKey)).address;
    const flashLoanInitiatorFlufTokenAccount = (await getOrCreateAssociatedTokenAccount(provider.connection, flashLoanInitiator, fluf_mint, flashLoanInitiator.publicKey)).address;
    const poolInvestor1FlufTokenAccount = (await getOrCreateAssociatedTokenAccount(provider.connection, poolInvestor1, fluf_mint, poolInvestor1.publicKey)).address;
    const poolInvestor2FlufTokenAccount = (await getOrCreateAssociatedTokenAccount(provider.connection, poolInvestor2, fluf_mint, poolInvestor2.publicKey)).address;

    // Deposit 50 tokens T into the pool to get 50 fT tokens
    const depositTx = await program.methods.deposit(new BN(50_000_000_000)).accounts({
      user: poolInvestor1.publicKey,
      pool: pool,
      poolMint: pool_mint,
      poolAccount: pool_account,
      userAccount: poolInvestor1TokenAccount,
      flufMint: fluf_mint,
      poolFlufAccount: pool_fluf_account,
      userFlufAccount: poolInvestor1FlufTokenAccount,
      rent: rent,
      systemProgram: system_program,
      tokenProgram: token_program,
    }).signers([poolInvestor1]).rpc();  
    console.log("deposit transaction signature", depositTx);

    // Get the pool token balance of the user_fluf_account
    const userAccountInfo = await getAccount(
      provider.connection,
      poolInvestor1TokenAccount
    );
    console.log("poolInvestor1TokenAccount token balance", userAccountInfo.amount);
    // Get the fluf_mint token balance of the user_fluf_account
    const userFlufAccountInfo = await getAccount(
      provider.connection,
      poolInvestor1FlufTokenAccount
    );
    console.log("poolInvestor1FlufTokenAccount token balance", userFlufAccountInfo.amount);
    
    // Call the lendAndCall function of the program
    const borrower_account = (await PublicKey.findProgramAddress(
      [Buffer.from("borrower_account"), fluf_mint.toBuffer()], 
      PROGRAM_BORROWER_ID
    ))[0];
    console.log("borrower_account", borrower_account);
    const createAccountsTx = await programBorrowerSample.methods.createAccounts().accounts({
      user: flashLoanInitiator.publicKey,
      borrowerAccount: borrower_account,
      mint: fluf_mint,
      systemProgram: system_program,
      tokenProgram: token_program,
    }).signers([flashLoanInitiator]).rpc();
    console.log("createAccounts transaction signature", createAccountsTx);

    // poolInvestor1 gives 10 fT tokens from his poolInvestor1FlufTokenAccount to the flashLoanInitiatorFlufTokenAccount
    // !!! This should be done atomically in a single transaction with the lendAndCall function but in a separate instruction
    await transfer (
      provider.connection,
      poolInvestor1,
      poolInvestor1FlufTokenAccount,
      borrower_account,
      poolInvestor1.publicKey,
      10_000_000_000
    );

    const fee_account = (await PublicKey.findProgramAddress(
      [Buffer.from("fee_account"), tokenMint.toBuffer()], 
      PROGRAM_ID
    ))[0];
    const lendAndCallTx = await program.methods.lendAndCall(new BN(300_000_000_000)).accounts({
      user: flashLoanInitiator.publicKey,
      pool: pool,
      poolMint: pool_mint,
      poolAccount: pool_account,
      flufMint: fluf_mint,
      poolFlufAccount: pool_fluf_account,
      borrowerFlufAccount: borrower_account,
      userFlufAccount: flashLoanInitiatorFlufTokenAccount,
      feeAccount: fee_account,
      rent: rent,
      systemProgram: system_program,
      tokenProgram: token_program,
      borrowerProgram: PROGRAM_BORROWER_ID,
    }).signers([flashLoanInitiator]).rpc();
    console.log("lendAndCall transaction signature", lendAndCallTx);
  });
});
