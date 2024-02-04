import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { expect } from 'chai';
import { Transaction, PublicKey, SystemProgram } from "@solana/web3.js";
import { createMint, getOrCreateAssociatedTokenAccount, mintTo, transfer, createSetAuthorityInstruction, createTransferInstruction, AuthorityType, getAccount, getMint, Token, TOKEN_PROGRAM_ID, MintLayout } from "@solana/spl-token";
import { Fluf } from "../target/types/fluf";
import { BorrowerSample } from "../target/types/borrower_sample";

const PROGRAM_ID = new PublicKey("7Crsw9yaDiT5jMZ8yWJgkdVeWpLirh9G5hJZCp9G1Aiy");

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
  const programBorrowerSample = anchor.workspace.Fluf as Program<BorrowerSample>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
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
    // #[account(mut)]
    // pub initializer: Signer<'info>,
    // #[account(init, payer = initializer, space = 8 + size_of::<Pool>(), seeds = [b"pool".as_ref(), pool_mint.key().as_ref()], bump)]
    // pub pool: Account<'info, Pool>,
    // #[account(mut)]
    // pub pool_mint: Account<'info, Mint>,
    // #[account(init, payer = initializer, space = Mint::LEN, seeds = [b"wrapped".as_ref(), pool_mint.key().as_ref()], bump)]
    // pub wrapped_mint: Account<'info, Mint>,
    // #[account(init, payer = initializer, space = Mint::LEN, seeds = [b"voucher".as_ref(), pool_mint.key().as_ref()], bump)]
    // pub voucher_mint: Account<'info, Mint>,
    // pub rent: Sysvar<'info, Rent>,
    // pub system_program: Program<'info, System>,
    // pub token_program: Program<'info, Token>,
    const initializer = poolInvestor1.publicKey;
    const pool = (await PublicKey.findProgramAddress(
      [Buffer.from("pool"), tokenMint.toBuffer()], 
      PROGRAM_ID
    ))[0];
    const pool_mint = tokenMint;
    const wrapped_mint = (await PublicKey.findProgramAddress(
      [Buffer.from("wrapped"), tokenMint.toBuffer()], 
      PROGRAM_ID
    ))[0];
    const voucher_mint = (await PublicKey.findProgramAddress(
      [Buffer.from("voucher"), tokenMint.toBuffer()], 
      PROGRAM_ID
    ))[0];
    const rent = anchor.web3.SYSVAR_RENT_PUBKEY;
    const system_program = SystemProgram.programId;
    const token_program = TOKEN_PROGRAM_ID;
  
    // Call the create_pool function of the program
    const createPoolTx = await program.methods.createPool(TOKEN_DECIMALS).accounts({
      // List of accounts:
      initializer: initializer,
      pool: pool,
      poolMint: pool_mint,
      wrappedMint: wrapped_mint,
      voucherMint: voucher_mint,
      rent: rent,
      systemProgram: system_program,
      tokenProgram: token_program,
    }).signers([poolInvestor1]).rpc();
    console.log("create_pool transaction signature", createPoolTx);
  });

});
