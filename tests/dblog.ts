import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { Dblog } from "../target/types/dblog";

describe("dblog", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.Dblog as Program<Dblog>;
  const connection = anchor.getProvider().connection;
  const payer = anchor.web3.Keypair.generate();
  const initAmount = 10 * anchor.web3.LAMPORTS_PER_SOL;
  const commitment: anchor.web3.Commitment = "processed";

  before(async () => {
    await connection.confirmTransaction(
      await connection.requestAirdrop(payer.publicKey, initAmount),
      commitment
    );
  });

  let preBlog;
  it("Is initialized!", async () => {
    // Add your test here.
    const nonce = Math.random().toString(16).substring(2, 14);
    const key = "QeSUFwff9xDbl4SCXlOmEn0TuS4vPg11r2_ETPPu_nk";
    const [blogPda] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("d-blog"), Buffer.from(nonce), payer.publicKey.toBuffer()],
      program.programId
    );
    preBlog = blogPda;
    const title = "哈".repeat(10);
    try {
      const tx = await program.methods
        .initialize(nonce, key, title, false, null)
        .accounts({
          blog: blogPda,
          owner: payer.publicKey,
          payer: payer.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        })
        .signers([payer])
        .rpc();
      console.log("Your transaction signature", tx);
      const blogPdaInfo = await connection.getAccountInfo(blogPda);
      console.log(payer.publicKey.toBuffer().toJSON());
      console.log(Buffer.from(key).toJSON());
      console.log(Buffer.from(nonce).toJSON());
      console.log("preBlog", blogPda.toString());

      console.log(
        "blogPdaInfo",
        blogPdaInfo.owner.toBase58(),
        JSON.stringify(blogPdaInfo.data.toJSON())
      );
      const blogData = await program.account.blog.fetch(blogPda);
      console.log(blogData);
    } catch (error) {
      console.error(error);
    }
  });

  it("Is initialized!", async () => {
    // Add your test here.
    const nonce = Math.random().toString(16).substring(2, 14);
    const key = "QeSUFwff9xDbl4SCXlOmEn0TuS4vPg11r2_ETPPu_nk";
    const [blogPda] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("d-blog"), Buffer.from(nonce), payer.publicKey.toBuffer()],
      program.programId
    );
    const title = "哈".repeat(20);
    try {
      const tx = await program.methods
        .initialize(nonce, key, title, false, null)
        .accounts({
          blog: blogPda,
          payer: payer.publicKey,
          owner: payer.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        })
        .remainingAccounts([
          {
            pubkey: preBlog,
            isSigner: false,
            isWritable: false,
          },
        ])
        .signers([payer])
        .rpc();
      console.log("Your transaction signature", tx);
      const blogPdaInfo = await connection.getAccountInfo(blogPda);
      console.log(payer.publicKey.toBuffer().toJSON());
      console.log(Buffer.from(key).toJSON());
      console.log(Buffer.from(nonce).toJSON());
      console.log(blogPda.toString());

      console.log(
        "blogPdaInfo",
        blogPdaInfo.owner.toBase58(),
        JSON.stringify(blogPdaInfo.data.toJSON())
      );
      const blogData = await program.account.blog.fetch(blogPda);
      console.log(blogData, blogData.prevBlog.toString());
    } catch (error) {
      console.error(error);
    }
  });
});
