import * as anchor from "@coral-xyz/anchor";
import { PublicKey, SystemProgram, Transaction, TransactionInstruction } from "@solana/web3.js";
import { readFileSync } from "fs";
import { fileURLToPath } from "url";
import { dirname, join } from "path";
import dotenv from "dotenv";

dotenv.config({ path: join(dirname(fileURLToPath(import.meta.url)), "..", "backend", ".env") });

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

async function main() {
  const rpcUrl = process.env.SOLANA_RPC_HTTP || "http://172.17.0.1:8899";
  const connection = new anchor.web3.Connection(rpcUrl, "confirmed");
  
  const keypairPath = process.env.BACKEND_KEYPAIR_PATH || "~/.config/solana/id.json";
  const expandedPath = keypairPath.replace("~", process.env.HOME);
  const secretKey = JSON.parse(readFileSync(expandedPath, "utf-8"));
  const payer = anchor.web3.Keypair.fromSecretKey(Uint8Array.from(secretKey));
  
  const provider = new anchor.AnchorProvider(connection, new anchor.Wallet(payer), {
    commitment: "confirmed",
  });
  anchor.setProvider(provider);

  const programIdStr = process.env.ORACLE_PROGRAM_ID;
  if (!programIdStr) {
    console.error("ERROR: ORACLE_PROGRAM_ID not found in .env");
    process.exit(1);
  }
  
  const programId = new PublicKey(programIdStr);
  const idlPath = join(__dirname, "target/idl/sol_usd_oracle.json");
  const idl = JSON.parse(readFileSync(idlPath, "utf-8"));
  
  const program = new anchor.Program(idl, provider);

  // Seed из IDL: [111, 114, 97, 99, 108, 101, 95, 115, 116, 97, 116, 101] = "oracle_state"
  const [oraclePda, bump] = PublicKey.findProgramAddressSync(
    [Buffer.from("oracle_state")],
    program.programId
  );

  console.log("Oracle PDA:", oraclePda.toString());
  console.log("Bump:", bump);
  console.log("Payer:", payer.publicKey.toString());

  // Проверка, существует ли уже аккаунт
  const accountInfo = await connection.getAccountInfo(oraclePda);
  if (accountInfo) {
    console.log("✓ Oracle already initialized!");
    return;
  }

  console.log("Oracle not initialized, creating...");

  try {
    // Метод принимает аргумент admin типа Pubkey
    const tx = await program.methods
      .initializeOracle(payer.publicKey) // Передаем admin
      .accountsPartial({
        oracle: oraclePda,
        payer: payer.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    console.log("✓ Oracle initialized! Transaction signature:", tx);
    
    const account = await program.account.oracleState.fetch(oraclePda);
    console.log("Oracle state:", account);
  } catch (err) {
    console.error("✗ Failed to initialize oracle:", err);
    console.error("\nError details:", err.message);
    if (err.logs) {
      console.error("\nTransaction logs:");
      err.logs.forEach(log => console.error(log));
    }
    process.exit(1);
  }
}

main().catch(console.error);