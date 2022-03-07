const {
  Connection,
  sendAndConfirmTransaction,
  Keypair,
  Transaction,
  SystemProgram,
  PublicKey,
  TransactionInstruction,
} = require("@solana/web3.js");
const {ASSOCIATED_TOKEN_PROGRAM_ID, Token, TOKEN_PROGRAM_ID, MintLayout} = require("@solana/spl-token");

const connectionURLs = {
  localhost: "http://127.0.0.1:8899",
  devnet: "https://api.devnet.solana.com/",
}

const BN = require("bn.js");

const main = async () => {
  var args = process.argv.slice(2);
  const programId = new PublicKey(args[0]);
  const echo = args[1];
  const price = parseInt(args[2]);

  const connection = new Connection(connectionURLs.devnet);


  // Init mint
  const payer = Keypair.generate();
  console.log("Requesting Airdrop of 2 SOL...");
  const airdropTx = await connection.requestAirdrop(payer.publicKey, 2e9);
  console.log(airdropTx);
  await connection.confirmTransaction(airdropTx, "finalized");
  console.log("Airdrop received");
  console.log("payer:", payer.publicKey.toBase58());
  
  // create token mint
  const vendingMachineMint = await Token.createMint(
    connection,
    payer,
    payer.publicKey,
    payer.publicKey,
    8,
    TOKEN_PROGRAM_ID,
  );

  // get token account
  const payer_token_account = await vendingMachineMint.getOrCreateAssociatedAccountInfo(
    payer.publicKey,
  );

  console.log("got here");
  // mint enough to pay
  await vendingMachineMint.mintTo(
    payer_token_account.address,
    payer.publicKey,
    [],
    1000000000,
  )

  // give some tokens
  // let mintTx = new Transaction().add(
  //   Token.createMintToInstruction(
  //     TOKEN_PROGRAM_ID, // always TOKEN_PROGRAM_ID
  //     vendingMachineMint.publicKey, // mint
  //     payer_token_account, // receiver (sholud be a token account)
  //     payer.publicKey, // mint authority
  //     [], // only multisig account will use. leave it empty now.
  //     100000 // amount. if your decimals is 8, you mint 10^8 for 1 token.
  //   )
  // );
  
  // console.log(`minted some tokens: txhash: ${await connection.sendTransaction(mintTx, [payer/* fee payer + mint authority */])}`);


  // PART 1

  // const vendingMachineMint = new Keypair(); // COME BACK
  // const vendingMachineMint = new PublicKey("ErhahGtwLBgKuUcFSca7TEnsJSXa7RH1TySzE2bcV5aP");
  const [ vendingMachineBuffer, bump ] = (await PublicKey.findProgramAddress(
    [Buffer.from("vending_machine"),
    vendingMachineMint.publicKey.toBuffer(),
    Buffer.from(new Uint8Array((new BN(price)).toArray("le", 8)))],
    programId
  ));


  let idx = Buffer.from(new Uint8Array([3]));
  // const buffer_seed_buf = Buffer.from(new Uint8Array((new BN(buffer_seed)).toArray("le", 4)))
  // const buffer_size = Buffer.from(new Uint8Array((new BN(echo.length)).toArray("le", 4)));
  const buffer_size = echo.length;
  const message = Buffer.from(echo, "ascii");
  const messageLen = Buffer.from(new Uint8Array((new BN(echo.length)).toArray("le", 4)));

  let initAuthIx = new TransactionInstruction({
    keys: [
      {
        pubkey: vendingMachineBuffer,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: vendingMachineMint.publicKey,
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: payer.publicKey,
        isSigner: true,
        isWritable: false,
      },
      {
        pubkey: SystemProgram.programId,
        isSigner: false,
        isWritable: false,
      },
    ],
    programId: programId,
    // data: Buffer.concat([idx, Buffer.from(new Uint8Array([2, 0, 0, 0])), Buffer.from(new Uint8Array([buffer_seed])), Buffer.from(new Uint8Array([buffer_size]))]),
    data: Buffer.concat([
      idx,
      Buffer.from(new Uint8Array((new BN(price)).toArray("le", 8))),
      Buffer.from(new Uint8Array((new BN(buffer_size)).toArray("le", 8)))
    ])
  });

  let tx = new Transaction();
  tx.add(initAuthIx);

  // let txid = await sendAndConfirmTransaction(
  //   connection,
  //   tx,
  //   [payer],
  //   {
  //     skipPreflight: true,
  //     preflightCommitment: "confirmed",
  //     confirmation: "confirmed",
  //   }
  // );

  // data = (await connection.getAccountInfo(vendingMachineBuffer)).data;
  // console.log("authorized_buffer data:", data);

  // console.log(`https://explorer.solana.com/tx/${txid}?cluster=devnet`);

  
  // PART 2
  idx = Buffer.from(new Uint8Array([4]));
  // form instruction
  let authEchoIx = new TransactionInstruction({
    keys: [
      {
        pubkey: vendingMachineBuffer,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: payer.publicKey,
        isSigner: true,
        isWritable: false,
      },
      {
        pubkey: payer_token_account.address,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: vendingMachineMint.publicKey,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: TOKEN_PROGRAM_ID,
        isSigner: false,
        isWritable: false,
      }
    ],
    programId: programId,
    data: Buffer.concat([
      idx,
      messageLen,
      message,
    ])
  });

  // // tx = new Transaction();
  tx.add(authEchoIx);

  txid = await sendAndConfirmTransaction(
    connection,
    tx,
    [payer],
    {
      skipPreflight: true,
      preflightCommitment: "confirmed",
      confirmation: "confirmed",
    }
  );
  console.log(`https://explorer.solana.com/tx/${txid}?cluster=devnet`);

  data = (await connection.getAccountInfo(vendingMachineBuffer)).data;
  console.log("vendingMachineBuffer data:", data.slice(9, data.length).toString());

};

main()
  .then(() => {
    console.log("Success");
  })
  .catch((e) => {
    console.error(e);
  });
