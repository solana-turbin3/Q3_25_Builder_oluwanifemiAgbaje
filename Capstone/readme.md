# Realvue - Review to Earn Platform

A decentralized review-to-earn platform built on Solana where merchants can create campaigns to gather authentic reviews, and reviewers earn SOL rewards for their contributions.

## 🎯 Project Overview

Realvue is a Turbin3 capstone project that creates a trustless review ecosystem on the Solana blockchain. Merchants deposit SOL to create review campaigns, reviewers submit authentic reviews, and admin validates them before rewards are distributed. The platform uses non-transferable REV tokens to track merchant campaign participation and implements a comprehensive reward distribution system.

## ✨ Key Features

### For Merchants
- **Campaign Creation**: Create review campaigns by depositing SOL
- **Customizable Parameters**: Set campaign duration, required review count, and product details
- **REV Token Rewards**: Receive non-transferable REV tokens upon campaign creation
- **Refund System**: Get remaining SOL refunded when campaign closes
- **Campaign Management**: Monitor review progress and campaign status

### For Reviewers
- **Review Submission**: Submit detailed reviews with transaction IDs for verification
- **Reward Claims**: Earn SOL rewards for approved reviews
- **Ranking System**: Build reputation through approved review history
- **Account Management**: Initialize and manage reviewer profiles

### For Platform Admin
- **Review Validation**: Approve or reject submitted reviews with optional flagging reasons
- **Fee Collection**: Collect platform fees from campaign deposits
- **Platform Management**: Initialize and maintain platform configurations
- **Treasury Management**: Oversee platform treasury and fee distribution

## 🏗 Architecture

### Core Components

1. **Platform Configuration**
   - Admin-controlled platform settings
   - Fee structure management
   - REV token mint authority

2. **Campaign System**
   - Merchant-created review campaigns
   - SOL deposit and vault management
   - Time-based campaign lifecycle

3. **Review Management**
   - Reviewer submission system
   - Admin validation workflow
   - Reward distribution mechanism

4. **Token Economics**
   - Non-transferable REV tokens for merchants
   - SOL rewards for reviewers
   - Platform fee collection in treasury

### Account Structure

```
├── Platform Config
│   ├── Admin authority
│   ├── Fee configuration
│   └── Platform statistics
├── Campaign
│   ├── Merchant details
│   ├── Campaign parameters
│   └── Review tracking
├── Review Account
│   ├── Review content
│   ├── Approval status
│   └── Reward claim status
└── Reviewer Account
    ├── Review history
    ├── Reputation ranking
    └── Reward statistics
```

## 🛠 Technical Implementation

### Built With
- **Anchor Framework**: Solana program development
- **TypeScript**: Testing and client interaction
- **SPL Token Program**: REV token implementation
- **Solana Web3.js**: Blockchain interaction

### Key Program Instructions

| Instruction | Description | Authority |
|-------------|-------------|-----------|
| `init_platform` | Initialize platform with admin and fee settings | Admin |
| `create_campaign` | Create review campaign with SOL deposit | Merchant |
| `init_reviewer` | Initialize reviewer account | Reviewer |
| `make_review` | Submit review for campaign | Reviewer |
| `approve_review` | Validate and approve/reject reviews | Admin |
| `claim_reward` | Claim SOL rewards for approved reviews | Reviewer |
| `claim_fee` | Withdraw platform fees from treasury | Admin |
| `close_campaign` | Close campaign and update status | Merchant |
| `refund_deposit` | Refund remaining campaign deposit | Merchant |

## 🚀 Getting Started

### Prerequisites

```bash
# Install Solana CLI
sh -c "$(curl -sSfL https://release.solana.com/v1.18.4/install)"

# Install Anchor CLI
cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
avm install latest
avm use latest

# Install Node.js dependencies
npm install
```

### Local Development

1. **Start Solana Test Validator**
```bash
solana-test-validator
```

2. **Configure Solana CLI**
```bash
solana config set --url localhost
solana config set --keypair ~/.config/solana/id.json
```

3. **Build the Program**
```bash
anchor build
```

4. **Deploy to Localnet**
```bash
anchor deploy
```

5. **Run Tests**
```bash
anchor test --skip-local-validator
```

### Environment Setup

Create test wallets for different roles:

