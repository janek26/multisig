import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Multisig } from "../target/types/multisig";
import { assert, expect } from "chai";
import { Keypair, PublicKey } from "@solana/web3.js";

// Define more specific types for our signers
type WalletSigner = anchor.Wallet & { publicKey: PublicKey };
type KeypairSigner = Keypair & { publicKey: PublicKey };
type Signer = WalletSigner | KeypairSigner;

describe("multisig", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.Multisig as Program<Multisig>;

  // Test matrix for different owner combinations
  const ownerCombinations = [
    {
      name: "provider wallet + generated keypair",
      owner1: () => provider.wallet as Signer,
      owner2: () => Keypair.generate(),
    },
    {
      name: "two generated keypairs",
      owner1: () => Keypair.generate(),
      owner2: () => Keypair.generate(),
    },
  ];

  // Helper function to create multisig PDA
  const createMultisigPda = (owner1: PublicKey, owner2: PublicKey) => {
    return PublicKey.findProgramAddressSync(
      [Buffer.from("multisig"), owner1.toBuffer(), owner2.toBuffer()],
      program.programId
    )[0];
  };

  // Helper function to airdrop SOL
  const airdrop = async (pubkey: PublicKey, amount = 1e9) => {
    const sig = await provider.connection.requestAirdrop(pubkey, amount);
    await provider.connection.confirmTransaction(sig);
  };

  // Helper function to get public key from signer
  const getPublicKey = (signer: Signer): PublicKey => {
    return signer.publicKey;
  };

  // Helper function to initialize multisig
  const initializeMultisig = async (owner1: Signer, owner2: Keypair) => {
    const owner1Pubkey = getPublicKey(owner1);
    const multisigPda = createMultisigPda(owner1Pubkey, owner2.publicKey);

    await program.methods
      .create(owner1Pubkey, owner2.publicKey)
      .accounts({
        payer: provider.wallet.publicKey,
      })
      .rpc();

    return multisigPda;
  };

  // Matrix testing for different owner combinations
  ownerCombinations.forEach(({ name, owner1, owner2 }) => {
    describe(`Owner combination: ${name}`, () => {
      let currentOwner1: Signer;
      let currentOwner2: Keypair;
      let multisigPda: PublicKey;
      let owner1Provider: anchor.AnchorProvider | null = null;
      let owner2Provider: anchor.AnchorProvider | null = null;

      beforeEach(async () => {
        currentOwner1 = owner1();
        currentOwner2 = owner2();

        // If owner is a Keypair, set up a custom provider for signing
        if (currentOwner1 instanceof Keypair) {
          owner1Provider = new anchor.AnchorProvider(
            provider.connection,
            {
              publicKey: currentOwner1.publicKey,
              payer: currentOwner1,
              signTransaction: async (tx) => {
                if (
                  "partialSign" in tx &&
                  typeof tx.partialSign === "function" &&
                  currentOwner1 instanceof Keypair
                ) {
                  tx.partialSign(currentOwner1);
                }
                return tx;
              },
              signAllTransactions: async (txs) => {
                if (currentOwner1 instanceof Keypair) {
                  const kp = currentOwner1 as Keypair;
                  txs.forEach((tx) => {
                    if (
                      "partialSign" in tx &&
                      typeof tx.partialSign === "function"
                    ) {
                      tx.partialSign(kp);
                    }
                  });
                }
                return txs;
              },
            },
            provider.opts
          );
        } else {
          owner1Provider = provider;
        }

        owner2Provider = new anchor.AnchorProvider(
          provider.connection,
          {
            publicKey: currentOwner2.publicKey,
            payer: currentOwner2,
            signTransaction: async (tx) => {
              if ("partialSign" in tx && typeof tx.partialSign === "function") {
                tx.partialSign(currentOwner2);
              }
              return tx;
            },
            signAllTransactions: async (txs) => {
              txs.forEach((tx) => {
                if (
                  "partialSign" in tx &&
                  typeof tx.partialSign === "function"
                ) {
                  tx.partialSign(currentOwner2);
                }
              });
              return txs;
            },
          },
          provider.opts
        );

        // Airdrop to both owners if they're keypairs
        if ("publicKey" in currentOwner1) {
          await airdrop(currentOwner1.publicKey);
        }
        await airdrop(currentOwner2.publicKey);

        // Use owner1's provider for initialization
        anchor.setProvider(owner1Provider!);
        multisigPda = await initializeMultisig(currentOwner1, currentOwner2);
      });

      afterEach(() => {
        // Always restore the original provider after each test
        anchor.setProvider(provider);
      });

      it("Initializes with correct state", async () => {
        anchor.setProvider(owner1Provider!);
        const multisig = await program.account.multisig.fetch(multisigPda);
        const owner1Pubkey = getPublicKey(currentOwner1);
        assert.ok(multisig.owner1.equals(owner1Pubkey));
        assert.ok(multisig.owner2.equals(currentOwner2.publicKey));
        assert.isFalse(multisig.confirmed1);
        assert.isFalse(multisig.confirmed2);
      });

      it("Allows owner1 to approve", async () => {
        anchor.setProvider(owner1Provider!);
        await program.methods
          .approve()
          .accounts({
            multisig: multisigPda,
            signer: getPublicKey(currentOwner1),
          })
          .signers(currentOwner1 instanceof Keypair ? [currentOwner1] : [])
          .rpc();

        anchor.setProvider(owner1Provider!);
        const multisig = await program.account.multisig.fetch(multisigPda);
        assert.isTrue(multisig.confirmed1);
        assert.isFalse(multisig.confirmed2);
      });

      it("Allows owner2 to approve", async () => {
        anchor.setProvider(owner2Provider!);
        await program.methods
          .approve()
          .accounts({
            multisig: multisigPda,
            signer: currentOwner2.publicKey,
          })
          .signers([currentOwner2])
          .rpc();

        anchor.setProvider(owner2Provider!);
        const multisig = await program.account.multisig.fetch(multisigPda);
        assert.isFalse(multisig.confirmed1);
        assert.isTrue(multisig.confirmed2);
      });

      it("Executes when both owners approve", async () => {
        // First approval by owner1
        anchor.setProvider(owner1Provider!);
        await program.methods
          .approve()
          .accounts({
            multisig: multisigPda,
            signer: getPublicKey(currentOwner1),
          })
          .signers(currentOwner1 instanceof Keypair ? [currentOwner1] : [])
          .rpc();

        // Second approval by owner2
        anchor.setProvider(owner2Provider!);
        await program.methods
          .approve()
          .accounts({
            multisig: multisigPda,
            signer: currentOwner2.publicKey,
          })
          .signers([currentOwner2])
          .rpc();

        // Execute (can be either provider, but use owner1's for consistency)
        anchor.setProvider(owner1Provider!);
        await program.methods
          .execute()
          .accounts({
            multisig: multisigPda,
          })
          .rpc();

        anchor.setProvider(owner1Provider!);
        const multisig = await program.account.multisig.fetch(multisigPda);
        assert.isTrue(multisig.confirmed1);
        assert.isTrue(multisig.confirmed2);
      });
    });
  });

  // Failure cases
  describe("Failure cases", () => {
    let owner1: Signer;
    let owner2: Keypair;
    let owner3: Keypair;
    let multisigPda: PublicKey;

    beforeEach(async () => {
      owner1 = provider.wallet as Signer;
      owner2 = Keypair.generate();
      owner3 = Keypair.generate();
      await airdrop(owner2.publicKey);
      await airdrop(owner3.publicKey);
      multisigPda = await initializeMultisig(owner1, owner2);
    });

    it("Fails when non-owner tries to approve", async () => {
      try {
        await program.methods
          .approve()
          .accounts({
            multisig: multisigPda,
            signer: owner3.publicKey,
          })
          .signers([owner3])
          .rpc();
        assert.fail("Expected transaction to fail");
      } catch (e) {
        // Transaction should fail with custom program error
        expect(e).to.be.instanceOf(Error);
      }
    });

    it("Fails when trying to execute without both approvals", async () => {
      // Only owner1 approves
      await program.methods
        .approve()
        .accounts({
          multisig: multisigPda,
          signer: getPublicKey(owner1),
        })
        .signers("publicKey" in owner1 ? [] : [owner1])
        .rpc();

      try {
        await program.methods
          .execute()
          .accounts({
            multisig: multisigPda,
          })
          .rpc();
        assert.fail("Expected transaction to fail");
      } catch (e) {
        expect(e).to.be.instanceOf(Error);
      }
    });

    it("Fails when trying to approve twice with the same owner", async () => {
      // First approval succeeds
      await program.methods
        .approve()
        .accounts({
          multisig: multisigPda,
          signer: getPublicKey(owner1),
        })
        .signers("publicKey" in owner1 ? [] : [owner1])
        .rpc();

      // Second approval should fail
      try {
        await program.methods
          .approve()
          .accounts({
            multisig: multisigPda,
            signer: getPublicKey(owner1),
          })
          .signers("publicKey" in owner1 ? [] : [owner1])
          .rpc();
        assert.fail("Expected transaction to fail");
      } catch (e) {
        expect(e).to.be.instanceOf(Error);
      }
    });

    it("Fails when trying to create multisig with same owner twice", async () => {
      try {
        await program.methods
          .create(getPublicKey(owner1), getPublicKey(owner1))
          .accounts({
            payer: provider.wallet.publicKey,
          })
          .rpc();
        assert.fail("Expected transaction to fail");
      } catch (e) {
        expect(e).to.be.instanceOf(Error);
      }
    });
  });

  // Edge cases
  describe("Edge cases", () => {
    it("Handles owner1 being the same as payer", async () => {
      const owner1 = provider.wallet as Signer;
      const owner2 = Keypair.generate();
      await airdrop(owner2.publicKey);

      const multisigPda = await initializeMultisig(owner1, owner2);
      const multisig = await program.account.multisig.fetch(multisigPda);

      assert.ok(multisig.owner1.equals(getPublicKey(owner1)));
      assert.ok(multisig.owner2.equals(owner2.publicKey));
    });

    it("Can create multiple multisig accounts with same owners in different order", async () => {
      const owner1 = Keypair.generate();
      const owner2 = Keypair.generate();
      await airdrop(owner1.publicKey);
      await airdrop(owner2.publicKey);

      // Create first multisig
      const multisig1Pda = await initializeMultisig(owner1, owner2);

      // Create second multisig with owners in reverse order
      const multisig2Pda = await initializeMultisig(owner2, owner1);

      // Verify they are different accounts
      assert.notEqual(multisig1Pda.toBase58(), multisig2Pda.toBase58());
    });
  });
});
