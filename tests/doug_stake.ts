import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { DougStake } from "../target/types/doug_stake";

import {
  createMint,
  getOrCreateAssociatedTokenAccount,
  mintTo,
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import {
  Connection,
  LAMPORTS_PER_SOL,
  PublicKey,
  SystemProgram,
} from "@solana/web3.js";
import assert from "assert";

const RPC_URL = "http://127.0.0.1:8899";
const DECIMALS_PER_TOKEN = 1000000;

const createTokenMint = async (
  connection: any,
  payer: anchor.Wallet,
  mintKeypair: anchor.web3.Keypair
) => {
  try {
    const mint = await createMint(
      connection,
      payer.payer,
      payer.publicKey,
      payer.publicKey,
      6,
      mintKeypair
    );

    // console.log(mint);
  } catch (e) {
    console.log(e);
  }
};

describe("doug_stake", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const payer = provider.wallet as anchor.Wallet;
  const connection = new Connection(RPC_URL, "confirmed");

  const program = anchor.workspace.DougStake as Program<DougStake>;

  // 5qZFpca9BAY3gyVNkXVoDBLd3z2FY2DndXSg8qL9LDyD
  const dougToken = anchor.web3.Keypair.fromSecretKey(
    new Uint8Array([
      82, 182, 168, 68, 185, 27, 188, 83, 6, 192, 235, 141, 68, 157, 219, 58,
      150, 38, 123, 228, 28, 172, 145, 66, 207, 251, 221, 18, 236, 237, 132, 61,
      71, 223, 25, 21, 198, 248, 212, 9, 50, 97, 15, 82, 134, 215, 228, 87, 178,
      149, 140, 165, 200, 3, 91, 121, 69, 55, 222, 156, 44, 85, 205, 68,
    ])
  );

  const [REWARD_VAULT_PDA] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("reward_vault")],
    program.programId
  );

  const [VAULT_INFO_PDA] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("vault_info")],
    program.programId
  );

  const [USER_VAULT_PDA] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("user_vault"), payer.publicKey.toBuffer()],
    program.programId
  );

  const [STAKE_ACCOUNT_SEED] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("stake_account"), payer.publicKey.toBuffer()],
    program.programId
  );

  console.log("REWARD_VAULT_PDA", REWARD_VAULT_PDA.toBase58());
  console.log("VAULT_INFO_PDA", VAULT_INFO_PDA.toBase58());

  it("Initializes the vault info account", async () => {
    await createTokenMint(connection, payer, dougToken);

    await program.methods
      .init()
      .accounts({
        vaultInfo: VAULT_INFO_PDA,
        rewardVault: REWARD_VAULT_PDA,
        user: payer.publicKey,
        rewardTokenMint: dougToken.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc()
      .catch((err) => {
        console.log(err);
      });

    const vaultInfo = await program.account.vaultInfo.fetch(VAULT_INFO_PDA);
    console.log("vaultInfo", vaultInfo);
  });

  it("funds the reward vault", async () => {
    const mintAndSendTokens = async (
      mintPubKey: PublicKey,
      destination: PublicKey,
      amount: number
    ) => {
      await mintTo(
        connection,
        payer.payer,
        mintPubKey,
        destination,
        payer.payer,
        amount * DECIMALS_PER_TOKEN
      );
    };

    // funds the user account
    const userTokenAccount = await getOrCreateAssociatedTokenAccount(
      connection,
      payer.payer,
      dougToken.publicKey,
      payer.publicKey
    );

    await mintAndSendTokens(dougToken.publicKey, REWARD_VAULT_PDA, 100000);
    await mintAndSendTokens(
      dougToken.publicKey,
      userTokenAccount.address,
      100000
    );

    const rewardVaultBalance = await connection.getTokenAccountBalance(
      REWARD_VAULT_PDA
    );
    console.log("balance", rewardVaultBalance);
  });

  it("stakes the user tokens", async () => {
    const stakeAmount = new anchor.BN(100 * DECIMALS_PER_TOKEN);
    const duration = new anchor.BN(1);

    const userTokenAccount = await getOrCreateAssociatedTokenAccount(
      connection,
      payer.payer,
      dougToken.publicKey,
      payer.publicKey
    );

    await program.methods
      .stake(stakeAmount, duration)
      .accounts({
        vaultInfo: VAULT_INFO_PDA,
        userVault: USER_VAULT_PDA,
        stakeAccount: STAKE_ACCOUNT_SEED,
        userTokenAccount: userTokenAccount.address,
        user: payer.publicKey,
        mint: dougToken.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .rpc()
      .catch((err) => {
        console.log(err);
      });

    // get stake account
    const stakeAccount: any = await program.account.stakeAccount.fetch(
      STAKE_ACCOUNT_SEED
    );

    console.log(
      "Amount Staked: ",
      stakeAccount?.amount.toNumber() / DECIMALS_PER_TOKEN
    );

    console.log(
      "Rewards Earned: ",
      stakeAccount?.rewards.toNumber() / DECIMALS_PER_TOKEN
    );

    // sleep for 2 minutes
    await new Promise((r) => setTimeout(r, 120000));
  });

  it("restakes the users tokens", async () => {
    await program.methods
      .restake()
      .accounts({
        vaultInfo: VAULT_INFO_PDA,
        userVault: USER_VAULT_PDA,
        stakeAccount: STAKE_ACCOUNT_SEED,
        rewardVault: REWARD_VAULT_PDA,
        user: payer.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .rpc()
      .catch((err) => {
        console.log(err);
      });

    // get stake account
    const stakeAccount: any = await program.account.stakeAccount.fetch(
      STAKE_ACCOUNT_SEED
    );

    console.log(
      "Amount Staked: ",
      stakeAccount?.amount.toNumber() / DECIMALS_PER_TOKEN
    );

    console.log(
      "Rewards Earned: ",
      stakeAccount?.rewards.toNumber() / DECIMALS_PER_TOKEN
    );

    // sleep for 2 minutes
    await new Promise((r) => setTimeout(r, 120000));
  });

  // it("extends the users stake duration", async () => {
  //   const duration = new anchor.BN(1);

  //   await program.methods
  //     .extend(duration)
  //     .accounts({
  //       stakeAccount: STAKE_ACCOUNT_SEED,
  //       user: payer.publicKey,
  //       systemProgram: SystemProgram.programId,
  //     })
  //     .rpc()
  //     .catch((err) => {
  //       console.log(err);
  //     });

  //   await new Promise((r) => setTimeout(r, 120000));
  // });

  it("unstakes the users tokens", async () => {
    const userTokenAccount = await getOrCreateAssociatedTokenAccount(
      connection,
      payer.payer,
      dougToken.publicKey,
      payer.publicKey
    );

    await program.methods
      .unstake()
      .accounts({
        vaultInfo: VAULT_INFO_PDA,
        userVault: USER_VAULT_PDA,
        stakeAccount: STAKE_ACCOUNT_SEED,
        userTokenAccount: userTokenAccount.address,
        rewardVault: REWARD_VAULT_PDA,
        user: payer.publicKey,
        mint: dougToken.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .rpc()
      .catch((err) => {
        console.log(err);
      });
  });
});
