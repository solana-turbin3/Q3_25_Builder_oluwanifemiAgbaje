import { address, createSolanaClient, createTransaction, getExplorerLink, getSignatureFromTransaction, KeyPairSigner, signTransactionMessageWithSigners } from "gill";
import { loadKeypairSignerFromFile } from "gill/node";
import { getAssociatedTokenAccountAddress, getCreateAssociatedTokenIdempotentInstruction, getTransferInstruction, TOKEN_PROGRAM_ADDRESS } from "gill/programs";

const { rpc, sendAndConfirmTransaction } = createSolanaClient({
  urlOrMoniker: "devnet",
});

( async () => {
    const signer: KeyPairSigner = await loadKeypairSignerFromFile("./wallet.json");

    const {value: latestBlockhash} = await rpc.getLatestBlockhash().send();

    const mint = address("4N94pRaDXiR1mzX3Mu3xx3AkfjBDR5uAawvWLDBFrgFV");

    const token_program = TOKEN_PROGRAM_ADDRESS;
    const token_decimals = BigInt(1_000_000);

    const to = address("9hAMtLfZojhD5efNHimyhXQrb5p13K7Z6Vrfnp6PrQUv");
    const toAta = await getAssociatedTokenAccountAddress(mint, to, token_program);
    const sourceAta = await getAssociatedTokenAccountAddress(mint, signer, token_program);
    console.log("toAta:", toAta);
    console.log("sourceAta:", sourceAta);    

    const tx = createTransaction({
        feePayer: signer,
        version: "legacy",
        instructions: [
            getCreateAssociatedTokenIdempotentInstruction({
                mint,
                payer: signer,
                tokenProgram: token_program,
                owner: to,
                ata: toAta
            }),
            getTransferInstruction({
                source: sourceAta,
                destination: toAta,
                authority: signer,
                amount: BigInt(10000)* token_decimals,
            },
            {
                programAddress: token_program
            }
        )
        ],
        latestBlockhash
    })

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