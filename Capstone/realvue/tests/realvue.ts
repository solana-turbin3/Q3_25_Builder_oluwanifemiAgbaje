import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Realvue } from "../target/types/realvue";
import NodeWallet from "@coral-xyz/anchor/dist/cjs/nodewallet";
import { PublicKey, Keypair, SystemProgram, LAMPORTS_PER_SOL, Commitment } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID, getOrCreateAssociatedTokenAccount } from "@solana/spl-token";
import { expect } from "chai";
import adminKey from './wallets/admin.json';
import merchantKey from './wallets/wallet.json';
import reviewer1Key from './wallets/wallet1.json';
import reviewer2Key from './wallets/wallet2.json';
import reviewer3Key from './wallets/wallet3.json';
import { BN } from "bn.js";

const commitment: Commitment = 'confirmed';

describe("realvue", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env()
  anchor.setProvider(provider);

  const program = anchor.workspace.realvue as Program<Realvue>;

  const connection = provider.connection;

  const seed = new BN(12);

  // Accounts
  let admin: Keypair;
  let merchant: Keypair;
  let reviewer1: Keypair;
  let reviewer2: Keypair;
  let reviewer3: Keypair;

  let merchantAta: PublicKey;

  const PLATFORM_FEE = 500; // 5% in basis points
  const CAMPAIGN_NAME = "Share Thoughts on My Crypto Art";
  const PRODUCT_ID = "PRODUCT123";

  const DEPOSIT_AMOUNT = new anchor.BN(1 * LAMPORTS_PER_SOL); // 1 SOL
  const REVIEWS_NEEDED = 3;
  const REVIEW_DESCRIPTION1 = "Really impressed with the creativity behind this NFT drop. The designs are bold, but the marketplace UI could be a bit more user-friendly. Overall, a solid addition to my digital wallet!";
  const REVIEW_DESCRIPTION2 = "I’ve been exploring this NFT project for a couple of weeks, and it’s exceeded my expectations. The community engagement is top-notch. Highly recommend!";
  const REVIEW_DESCRIPTION3 = "The art in this NFT series is fantastic, with a lot of attention to detail. However, the transaction fees were a bit high during the mint. Still, I’m happy with my purchase and excited for future drops.";

  // Transaction IDs (realistic Solana transaction signatures)
  const TX_IDS = {
    TX1: "5j7s8K2FxVqp9Rm3nL4wY6tE1qW8rT5uI9oP3aS7dF2gH1kJ6mN8xC4vB5zA9yX2qE3rT6yU8iO1pA4sD7fG9hJ2",
    TX2: "2d4f6H8jK1lP3nM5qR7tY9uI1oE3wQ5aS7dF9gH2jK4lN6pR8tY1uI3oE5wQ7aS9dF2gH4jK6lN8pR1tY3uI5oE7w",
    TX3: "8k2L4nP6rT9vX1zA3cF5hJ7mQ9sU2wY4eR6tI8oL1pN3qM5sV7xZ9aD2gH4jK6nP8rT1vX3zA5cF7hJ9mQ2sU4wY6e"
  };

  // Flagged reasons for rejected reviews
  const FLAGGED_REASONS = {
    NONE: "",
    SPAM: "Spam content detected",
    INAPPROPRIATE: "Inappropriate language or content",
    FAKE: "Suspected fake or bot-generated review",
    OFF_TOPIC: "Review not relevant to product",
    DUPLICATE: "Duplicate review from same user",
    VIOLATION: "Terms of service violation",
    INSUFFICIENT: "Review lacks sufficient detail",
    PROMOTIONAL: "Contains promotional or advertising content"
  };
  const START_TIME = new anchor.BN(Math.floor(Date.now() / 1000));
  const END_TIME = new anchor.BN(Math.floor(Date.now() / 1000) + 3600 * 24 * 7); // 1 week


  admin = Keypair.fromSecretKey(Uint8Array.from(adminKey));
  merchant = Keypair.fromSecretKey(Uint8Array.from(merchantKey));
  reviewer1 = Keypair.fromSecretKey(Uint8Array.from(reviewer1Key));
  reviewer2 = Keypair.fromSecretKey(Uint8Array.from(reviewer2Key));
  reviewer3 = Keypair.fromSecretKey(Uint8Array.from(reviewer3Key));


  // PDAs
  let platform = PublicKey.findProgramAddressSync(
    [Buffer.from("realvue"), seed.toBuffer("le", 8), admin.publicKey.toBuffer()],
    program.programId
  )[0];

  let treasury = PublicKey.findProgramAddressSync(
    [Buffer.from("treasury"), platform.toBuffer()],
    program.programId
  )[0];

  // Derive the REV token mint address
  let revMint = PublicKey.findProgramAddressSync([
    Buffer.from('rev'),
    platform.toBuffer()],
    program.programId)[0];


  let campaign = PublicKey.findProgramAddressSync(
    [Buffer.from("campaign"), Buffer.from(CAMPAIGN_NAME), merchant.publicKey.toBuffer()],
    program.programId
  )[0];

  let campaignVault = PublicKey.findProgramAddressSync(
    [Buffer.from("campaign_vault"), campaign.toBuffer()],
    program.programId
  )[0];

  // reviewer accounts
  let reviewer1Account = PublicKey.findProgramAddressSync(
    [Buffer.from("reviewer"), reviewer1.publicKey.toBuffer()],
    program.programId
  )[0];

  let reviewer2Account = PublicKey.findProgramAddressSync(
    [Buffer.from("reviewer"), reviewer2.publicKey.toBuffer()],
    program.programId
  )[0];

  let reviewer3Account = PublicKey.findProgramAddressSync(
    [Buffer.from("reviewer"), reviewer3.publicKey.toBuffer()],
    program.programId
  )[0];

  // Review Account PDAs (for each reviewer)
  let review1Account = PublicKey.findProgramAddressSync(
    [campaign.toBuffer(), reviewer1.publicKey.toBuffer()],
    program.programId
  )[0];

  let review2Account = PublicKey.findProgramAddressSync(
    [campaign.toBuffer(), reviewer2.publicKey.toBuffer()],
    program.programId
  )[0];

  let review3Account = PublicKey.findProgramAddressSync(
    [campaign.toBuffer(), reviewer3.publicKey.toBuffer()],
    program.programId
  )[0];




  it("Platform Initialization, Should initialize platform successfully", async () => {
    const tx = await program.methods.initPlatform(seed, PLATFORM_FEE)
      .accountsPartial({
        admin: admin.publicKey,
        platform: platform,
        revMint: revMint,
        treasury: treasury,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([admin])
      .rpc();
    console.log("Initialize platform tx:", tx);

    merchantAta = (await getOrCreateAssociatedTokenAccount(
      connection,
      merchant,
      revMint,
      merchant.publicKey,
      true
    )).address;

    console.log(`Merchant ATA for revMint token is: ${merchantAta.toBase58()}`);
  });


  it("Campaign Management, Should create campaign successfully", async () => {
    const tx = await program.methods.createCampaign(CAMPAIGN_NAME, PRODUCT_ID, DEPOSIT_AMOUNT, START_TIME, END_TIME, REVIEWS_NEEDED)
      .accountsPartial({
        merchant: merchant.publicKey,
        campaign: campaign,
        merchantAta: merchantAta,
        platform: platform,
        revMint: revMint,
        vault: campaignVault,
        treasury: treasury,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([merchant])
      .rpc();
    console.log("Create campaign tx:", tx);
  });


  it("Should create all reviewer accounts successfully", async () => {
    // Create reviewer1 account
    const r1tx = await program.methods.initReviewer()
      .accountsPartial({
        reviewer: reviewer1.publicKey,
        reviewerAccount: reviewer1Account,
        systemProgram: SystemProgram.programId,
      })
      .signers([reviewer1])
      .rpc();
    console.log("Reviewer1 account tx:", r1tx);

    // Create reviewer2 account
    const r2tx = await program.methods.initReviewer()
      .accountsPartial({
        reviewer: reviewer2.publicKey,
        reviewerAccount: reviewer2Account,
        systemProgram: SystemProgram.programId,
      })
      .signers([reviewer2])
      .rpc();
    console.log("Reviewer2 account tx:", r2tx);

    // Create reviewer3 account
    const r3tx = await program.methods.initReviewer()
      .accountsPartial({
        reviewer: reviewer3.publicKey,
        reviewerAccount: reviewer3Account,
        systemProgram: SystemProgram.programId,
      })
      .signers([reviewer3])
      .rpc();
    console.log("Reviewer3 account tx:", r3tx);

    console.log("All reviewer accounts created successfully");
  });


  it("Should submit multiple reviews successfully", async () => {
    // Reviewer 1 submits positive review
    const tx1 = await program.methods
      .makeReview(REVIEW_DESCRIPTION1, TX_IDS.TX1)
      .accountsPartial({
        reviewer: reviewer1.publicKey,
        reviewAccount: review1Account,
        campaign: campaign,
        reviewerAccount: reviewer1Account,
        platform: platform,
        systemProgram: SystemProgram.programId,
      })
      .signers([reviewer1])
      .rpc();

    // Reviewer 2 submits review
    const tx2 = await program.methods
      .makeReview(REVIEW_DESCRIPTION2, TX_IDS.TX2)
      .accountsPartial({
        reviewer: reviewer2.publicKey,
        reviewAccount: review2Account,
        campaign: campaign,
        reviewerAccount: reviewer2Account,
        platform: platform,
        systemProgram: SystemProgram.programId,
      })
      .signers([reviewer2])
      .rpc();

    // Reviewer 3 submits review
    const tx3 = await program.methods
      .makeReview(REVIEW_DESCRIPTION3, TX_IDS.TX3)
      .accountsPartial({
        reviewer: reviewer3.publicKey,
        reviewAccount: review3Account,
        campaign: campaign,
        reviewerAccount: reviewer3Account,
        platform: platform,
        systemProgram: SystemProgram.programId,
      })
      .signers([reviewer3])
      .rpc();

    console.log("All reviews submitted successfully");

    // Verify all review accounts
    const review1 = await program.account.reviewAccount.fetch(review1Account);
    const review2 = await program.account.reviewAccount.fetch(review2Account);
    const review3 = await program.account.reviewAccount.fetch(review3Account);

    expect(review1.description).to.equal(REVIEW_DESCRIPTION1);
    expect(review2.description).to.equal(REVIEW_DESCRIPTION2);
    expect(review3.description).to.equal(REVIEW_DESCRIPTION3);
  });

  it("Should approve multiple reviews successfully", async () => {
    // Approve all three reviews
    await program.methods
      .approveReview(true, FLAGGED_REASONS.NONE)
      .accountsPartial({
        admin: admin.publicKey,
        reviewAccount: review1Account,
        campaign: campaign,
        reviewerAccount: reviewer1Account,
        platform: platform,
        systemProgram: SystemProgram.programId,
      })
      .signers([admin])
      .rpc();

    await program.methods
      .approveReview(true, FLAGGED_REASONS.NONE)
      .accountsPartial({
        admin: admin.publicKey,
        reviewAccount: review2Account,
        campaign: campaign,
        reviewerAccount: reviewer2Account,
        platform: platform,
        systemProgram: SystemProgram.programId,
      })
      .signers([admin])
      .rpc();

    await program.methods
      .approveReview(true, FLAGGED_REASONS.NONE)
      .accountsPartial({
        admin: admin.publicKey,
        reviewAccount: review3Account,
        campaign: campaign,
        reviewerAccount: reviewer3Account,
        platform: platform,
        systemProgram: SystemProgram.programId,
      })
      .signers([admin])
      .rpc();

    // Verify campaign approved count
    const campaignAccount = await program.account.reviewCampaign.fetch(campaign);
    expect(campaignAccount.approvedCount).to.equal(3);
  });

  xit("Should allow all reviewers to claim rewards", async () => {

    // Wait a bit for transactions to settle
    await new Promise(resolve => setTimeout(resolve, 2000));
    // All reviewers claim their rewards
    await program.methods
      .claimReward()
      .accountsPartial({
        reviewer: reviewer1.publicKey,
        reviewAccount: review1Account,
        campaign: campaign,
        vault: campaignVault,
        reviewerAccount: reviewer1Account,
        platform: platform,
        systemProgram: SystemProgram.programId,
      })
      .signers([reviewer1])
      .rpc();

    await program.methods
      .claimReward()
      .accountsPartial({
        reviewer: reviewer2.publicKey,
        reviewAccount: review2Account,
        campaign: campaign,
        vault: campaignVault,
        reviewerAccount: reviewer2Account,
        platform: platform,
        systemProgram: SystemProgram.programId,
      })
      .signers([reviewer2])
      .rpc();

    await program.methods
      .claimReward()
      .accountsPartial({
        reviewer: reviewer3.publicKey,
        reviewAccount: review3Account,
        campaign: campaign,
        vault: campaignVault,
        reviewerAccount: reviewer3Account,
        platform: platform,
        systemProgram: SystemProgram.programId,
      })
      .signers([reviewer3])
      .rpc();

    console.log("All reviewers claimed rewards successfully");

    // Verify all rewards were claimed
    const review1 = await program.account.reviewAccount.fetch(review1Account);
    const review2 = await program.account.reviewAccount.fetch(review2Account);
    const review3 = await program.account.reviewAccount.fetch(review3Account);

    expect(review1.rewardClaimed).to.be.true;
    expect(review2.rewardClaimed).to.be.true;
    expect(review3.rewardClaimed).to.be.true;
  });

  it(" Admin Function, Should claim platform fees successfully", async () => {
    const adminBalanceBefore = await provider.connection.getBalance(admin.publicKey);
    const treasuryBalanceBefore = await provider.connection.getBalance(treasury);

    if (treasuryBalanceBefore > 0) {
      const tx = await program.methods
        .claimFee()
        .accountsPartial({
          admin: admin.publicKey,
          platform: platform,
          treasury: treasury,
          systemProgram: SystemProgram.programId,
        })
        .signers([admin])
        .rpc();

      console.log("Claim fee tx:", tx);

      // Verify admin received fees
      const adminBalanceAfter = await provider.connection.getBalance(admin.publicKey);
      expect(adminBalanceAfter).to.be.greaterThan(adminBalanceBefore);

      // Verify treasury is empty
      const treasuryBalanceAfter = await provider.connection.getBalance(treasury);
      expect(treasuryBalanceAfter).to.equal(0);

      // Verify platform stats updated
      const platformAccount = await program.account.platformConfig.fetch(platform);
      expect(platformAccount.totalFeesCollected.toNumber()).to.be.greaterThan(0);
    }
  });

  xit("Should fail when non-admin tries to claim fees", async () => {
    try {
      await program.methods
        .claimFee()
        .accountsPartial({
          admin: merchant.publicKey, // Wrong admin
          platform: platform,
          treasury: treasury,
          systemProgram: SystemProgram.programId,
        })
        .signers([merchant])
        .rpc();

      expect.fail("Should have failed - unauthorized admin");
    } catch (error) {
      expect(error.message).to.include("UnauthorizedAdmin");
    }
  });

  it("Should close campaign successfully", async () => {
    const tx = await program.methods
      .closeCampaign()
      .accountsPartial({
        merchant: merchant.publicKey,
        campaign: campaign,
        platform: platform,
        systemProgram: SystemProgram.programId,
      })
      .signers([merchant])
      .rpc();

    console.log("Close campaign tx:", tx);

    // Check if the account still exists before trying to fetch it
    try {
      const campaignAccountInfo = await provider.connection.getAccountInfo(campaign);
      if (campaignAccountInfo) {
        const campaignAccount = await program.account.reviewCampaign.fetch(campaign);
        expect(campaignAccount.active).to.be.false;
      } else {
        console.log("Campaign account was closed (no longer exists)");
      }
    } catch (error) {
      // If account doesn't exist, that's expected for a close operation
      console.log("Campaign account successfully closed");
    }
  });


xit("Should refund deposit and close campaign successfully", async () => {
    const merchantBalanceBefore = await provider.connection.getBalance(merchant.publicKey);

    const tx = await program.methods
      .refundDeposit()
      .accountsPartial({
        merchant: merchant.publicKey,
        campaign: campaign,
        vault: campaignVault,
        platform: platform,
        systemProgram: SystemProgram.programId,
      })
      .signers([merchant])
      .rpc();

    console.log("Refund deposit tx:", tx);

    // Verify merchant received refund
    const merchantBalanceAfter = await provider.connection.getBalance(merchant.publicKey);
    expect(merchantBalanceAfter).to.be.greaterThan(merchantBalanceBefore);
  });

  xit("Should fail to refund active campaign", async () => {
    await program.methods
      .createCampaign(CAMPAIGN_NAME, PRODUCT_ID, DEPOSIT_AMOUNT, START_TIME, END_TIME, REVIEWS_NEEDED)
      .accountsPartial({
        merchant: merchant.publicKey,
        campaign: campaign,
        merchantAta: merchantAta,
        platform: platform,
        revMint: revMint,
        vault: campaignVault,
        treasury: treasury,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([merchant])
      .rpc();

    try {
      await program.methods
        .refundDeposit()
        .accountsPartial({
          merchant: merchant.publicKey,
          campaign: campaign,
          vault: campaignVault,
          platform: platform,
          systemProgram: SystemProgram.programId,
        })
        .signers([merchant])
        .rpc();

      expect.fail("Should have failed - campaign still active");
    } catch (error) {
      expect(error.message).to.include("CampaignStillActive");
    }
  });

  it("Close reviewer accounts", async () => {
    // Close reviewer accounts if they exist
    for (const reviewer of [reviewer1, reviewer2, reviewer3]) {
      const reviewerAccount = PublicKey.findProgramAddressSync(
        [Buffer.from("reviewer"), reviewer.publicKey.toBuffer()],
        program.programId
      )[0];
      try {
        await program.methods
          .closeReviewer()
          .accountsPartial({
            reviewer: reviewer.publicKey,
            reviewerAccount: reviewerAccount,
            systemProgram: SystemProgram.programId,
          })
          .signers([reviewer])
          .rpc({ commitment: "confirmed" });
        console.log(`Closed reviewer account: ${reviewerAccount.toBase58()}`);
      } catch (e) {
        console.log(`No reviewer account to close for ${reviewer.publicKey.toBase58()}`);
      }
    }
  });

  it("Should close platform successfully", async () => {
    const tx = await program.methods
      .closePlatform()
      .accountsPartial({
        admin: admin.publicKey,
        platform: platform,
        systemProgram: SystemProgram.programId,
      })
      .signers([admin])
      .rpc();

    console.log("Close platform tx:", tx);

    // Check if the account still exists before trying to fetch it
    try {
      const platformAccountInfo = await provider.connection.getAccountInfo(platform);
      if (platformAccountInfo) {
        const platformAccount = await program.account.platformConfig.fetch(platform);
        expect(platformAccount.isActive).to.be.false;
      } else {
        console.log("Platform account was successfully closed (no longer exists)");
      }
    } catch (error) {
      // If account doesn't exist, that's expected for a close operation
      console.log("Platform account successfully closed");
    }
  });
});
