const assert = require("assert");
const anchor = require('@project-serum/anchor');
const serumCmn = require("@project-serum/common");
const { TOKEN_PROGRAM_ID, Token } = require("@solana/spl-token");
const TokenInstructions = require("@project-serum/serum").TokenInstructions;
const utils = require("./utils");
const { User, claimForUsers } = require("./user");

let program = anchor.workspace.RewardPool;

//Read the provider from the configured environmnet.
//represents an outside actor
//owns mints out of any other actors control, provides initial $$ to others
const envProvider = anchor.Provider.env();

//we allow this convenience var to change between default env and mock user(s)
//initially we are the outside actor
let provider = envProvider;
//convenience method to set in anchor AND above convenience var
//setting in anchor allows the rpc and accounts namespaces access
//to a different wallet from env
function setProvider(p) {
  provider = p;
  anchor.setProvider(p);
  program = new anchor.Program(program.idl, program.programId, p);
};
setProvider(provider);

describe('Multiuser Reward Pool', () => {

  const rewardDuration = new anchor.BN(10);
  const rewardDuration2 = new anchor.BN(20);

  let users;
  let funders;
  let mintA;
  let mintB;
  let stakingMint;
  let poolCreationAuthorityMint;
  let poolKeypairs = [1,1,1,1,1,1,1,1,1,1].map(a=>anchor.web3.Keypair.generate());

  it("Initialize mints", async () => {
    setProvider(envProvider);
    //these mints are ecosystem mints not owned
    //by funder or user
    mintA = await utils.createMint(provider, 9);
    mintB = await utils.createMint(provider, 9);
    stakingMint = await utils.createMint(provider, 9);
    poolCreationAuthorityMint = await utils.createMint(provider, 0);
  });

  it("Initialize program", async () => {
    setProvider(envProvider);
    //by funder or user
    await utils.initializeProgram(program, provider, poolCreationAuthorityMint.publicKey);
  });

  it("Initialize users", async () => {
    users = [1, 2, 3, 4, 5].map(a => new User(a));
    await Promise.all(
      users.map(a => a.init(10_000_000_000, poolCreationAuthorityMint.publicKey, false, stakingMint.publicKey, 5_000_000_000, mintA.publicKey, 0, mintB.publicKey, 0))
    );
  })

  it("Initialize funders", async () => {
    funders = [0,1,2,3,4,5,6,7,8,9].map(a => new User(a));
    await Promise.all(
      funders.map(a=>a.init(10_000_000_000, poolCreationAuthorityMint.publicKey, true, stakingMint.publicKey, 0, mintA.publicKey, 100_000_000_000, mintB.publicKey, 200_000_000_000))
    );
  });

  it("Creates pools", async () => {
    await Promise.all(
      funders.map(a=>a.initializePool(poolKeypairs[a.id], rewardDuration))
    );
  });

  it('User create staking accounts', async () => {
    await Promise.all(
      funders.map(a=>users[0].createUserStakingAccount(a.poolPubkey))
    );
  });

  //user[0] now has 10 farm accounts

  it('find all da farms', async () => {
    //these would be hard coded somewhere
    let theFarmPubkeys = poolKeypairs.map(a=>a.publicKey);
    //this is the wallet pubkey
    let walletPubKey = users[0].provider.wallet.publicKey;

    let before = Date.now();
    let userPoolAccounts = await getUserPools(walletPubKey, theFarmPubkeys);
    console.log("lookup all farms took: ", (Date.now() - before).toString() + "ms")

    userPoolAccounts.forEach(a => {
      console.log('pool: ', a.pool.toString(), ' balanceStaked: ', a.balanceStaked.toString());
    });
  });

});  

async function getUserPools(walletPubKey, theFarmPubkeys) {
  //derive all the user pool (farm) accounts at the same time
  let userPoolAccountsAndNonces = await Promise.all(
    theFarmPubkeys.map(a=>anchor.web3.PublicKey.findProgramAddress([walletPubKey.toBuffer(), a.toBuffer()], program.programId))
  );
  //lookup all farm accounts (this is the one and only rpc call)
  //the a=>a[0] is popping the publicKey from userPoolAccountsAndNonces - a[1] would be the nonce
  let accounts = await provider.connection.getMultipleAccountsInfo(userPoolAccountsAndNonces.map(a=>a[0]));
  //grab the accounts coder
  let accountsCoder = program.coder.accounts;
  //parse the accounts as our types
  return accounts.map(a=>accountsCoder.decode("User", a.data));
}

async function getTokenBalance(pubkey) {
  return parseFloat((await provider.connection.getTokenAccountBalance(pubkey)).value.uiAmount.toFixed(6))
}

async function wait(seconds) {
  while(seconds > 0) {
    console.log("countdown " + seconds--);
    await new Promise(a=>setTimeout(a, 1000));
  }
  console.log("wait over");
}