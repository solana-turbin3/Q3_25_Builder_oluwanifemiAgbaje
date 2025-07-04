import { createUmi } from "@metaplex-foundation/umi-bundle-defaults"
import { createSignerFromKeypair, signerIdentity, generateSigner, percentAmount } from "@metaplex-foundation/umi"
import { createNft, mplTokenMetadata } from "@metaplex-foundation/mpl-token-metadata";

import wallet from "../wallet.json"
import base58 from "bs58";

const RPC_ENDPOINT = "https://api.devnet.solana.com";

const umi = createUmi(RPC_ENDPOINT);

let keypair = umi.eddsa.createKeypairFromSecretKey(new Uint8Array(wallet));
const myKeypairSigner = createSignerFromKeypair(umi, keypair);
umi.use(signerIdentity(myKeypairSigner));
umi.use(mplTokenMetadata())

const mint = generateSigner(umi);

const uri = "https://devnet.irys.xyz/8L87RHqs7n5JqAT2pjGisrZMzNBNw1eWLKqogqGDfQW3";

(async () => {
    let tx = createNft(umi, {
        mint,
        name: "Mikasa",
        symbol: "MKS", 
        uri, 
        sellerFeeBasisPoints: percentAmount(5), 
    });
    let result = await tx.sendAndConfirm(umi);
    const signature = base58.encode(result.signature);

    // metadataURI= https://arweave.net/8L87RHqs7n5JqAT2pjGisrZMzNBNw1eWLKqogqGDfQW3
    
    console.log(`Succesfully Minted! Check out your TX here:\nhttps://explorer.solana.com/tx/${signature}?cluster=devnet`)

    console.log("Mint Address: ", mint.publicKey);
})();