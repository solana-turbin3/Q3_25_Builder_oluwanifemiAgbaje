import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Escrow } from "../target/types/escrow";
import * as spl from "@solana/spl-token" 
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { ASSOCIATED_TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { Commitment, Keypair, LAMPORTS_PER_SOL, PublicKey, SystemProgram } from "@solana/web3.js";
import { BN } from "bn.js"; // BigNumber for handling large numbers
import makerkey from './wallets/mwallet.json';
import takerkey from './wallets/twallet.json';

//commitment level
const commitment : Commitment = 'confirmed';

describe("escrow", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider)

  const program = anchor.workspace.escrow as Program<Escrow>;

  const connection = provider.connection;

  const seed = new BN(101);

  let maker : Keypair;
  let taker : Keypair;
  let mintA : PublicKey;
  let mintB : PublicKey;
  let makerAtaA : PublicKey;
  let takerAtaB : PublicKey;
  let makerAtaB : PublicKey;
  let takerAtaA : PublicKey;
  let vault : PublicKey;
  let escrow : PublicKey;

  const depositAmount = new BN(200_000_000);
  const recieveAmount = new BN(5_000_000)
  maker = Keypair.fromSecretKey(Uint8Array.from(makerkey));
  taker = Keypair.fromSecretKey(Uint8Array.from(takerkey));

  before( async () => {


    mintA = await spl.createMint(
     connection,
     maker,
     maker.publicKey,
     maker.publicKey,
     6
    );
    console.log(`mintA is: ${mintA.toBase58()}`);

    mintB = await spl.createMint(
     connection,
     taker,
     taker.publicKey,
     taker.publicKey,
     6
    );
    console.log(`mintB is: ${mintB.toBase58()}`);

    makerAtaA = (await spl.getOrCreateAssociatedTokenAccount(
      connection,
      maker,
      mintA,
      maker.publicKey,
      true
    )).address;
    console.log(`Maker ATA for mintA is: ${makerAtaA.toBase58()}`);

    takerAtaA = (await spl.getOrCreateAssociatedTokenAccount(
      connection,
      taker,
      mintA,
      taker.publicKey,
      true
    )).address;
    console.log(`Taker ATA for mintA is: ${takerAtaA.toBase58()}`);

    makerAtaB = (await spl.getOrCreateAssociatedTokenAccount(
      connection,
      maker,
      mintB,
      maker.publicKey,
      true
    )).address;
    console.log(`Maker ATA for mintB is: ${makerAtaB.toBase58()}`);

    takerAtaB = (await spl.getOrCreateAssociatedTokenAccount(
      connection,
      taker,
      mintB,
      taker.publicKey,
      true
    )).address;
    console.log(`Taker ATA for mintB is: ${takerAtaB.toBase58()}`);
  
    const mintATx = await spl.mintTo(
      connection,
      maker,
      mintA,
      makerAtaA,
      maker.publicKey,
      300_000_000
    )
    console.log(`mintATx txid is: ${mintATx}`);

    const mintBTx = await spl.mintTo(
      connection,
      taker,
      mintB,
      takerAtaB,
      taker.publicKey,
      10_000_000
    )
    console.log(`mintBTx txid is: ${mintBTx}`);

    // Calculate escrow PDA
    [escrow] = PublicKey.findProgramAddressSync([
      Buffer.from('escrow'),
      maker.publicKey.toBuffer(),
      seed.toArrayLike(Buffer, "le", 8),],
      program.programId
    );

    // Calculate vault ATA
    vault = await spl.getAssociatedTokenAddressSync(mintA, escrow, true);
    
    console.log(`Escrow PDA: ${escrow.toBase58()}`);
    console.log(`Vault ATA: ${vault.toBase58()}`);
  })

  it("Initialize Escrow!, make offer", async () => {
    // Add your test here.
    const tx = await program.methods.make(
      seed,
      recieveAmount,
      depositAmount,
    )
    .accountsPartial({
      maker: maker.publicKey,
      mintA: mintA,
      mintB: mintB,
      makerAtaA: makerAtaA,
      escrow: escrow,
      vault: vault,
      tokenProgram: TOKEN_PROGRAM_ID,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    })
    .signers([maker])
    .rpc();
    console.log("Your transaction signature", tx);
  });

  xit("take offer", async () => {
    const tx = await program.methods.take()
    .accountsPartial({
      taker: taker.publicKey,
      maker: maker.publicKey,
      mintA: mintA,
      mintB: mintB,
      takerAtaA: takerAtaA,
      takerAtaB: takerAtaB,
      makerAtaB: makerAtaB,
      escrow: escrow,
      vault: vault,
      tokenProgram: TOKEN_PROGRAM_ID,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    })
    .signers([taker])
    .rpc();
    console.log("Your transaction signature", tx);
  });

  it("Refund and close offer", async () => {
    const tx = await program.methods.refund()
    .accountsPartial({
      maker: maker.publicKey,
      mintA: mintA,
      makerAtaA: makerAtaA,
      vault: vault,
      escrow: escrow,
      tokenProgram: TOKEN_PROGRAM_ID,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    })
    .signers([maker])
    .rpc();
    console.log("Your transaction signature", tx);
  });
});