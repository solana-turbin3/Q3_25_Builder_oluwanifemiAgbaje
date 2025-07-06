import { log } from "console";
import {
  address,
  createSolanaClient,
  createTransaction,
  getExplorerLink,
  getSignatureFromTransaction,
  KeyPairSigner,
  signTransactionMessageWithSigners,
} from "gill";
import { loadKeypairSignerFromFile } from "gill/node";
import { getAssociatedTokenAccountAddress, getCreateAssociatedTokenIdempotentInstruction, getMintToInstruction, TOKEN_PROGRAM_ADDRESS } from "gill/programs/token";

const { rpc, sendAndConfirmTransaction } = createSolanaClient({
  urlOrMoniker: "devnet",
});

(async () => {    
const signer: KeyPairSigner = await loadKeypairSignerFromFile("./wallet.json");
console.log("signer:", signer.address);

const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();

const mint = address("4N94pRaDXiR1mzX3Mu3xx3AkfjBDR5uAawvWLDBFrgFV");

const owner = address("5FaER8yXfEujMT7Q7N7BZ8hDdMgh15Vow7LgQpAeiTTy");

const token_decimals = BigInt(1_000_000);

const ata = await getAssociatedTokenAccountAddress(mint, owner, TOKEN_PROGRAM_ADDRESS);
console.log("ata:", ata);


const tx = createTransaction({
  feePayer: signer,
  version: "legacy",
  instructions: [
    getCreateAssociatedTokenIdempotentInstruction({
      mint,
      owner,
      payer: signer,
      tokenProgram: TOKEN_PROGRAM_ADDRESS,
      ata,
    }),
    getMintToInstruction(
      {
        mint,
        mintAuthority: signer,
        token: ata,
        amount: BigInt(1000000) * token_decimals,
      },
      {
        programAddress: TOKEN_PROGRAM_ADDRESS,
      },
    ),
  ],
  latestBlockhash,
});

const signedTransaction = await signTransactionMessageWithSigners(tx);

console.log(
  "Explorer:",
  getExplorerLink({
    cluster: "devnet",
    transaction: getSignatureFromTransaction(signedTransaction),
  }),
);

await sendAndConfirmTransaction(signedTransaction);
})()