```bash
# Generate test keypairs
solana-keygen new --outfile ./tests/wallets/admin.json
solana-keygen new --outfile ./tests/wallets/wallet.json
solana-keygen new --outfile ./tests/wallets/wallet1.json
solana-keygen new --outfile ./tests/wallets/wallet2.json
solana-keygen new --outfile ./tests/wallets/wallet3.json

# Airdrop SOL to test accounts
solana airdrop 10 <ADMIN_PUBKEY>
solana airdrop 10 <MERCHANT_PUBKEY>
solana airdrop 5 <REVIEWER1_PUBKEY>
solana airdrop 5 <REVIEWER2_PUBKEY>
solana airdrop 5 <REVIEWER3_PUBKEY>
```

## 📊 Platform Economics

### Fee Structure
- **Platform Fee**: 5% (500 basis points) of campaign deposits
- **Reward Distribution**: Remaining SOL split among approved reviews
- **REV Tokens**: Non-transferable, minted to merchant ATA

### Campaign Lifecycle
1. **Creation**: Merchant deposits SOL, receives REV tokens
2. **Review Phase**: Reviewers submit reviews with transaction IDs
3. **Validation**: Admin approves/rejects reviews
4. **Reward Phase**: Approved reviewers claim SOL rewards
5. **Closure**: Campaign closes, remaining SOL refunded to merchant

## 🧪 Testing

The test suite covers comprehensive scenarios:

### Core Functionality Tests
- ✅ Platform initialization
- ✅ Campaign creation and management
- ✅ Reviewer account setup
- ✅ Review submission and validation
- ✅ Reward distribution
- ✅ Admin fee collection
- ✅ Campaign closure and refunds

### Test Data Examples
```typescript
const CAMPAIGN_NAME = "Share Thoughts on My Crypto Art";
const PRODUCT_ID = "PRODUCT123";
const DEPOSIT_AMOUNT = new anchor.BN(1 * LAMPORTS_PER_SOL);
const REVIEWS_NEEDED = 3;
const PLATFORM_FEE = 500; // 5%
```

Run specific test suites:
```bash
# Run all tests
npm test

# Run with detailed output
anchor test --skip-local-validator -- --reporter spec
```

## 📁 Project Structure

```
realvue/
├── programs/
│   └── realvue/
│       ├── src/
│       │   ├── lib.rs              # Main program entry
│       │   ├── instructions/       # Program instructions
│       │   ├── state/             # Account structures
│       │   └── errors.rs          # Custom error definitions
├── tests/
│   ├── realvue.ts                 # Comprehensive test suite
│   └── wallets/                   # Test keypairs
├── target/
│   └── types/                     # Generated TypeScript types
├── migrations/
├── Anchor.toml                    # Anchor configuration
└── package.json                   # Node.js dependencies
```

## 🔐 Security Features

- **Authority Checks**: Role-based access control for all instructions
- **Campaign Validation**: Prevent self-reviewing and duplicate submissions
- **Transaction Verification**: TX ID validation for authentic reviews
- **Time-based Controls**: Campaign duration enforcement
- **Reward Protection**: Claim once per approved review
- **Treasury Security**: Admin-only fee withdrawal

## 🚧 Future Enhancements

- [ ] **Mobile Application**: React Native app for mobile users
- [ ] **Advanced Analytics**: Dashboard for merchants and reviewers
- [ ] **Multi-token Support**: Support for other SPL tokens
- [ ] **Reputation NFTs**: NFT badges for top reviewers
- [ ] **Campaign Templates**: Pre-built campaign configurations
- [ ] **Integration APIs**: Third-party platform integration

## 🤝 Contributing

This is a capstone project for Turbin3, feedback and suggestions are welcome!

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Submit a pull request


## 🙏 Acknowledgments

- **Turbin3**: For providing an excellent Solana development program

## 📞 Contact

For questions about this capstone project:
- **GitHub**: https://github.com/Nifix001
- **Twitter**: https://x.com/Aytolu7
- **Discord**: nifix01

---

*Built with ❤️ for the Turbin3 Capstone Project*

**Program ID**: CB9cLPfpZM2Dkjrep4LhiNXCpFa5iXhU3Jjr7TDFR8XF
**Network**: Solana Devnet
**Framework**: Anchor 