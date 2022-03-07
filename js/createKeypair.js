const {
  Keypair,
} = require("@solana/web3.js");


const payer = new Keypair();
console.log(payer);