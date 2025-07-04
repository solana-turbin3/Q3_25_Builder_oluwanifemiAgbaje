import wallet from "../wallet.json"
import { createUmi } from "@metaplex-foundation/umi-bundle-defaults"
import { createGenericFile, createSignerFromKeypair, signerIdentity } from "@metaplex-foundation/umi"
import { irysUploader } from "@metaplex-foundation/umi-uploader-irys"
import { readFile } from "fs/promises"

// Create a devnet connection
const umi = createUmi('https://api.devnet.solana.com'); 
// solana devnet RPC URL down
// const umi = createUmi('https://solana-devnet.api.syndica.io/api-key/3nwfWkMbDcGpUrAFQYofpGgaKUK8LJAb3pZrvfmxm5UEZwsu7BYrkHytmK5BYttGtPh9cVw9EH665TstXvc6VoAMLdo8tpKsNVn');

let keypair = umi.eddsa.createKeypairFromSecretKey(new Uint8Array(wallet));
const signer = createSignerFromKeypair(umi, keypair);

umi.use(irysUploader({address: "https://devnet.irys.xyz/",}));
umi.use(signerIdentity(signer)); 

(async () => {
    try {
        //1. Load image
        const imageBuff = await readFile("./mikasa.png");

        //2. Convert image to generic file.
        const imgfile = await createGenericFile(imageBuff, 'mikasa.png', {contentType: 'image/png'});

        //3. Upload image
        const [myUri] = await umi.uploader.upload([imgfile]);
        console.log("Your image URI: ", myUri);
    }
    catch(error) {
        console.log("Oops.. Something went wrong", error);
    }
})();
