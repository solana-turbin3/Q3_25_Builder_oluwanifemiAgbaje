import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Vault } from "../target/types/vault";
import { PublicKey, SystemProgram, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { expect } from "chai";
import { it } from "mocha";

describe("vault", () => {
// Configure the client to use the local cluster
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.Vault as Program<Vault>;
  const provider = anchor.getProvider();

  const user = provider.wallet as anchor.Wallet;

  // Test accounts
  let vaultPda: anchor.web3.PublicKey;
  let vaultStatePda: anchor.web3.PublicKey;
  let vaultBump: number;
  let stateBump: number;

  before (async() =>{
    //Derive PDAs
    // VaultState PDA
    [vaultStatePda, stateBump] = PublicKey.findProgramAddressSync(
      [Buffer.from('state'), user.publicKey.toBuffer()],
      program.programId
    );

    // Vault PDA
    [vaultPda, vaultBump] = PublicKey.findProgramAddressSync(
      [Buffer.from('vault'), vaultStatePda.toBuffer()],
      program.programId
    );

    console.log("Vault PDA:", vaultPda.toString());
    console.log("Vault State PDA:", vaultStatePda.toString());
    console.log("Signer:", user.publicKey.toString());
  })

  

  it("Is initialized!", async () => {
    // Add your test here.
    try {
      const tx = await program.methods.initialize()
    .accountsPartial({
      signer: user.publicKey,
      vault: vaultPda,
      vaultState: vaultStatePda,
      systemProgram: SystemProgram.programId,
    })
    .rpc()

    // Verify the vault state was created correctly
      const vaultState = await program.account.vaultState.fetch(vaultStatePda);
      expect(vaultState.vaultBump).to.equal(vaultBump);
      expect(vaultState.stateBump).to.equal(stateBump);

      // Check that the vault account was created and has rent-exempt balance
      const vaultAccount = await provider.connection.getAccountInfo(vaultPda);
      expect(vaultAccount).to.not.be.null;
      expect(vaultAccount!.lamports).to.be.greaterThan(0);

      console.log("Initialize vault test passed ✅");
      console.log("Initialize transaction signature:", tx)
      
    } catch (error) {
      console.error("Initialize vault test failed ❌, error:", error);
      throw error;
    }
    
});

it("Deposits lamports to the vault", async () => {
    const depositAmount = 0.3 * LAMPORTS_PER_SOL; // 0.1 SOL

    // Get initial balances
    const initialVaultBalance = await provider.connection.getBalance(vaultPda);
    const initialSignerBalance = await provider.connection.getBalance(user.publicKey);

    try {
      const tx = await program.methods
        .deposit(new anchor.BN(depositAmount))
        .accountsPartial({
          signer: user.publicKey,
          vault: vaultPda,
          vaultState: vaultStatePda,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      // Check balances after deposit
      const finalVaultBalance = await provider.connection.getBalance(vaultPda);
      const finalSignerBalance = await provider.connection.getBalance(user.publicKey);

      expect(finalVaultBalance).to.equal(initialVaultBalance + depositAmount);
      expect(finalSignerBalance).to.be.lessThan(initialSignerBalance); // Less due to deposit + transaction fee

      console.log("Deposit test passed ✅");
      console.log("Deposit transaction signature:", tx);
      console.log("vault balance:", finalVaultBalance);
      console.log("Signer balance:", finalSignerBalance);

    } catch (error) {
      console.error("Deposit test failed ❌, error:", error);
      throw error;
    }
  });

  it("Withdraw lamports from the vault", async () => {
   const withdrawAmount = 0.25 * LAMPORTS_PER_SOL;

    // Get initial balances
    const initialVaultBalance = await provider.connection.getBalance(vaultPda);
    const initialSignerBalance = await provider.connection.getBalance(user.publicKey);

   try {
    const tx = await program.methods
    .withdraw(new anchor.BN(withdrawAmount))
    .accountsPartial({
        signer: user.publicKey,
        vault: vaultPda,
        vaultState: vaultStatePda,
        systemProgram: SystemProgram.programId,
    })
    .rpc();

          // Check balances after withdrawal
      const finalVaultBalance = await provider.connection.getBalance(vaultPda);
      const finalSignerBalance = await provider.connection.getBalance(user.publicKey);

      expect(finalVaultBalance).to.equal(initialVaultBalance - withdrawAmount);
      expect(finalSignerBalance).to.be.greaterThan(initialSignerBalance); // More due to withdrawal minus transaction fee

      console.log("Withdraw test passed ✅");
      console.log("Withdraw transaction signature:", tx);
      console.log("vault balance:", finalVaultBalance);
      console.log("Signer balance:", finalSignerBalance);
    
   } catch (error) {
      console.error("Withdraw test failed ❌, error:", error);
      throw error;
   }
  });

  it("Fails to withdraw more than available (rent-exempt check)", async () => {
    try {
      const vaultBalance = await provider.connection.getBalance(vaultPda);
      const excessiveAmount = vaultBalance; // Try to withdraw everything (should fail due to rent-exempt requirement)

      await program.methods
        .withdraw(new anchor.BN(excessiveAmount))
        .accountsPartial({
          signer: user.publicKey,
          vault: vaultPda,
          vaultState: vaultStatePda,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      // If we reach here, the test should fail
      expect.fail("Expected withdrawal to fail due to rent-exempt violation");
    } catch (error) {
      // This should fail - check that it's the right error
      expect(error.message).to.include("ViolateRentExemption");
    }
  });

  it("Fails to withdraw with insufficient funds", async () => {
    try {
      const excessiveAmount = 10 * LAMPORTS_PER_SOL; // Way more than what's in the vault

      await program.methods
        .withdraw(new anchor.BN(excessiveAmount))
        .accountsPartial({
          signer: user.publicKey,
          vault: vaultPda,
          vaultState: vaultStatePda,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      // If we reach here, the test should fail
      expect.fail("Expected withdrawal to fail due to insufficient funds");
    } catch (error) {
      // This should fail - check that it's the right error
      expect(error.message).to.include("InsufficientFunds");
    }
  });

  it("Closes the vault and returns all funds", async () => {
    // Get initial signer balance
    const initialSignerBalance = await provider.connection.getBalance(user.publicKey);
    const vaultBalance = await provider.connection.getBalance(vaultPda);

    try {
      const tx = await program.methods
        .close()
        .accountsPartial({
          signer: user.publicKey,
          vault: vaultPda,
          vaultState: vaultStatePda,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      // Check that the vault account is closed (should have 0 lamports)
      const finalVaultBalance = await provider.connection.getBalance(vaultPda);
      expect(finalVaultBalance).to.equal(0);

      // Check that the signer received the funds
      const finalSignerBalance = await provider.connection.getBalance(user.publicKey);
      expect(finalSignerBalance).to.be.greaterThan(initialSignerBalance);

      // Check that vault state account is closed
      try {
        await program.account.vaultState.fetch(vaultStatePda);
        expect.fail("Vault state should be closed");
      } catch (error) {
        // This is expected - the account should be closed
        expect(error.message).to.include("Account does not exist");
      }

      console.log("Close test passed ✅");
      console.log("Close transaction signature:", tx);
      console.log("vault balance:", finalVaultBalance);
      console.log("Signer balance:", finalSignerBalance);


    } catch (error) {
      console.error("Close test failed ❌, error:", error);
      throw error;
    }
  });

  it("Fails to initialize vault twice with same signer", async () => {
    try {
      // Try to initialize again with the same signer
      await program.methods
        .initialize()
        .accountsPartial({
          signer: user.publicKey,
          vault: vaultPda,
          vaultState: vaultStatePda,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      expect.fail("Expected initialization to fail for existing vault");
    } catch (error) {
      // This should fail because the account already exists
      expect(error.message).to.include("already in use");
    }
  });


})
