import * as anchor from "@coral-xyz/anchor";
import * as spl from "@solana/spl-token";
import { Program } from "@coral-xyz/anchor";
import { MintPlexerProgram } from "../target/types/mint_plexer_program";
import {
  Keypair,
  PublicKey,
  SystemProgram,
  Transaction,
} from "@solana/web3.js";
import {
  ASSOCIATED_PROGRAM_ID,
  associatedAddress,
} from "@coral-xyz/anchor/dist/cjs/utils/token";

describe("mint-plexer", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace
    .MintPlexerProgram as Program<MintPlexerProgram>;

  const programId = new PublicKey(
    "DcoHFXZHaLRQ2B37Bqc7afMpPr8T9VULyNdGj87wctcv"
  );

  const wallet = anchor.getProvider();

  const borg = Keypair.generate();
  const mintPlexer = PublicKey.findProgramAddressSync(
    [Buffer.from("mint_plexer"), borg.publicKey.toBuffer()],
    programId
  );

  let wborgATA: anchor.web3.PublicKey;
  let wBorg: anchor.web3.PublicKey;

  it("Initialize mintPlexer", async () => {
    const tx = await program.methods
      .initialize(mintPlexer[1], 9)
      .accounts({
        authority: wallet.publicKey,
        mainMint: borg.publicKey,
        mintPlexer: mintPlexer[0],
        systemProgram: SystemProgram.programId,
      })
      .signers([borg])
      .rpc();

    console.log("Init tx :", tx);
  });

  it("add pair", async () => {
    const authority = Keypair.generate();
    const transfer = new Transaction().add(
      SystemProgram.transfer({
        toPubkey: authority.publicKey,
        fromPubkey: wallet.publicKey,
        lamports: 5000000,
      })
    );
    await wallet.sendAndConfirm(transfer);

    wBorg = await spl.createMint(
      wallet.connection,
      authority,
      authority.publicKey,
      null,
      9
    );

    wborgATA = associatedAddress({
      mint: wBorg,
      owner: mintPlexer[0],
    });

    const tx = await program.methods
      .addPair()
      .accounts({
        mintPlexer: mintPlexer[0],
        authority: wallet.publicKey,
        mainMint: borg.publicKey,
        newPair: wBorg,
        newPairTokenAccount: wborgATA,
        tokenProgram: spl.TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .rpc({ skipPreflight: true });

    console.log("add pair :", tx);
  });
});
