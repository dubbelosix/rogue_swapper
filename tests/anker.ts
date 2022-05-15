import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { Anker } from "../target/types/anker";

describe("anker", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.Anker as Program<Anker>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
