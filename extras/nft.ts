import Arweave from "arweave";
import * as bip39 from "bip39";
import { Metaplex, keypairIdentity, toBigNumber, isBigNumber, toOptionBigNumber, bundlrStorage } from "@metaplex-foundation/js";
import { BN } from "bn.js";
import fs from 'mz/fs';
import os from 'os';
import path from 'path';
import {
    LAMPORTS_PER_SOL,
    sendAndConfirmTransaction,
    PublicKey,
    Connection,
    Keypair,
    Transaction,
    TransactionInstruction,
    SYSVAR_RENT_PUBKEY,
    SystemProgram,
    Struct,
    SYSVAR_CLOCK_PUBKEY
} from "@solana/web3.js";
const createAccount = async(connection:Connection) : Promise<Keypair> => {
    const key = Keypair.generate();
    const airdrop = await connection.requestAirdrop(key.publicKey,2*LAMPORTS_PER_SOL);
    await connection.confirmTransaction(airdrop)
    return key;
}

const createKeypairFromFile = async(path:string): Promise<Keypair> => {
    const secret_keypair = await fs.readFile(path,{encoding:"utf8"});
    const secret_key = Uint8Array.from(JSON.parse(secret_keypair));
    const programKeypair = Keypair.fromSecretKey(secret_key);
    return programKeypair;
}
const localenet = "http://127.0.0.1:8899";
const connection = new Connection(localenet);    
console.log("Pinging ... !");
const alice = await createAccount(connection);
const WALLET_PATH =  path.join(
    path.resolve(__dirname,"../"),"wallet.json"
);
const wallet  = JSON.parse(await fs.readFile(WALLET_PATH,{encoding:"utf8"}));
const NFT_URL = "https://arweave.net/I8sKri2fT3n2b559aSBS_kB0vNWZxWz5p1U3-hq8d8c";
const METADATA_ID = "hjZYxakJx-QWgWArcbEpgypKjwRAGJl4s42wCAalu0U";
const getNftMetadata = (imageUrl:string) =>{
    return {
        name: "Snowman",
        symbol: "Snow",
        description: "Snowman",
        seller_fee_basis_points: 0,
        external_url: "https://www.customnft.com/",
        attributes: [
          {
            trait_type: "NFT type",
            value: "Custom",
          },
        ],
        collection: {
          name: "Test Collection",
          family: "Custom NFTs",
        },
        properties: {
          files: [
            {
              uri: imageUrl,
              type: "image/png",
            },
          ],
          category: "image",
          maxSupply: 0,
          creators: [
            {
              address: "ErmPFzSPpa4UDt755HPUqR2yQmbckHcDZX5f4NRuNPxe",
              share: 100,
            },
          ],
        },
        image: imageUrl,
      };
}
console.log("Initialize arweave !");
    const arweave = Arweave.init({
        host: "arweave.net",
        port: 443,
        protocol: "https",
        timeout: 20000,
        logging: false,
    });
    const NFT_PATH = path.join(
        path.resolve(__dirname,"../Images/"),"Snowman.png"
    );
    console.log(NFT_PATH)
    const nftdata = fs.readFileSync(NFT_PATH);
    const arweaveTx = await arweave.createTransaction({data:nftdata},wallet);
    arweaveTx.addTag("Content-Type", "image/png");

    console.log("Creating NFT mint");
    await arweave.transactions.sign(arweaveTx,wallet);
    const response = await arweave.transactions.post(arweaveTx);
    console.log("arweave tx",response);
    
    const nftTxId = arweaveTx.id;
    const imageUrl = nftTxId ? `https://arweave.net/${nftTxId}` : undefined;
    console.log("imageUrl", imageUrl);
    if(!imageUrl){
        console.log("Image cannot be posted on arweave !");
        process.exit(-1);
    }
    const metadata = getNftMetadata(imageUrl);
    const metadataRequest = JSON.stringify(metadata);
    const metadataTransaction = await arweave.createTransaction({
      data: metadataRequest,
    });
    metadataTransaction.addTag("Content-Type", "application/json");
    await arweave.transactions.sign(metadataTransaction, wallet);
    console.log("metadata txid", metadataTransaction.id);
    console.log(await arweave.transactions.post(metadataTransaction));
    const metaplex = Metaplex.make(connection)
    .use(keypairIdentity(alice))
    .use(bundlrStorage());
    const mintNFTResponse = await metaplex.nfts().create({
        uri: NFT_URL,
        payer:alice,
        updateAuthority:alice,
        maxSupply: toBigNumber(1),
        name:"Snowman",
        sellerFeeBasisPoints:500
    }).run();
    console.log(mintNFTResponse);