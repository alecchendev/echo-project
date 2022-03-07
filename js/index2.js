const {
  Connection,
  sendAndConfirmTransaction,
  Keypair,
  Transaction,
  SystemProgram,
  PublicKey,
  TransactionInstruction,
} = require("@solana/web3.js");

const connectionURLs = {
  localhost: "http://127.0.0.1:8899",
  devnet: "https://api.devnet.solana.com/",
}

const BN = require("bn.js");

const main = async () => {
  var args = process.argv.slice(2);
  const programId = new PublicKey(args[0]);
  const echo = args[1];
  const buffer_seed = parseInt(args[2]);

  const connection = new Connection(connectionURLs.localhost);

  // PART 1

  const authority = new Keypair();
  const [ authPda, bump ] = (await PublicKey.findProgramAddress(
    [Buffer.from("authority"),
    authority.publicKey.toBuffer(),
    Buffer.from(new Uint8Array((new BN(buffer_seed)).toArray("le", 8)))],
    programId
  ));

  console.log("Requesting Airdrop of 1 SOL...");
  await connection.requestAirdrop(authority.publicKey, 2e9);
  console.log("Airdrop received");

  let idx = Buffer.from(new Uint8Array([1]));
  // const buffer_seed_buf = Buffer.from(new Uint8Array((new BN(buffer_seed)).toArray("le", 4)))
  // const buffer_size = Buffer.from(new Uint8Array((new BN(echo.length)).toArray("le", 4)));
  const buffer_size = echo.length;
  const message = Buffer.from(echo, "ascii");
  const messageLen = Buffer.from(new Uint8Array((new BN(echo.length)).toArray("le", 4)));

  let initAuthIx = new TransactionInstruction({
    keys: [
      { // authority buffer
        pubkey: authPda,
        isSigner: false,
        isWritable: true,
      },
      { // authority
        pubkey: authority.publicKey,
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
      Buffer.from(new Uint8Array((new BN(buffer_seed)).toArray("le", 8))),
      Buffer.from(new Uint8Array((new BN(buffer_size)).toArray("le", 8)))
    ])
  });

  let tx = new Transaction();
  tx.add(initAuthIx);

  // let txid = await sendAndConfirmTransaction(
  //   connection,
  //   tx,
  //   [authority],
  //   {
  //     skipPreflight: true,
  //     preflightCommitment: "confirmed",
  //     confirmation: "confirmed",
  //   }
  // );

  // data = (await connection.getAccountInfo(authPda)).data;
  // console.log("authorized_buffer data:", data);

  
  // PART 2

  idx = Buffer.from(new Uint8Array([2]));

  let authEchoIx = new TransactionInstruction({
    keys: [
      { // authority buffer
        pubkey: authPda,
        isSigner: false,
        isWritable: true,
      },
      { // authority
        pubkey: authority.publicKey,
        isSigner: true,
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

  // tx = new Transaction();
  tx.add(authEchoIx);

  let txid = await sendAndConfirmTransaction(
    connection,
    tx,
    [authority],
    {
      skipPreflight: true,
      preflightCommitment: "confirmed",
      confirmation: "confirmed",
    }
  );
  // console.log(`https://explorer.solana.com/tx/${txid}?cluster=devnet`);

  data = (await connection.getAccountInfo(authPda)).data;
  console.log("authorized_buffer data:", data.slice(9, data.length).toString());

};

main()
  .then(() => {
    console.log("Success");
  })
  .catch((e) => {
    console.error(e);
  });
