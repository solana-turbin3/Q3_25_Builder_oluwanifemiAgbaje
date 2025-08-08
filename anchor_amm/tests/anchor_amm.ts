// Import necessary Anchor libraries for Solana program interaction
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { AnchorAmm } from "../target/types/anchor_amm"; // Import the AMM program type
import {
  TOKEN_PROGRAM_ID,
  createMint,
  getAssociatedTokenAddressSync,
  getOrCreateAssociatedTokenAccount,
  mintTo,
  getAccount,
} from "@solana/spl-token"; // SPL token utilities for creating and managing tokens
import NodeWallet from "@coral-xyz/anchor/dist/cjs/nodewallet";
import { ASSOCIATED_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/utils/token";
import { Commitment, Keypair, LAMPORTS_PER_SOL, PublicKey, SystemProgram } from "@solana/web3.js";
import { BN } from "bn.js"; // BigNumber for handling large numbers

// Define the commitment level for Solana transactions
const commitment: Commitment = 'confirmed';

// Test suite for the Anchor AMM program
describe("anchor_amm", () => {
  // Set up the Anchor provider to connect to the local Solana cluster
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  // Load the AMM program from the workspace
  const program = anchor.workspace.anchorAmm as Program<AnchorAmm>;

  // Get the connection to the Solana cluster
  const connection = provider.connection;

  // Define a seed for deterministic address generation
  const seed = new BN(260);

  const payer = provider.wallet as NodeWallet;

  let mintX: PublicKey; // Mint address for token X
  let mintY: PublicKey; // Mint address for token Y
  let vaultX: PublicKey; // Vault for token X in the AMM pool
  let vaultY: PublicKey; // Vault for token Y in the AMM pool
  let userAtaX: PublicKey; // User's associated token account for token X
  let userAtaY: PublicKey; // User's associated token account for token Y
  let userLp: PublicKey; // User's associated token account for LP tokens

  // Derive the config account address using the seed
  let config = PublicKey.findProgramAddressSync([
    Buffer.from("config"), 
    seed.toBuffer("le", 8)], 
    program.programId)[0];

  // Derive the LP token mint address
  let mintLp = PublicKey.findProgramAddressSync([
    Buffer.from('lp'),
    config.toBuffer()], 
    program.programId)[0];

  // Test case to create token mints and associated token accounts
  it("create mints, derive TAs !", async () => {
    // Create a new mint for token X with 6 decimals
    mintX = await createMint(
      connection,
      payer.payer, // Payer for transaction fees
      provider.publicKey, // Mint authority
      provider.publicKey, // Freeze authority
      6 // Decimals
    );

    // Create a new mint for token Y with 6 decimals
    mintY = await createMint(
      connection,
      payer.payer,
      provider.publicKey,
      provider.publicKey,
      6
    );

    // Derive the associated token addresses for the vaults
    vaultX = await getAssociatedTokenAddressSync(mintX, config, true); // Allow off-curve addresses for program-owned accounts
    vaultY = await getAssociatedTokenAddressSync(mintY, config, true);

    // Derive the user's LP token account
    userLp = await getAssociatedTokenAddressSync(mintLp, provider.publicKey);

    // Create or get the user's associated token account for token X
    userAtaX = (await getOrCreateAssociatedTokenAccount(
      connection,
      payer.payer,
      mintX,
      provider.publicKey,
    )).address;

    // Create or get the user's associated token account for token Y
    userAtaY = (await getOrCreateAssociatedTokenAccount(
      connection,
      payer.payer,
      mintY,
      provider.publicKey,
    )).address;

    // Mint 200,000,000 units of token X to the user's ATA
    await mintTo(
      connection,
      payer.payer,
      mintX,
      userAtaX,
      provider.publicKey,
      200_000_000
    );

    // Mint 200,000,000 units of token Y to the user's ATA
    await mintTo(
      connection,
      payer.payer,
      mintY,
      userAtaY,
      provider.publicKey,
      200_000_000
    );
  });


  // Test case to initialize the AMM pool
  it("Is initialized!", async () => {
    // Call the initialize method to set up the AMM pool
    const tx = await program.methods.initialize(
      seed, // Seed for config address derivation
      500, // Fee (e.g., 500 basis points = 5%)
      provider.publicKey // Initializer's public key
    )
    .accountsPartial({
      initializer: provider.publicKey, // Account initializing the pool
      mintX: mintX, // Token X mint
      mintY: mintY, // Token Y mint
      mintLp: mintLp, // LP token mint
      vaultX: vaultX, // Vault for token X
      vaultY: vaultY, // Vault for token Y
      config: config, // Config account for the pool
      tokenProgram: TOKEN_PROGRAM_ID, // SPL Token program
      associatedTokenProgram: ASSOCIATED_PROGRAM_ID, // Associated token program
      systemProgram: SystemProgram.programId, // Solana system program
    })
    .rpc();
    console.log("Your transaction signature", tx);
  });

  // Test case to deposit tokens into the AMM pool
  it("Deposit Tokens", async () => {
    // Call the deposit method to add liquidity
    const tx = await program.methods.deposit(
      new BN(1000_000_000), // LP token amount
      new BN(10_000_000), // Max token X amount
      new BN(10_000_000) // Max token Y amount
    )
    .accountsPartial({
      user: provider.publicKey, // User depositing tokens
      mintX: mintX,
      mintY: mintY,
      config: config,
      mintLp: mintLp,
      vaultX: vaultX,
      vaultY: vaultY,
      userX: userAtaX, // User's token X account
      userY: userAtaY, // User's token Y account
      userLp: userLp, // User's LP token account
      tokenProgram: TOKEN_PROGRAM_ID,
      associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    })
.rpc();

console.log("Your transaction signature", tx);
const vaultXBalance = await getAccount(connection, vaultX);
const vaultYBalance = await getAccount(connection, vaultY);
const userAtaXBalance = await getAccount(connection, userAtaX);
const userAtaYBalance = await getAccount(connection, userAtaY);
const userLpBalance = await getAccount(connection, userLp);

console.log(`Vault X Balance: ${vaultXBalance.amount.toString()}`);
console.log(`Vault Y Balance: ${vaultYBalance.amount.toString()}`);
console.log(`User ATA X Balance: ${userAtaXBalance.amount.toString()}`);
console.log(`User ATA Y Balance: ${userAtaYBalance.amount.toString()}`);
console.log(`User LP Balance: ${userLpBalance.amount.toString()}`);

  });

  // Test case to swap tokens in the AMM pool
  it("Swap Tokens", async () => {
    // Call the swap method to exchange tokens
    const tx = await program.methods.swap(
      true, // Direction: true for X to Y, false for Y to X
      new BN(4_000_000), // Amount to swap
      new BN(2_000_000) // Minimum amount to receive
    )
    .accountsPartial({
      user: provider.publicKey, // User performing the swap
      mintX: mintX,
      mintY: mintY,
      config: config,
      vaultX: vaultX,
      vaultY: vaultY,
      userX: userAtaX,
      userY: userAtaY,
      tokenProgram: TOKEN_PROGRAM_ID,
      associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    })
    .rpc();
    console.log("Your transaction signature", tx);
    
    const vaultXBalance = await getAccount(connection, vaultX);
    const vaultYBalance = await getAccount(connection, vaultY);
    const userAtaXBalance = await getAccount(connection, userAtaX);
    const userAtaYBalance = await getAccount(connection, userAtaY);
    const userLpBalance = await getAccount(connection, userLp);

    console.log(`Vault X Balance: ${vaultXBalance.amount.toString()}`);
    console.log(`Vault Y Balance: ${vaultYBalance.amount.toString()}`);
    console.log(`User ATA X Balance: ${userAtaXBalance.amount.toString()}`);
    console.log(`User ATA Y Balance: ${userAtaYBalance.amount.toString()}`);
    console.log(`User LP Balance: ${userLpBalance.amount.toString()}`);

  });

  // Test case to withdraw tokens from the AMM pool
  it("Withdraw Tokens", async () => {
    // Call the withdraw method to remove liquidity
    const tx = await program.methods.withdraw(
      new BN(1000_000_000), // LP token amount to burn
      new BN(10_000_000), // Min token X to receive
      new BN(10_000_000) // Min token Y to receive
    )
    .accountsPartial({
      user: provider.publicKey, // User withdrawing tokens
      mintX: mintX,
      mintY: mintY,
      config: config,
      mintLp: mintLp,
      vaultX: vaultX,
      vaultY: vaultY,
      userX: userAtaX,
      userY: userAtaY,
      userLp: userLp,
      tokenProgram: TOKEN_PROGRAM_ID,
      associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    })
    .rpc();
    console.log("Your transaction signature", tx);

    const vaultXBalance = await getAccount(connection, vaultX);
    const vaultYBalance = await getAccount(connection, vaultY);
    const userAtaXBalance = await getAccount(connection, userAtaX);
    const userAtaYBalance = await getAccount(connection, userAtaY);
    const userLpBalance = await getAccount(connection, userLp);

    console.log(`Vault X Balance: ${vaultXBalance.amount.toString()}`);
    console.log(`Vault Y Balance: ${vaultYBalance.amount.toString()}`);
    console.log(`User ATA X Balance: ${userAtaXBalance.amount.toString()}`);
    console.log(`User ATA Y Balance: ${userAtaYBalance.amount.toString()}`);
    console.log(`User LP Balance: ${userLpBalance.amount.toString()}`);

  });
});