import { createSolanaClient, createTransaction, generateKeyPairSigner, getExplorerLink, getMinimumBalanceForRentExemption, getSignatureFromTransaction, KeyPairSigner, signTransactionMessageWithSigners } from "gill";
import { loadKeypairSignerFromFile } from "gill/node";
import { getInitializeMintInstruction, getMintSize, TOKEN_PROGRAM_ADDRESS } from "gill/programs/token";
import {
  getCreateAccountInstruction,
  getCreateMetadataAccountV3Instruction,
  getTokenMetadataAddress,
} from "gill/programs";

// create a connection
const { rpc, sendAndConfirmTransaction } = createSolanaClient({
  urlOrMoniker: "devnet", 
});

(async () => {
    // create a signer
    const signer: KeyPairSigner = await loadKeypairSignerFromFile("./wallet.json");
    console.log("signer:", signer.address);

    // get latestblockhash
    const {value: latestBlockhash } = await rpc.getLatestBlockhash().send();

    // create mint
    const mint = await generateKeyPairSigner();
    console.log("mint:", mint.address);

    const space = getMintSize();

    // create metadata address
    const metadata =  await getTokenMetadataAddress(mint);


    // create transaction
    const tx = createTransaction({
        feePayer: signer,
        version: "legacy",
        instructions: [
            getCreateAccountInstruction({
                space,
                lamports: getMinimumBalanceForRentExemption(space),
                newAccount: mint,
                payer: signer,
                programAddress: TOKEN_PROGRAM_ADDRESS,
            }),
            getInitializeMintInstruction({
                mint: mint.address,
                mintAuthority: signer.address,
                freezeAuthority: signer.address,
                decimals: 6,
            },
            {
                programAddress: TOKEN_PROGRAM_ADDRESS,
            }
        ),
        getCreateMetadataAccountV3Instruction({
            collectionDetails: null,
            isMutable: true,
            updateAuthority: signer,
            mint: mint.address,
            metadata,
            mintAuthority: signer,
            payer: signer,
            data: {
                sellerFeeBasisPoints: 0,
                collection: null,
                creators: null,
                uses: null,
                name: "Gill Practice",
                symbol: "GPT",
                uri: "https://res.cloudinary.com/dpr7stzp5/image/upload/v1751445694/Vhagar_xppfpn.webp",
            },
        })
        ],
        latestBlockhash
    })

    const signedTransaction = await signTransactionMessageWithSigners(tx);

     console.log( "Explorer:",getExplorerLink({
        cluster: "devnet",
        transaction: getSignatureFromTransaction(signedTransaction),
    }),)

    await sendAndConfirmTransaction(signedTransaction)
})()