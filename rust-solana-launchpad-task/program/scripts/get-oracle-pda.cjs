#!/usr/bin/env node
// Run from repo root: node program/scripts/get-oracle-pda.js
// Or from program/: node scripts/get-oracle-pda.js
const anchor = require("@coral-xyz/anchor");
const { PublicKey } = anchor.web3;
const ORACLE_PROGRAM_ID = new PublicKey("BkdrrqR4RrCB6m5YhMmWtjHpzuFPhdrvoA9x2tPFsWfj");
const [oraclePda] = PublicKey.findProgramAddressSync(
  [Buffer.from("oracle_state")],
  ORACLE_PROGRAM_ID
);
console.log("ORACLE_STATE_PUBKEY=" + oraclePda.toBase58());
