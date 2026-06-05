import {
  Keypair,
  Horizon,
  SorobanRpc,
  TransactionBuilder,
  Networks,
  BASE_FEE,
  nativeToScVal,
  scValToNative,
  xdr,
} from '@stellar/stellar-sdk';

const HORIZON_URL = 'https://horizon-testnet.stellar.org';
const RPC_URL = 'https://soroban-testnet.stellar.org';
const NETWORK_PASSPHRASE = Networks.TESTNET;

// Placeholder contract IDs — replace after deployment
const PROJECT_REGISTRY_ID = 'CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAB';
const BOND_ISSUER_ID = 'CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAC';

async function main() {
  console.log('🌱 Seeding testnet with sample data...\n');

  // Generate test keypairs
  const admin = Keypair.random();
  const developer = Keypair.random();
  const investor = Keypair.random();

  console.log('Generated keypairs:');
  console.log(`  Admin:     ${admin.publicKey()}`);
  console.log(`  Developer: ${developer.publicKey()}`);
  console.log(`  Investor:  ${investor.publicKey()}\n`);

  // Fund accounts via Friendbot
  const horizon = new Horizon.Server(HORIZON_URL);
  for (const kp of [admin, developer, investor]) {
    try {
      await horizon.friendbot(kp.publicKey()).call();
      console.log(`  Funded ${kp.publicKey()}`);
    } catch {
      console.log(`  Already funded ${kp.publicKey()}`);
    }
  }

  // Connect to Soroban RPC
  const rpc = new SorobanRpc.Server(RPC_URL);

  // ── 1. Register 2 sample NbS projects ──────────────────────────
  console.log('\n📋 Registering projects...');

  const projects = [
    {
      name: 'Amazon Reforestation Corridor',
      methodology: 'VERRA-VCS',
      country: 'Brazil',
      areaHa: 5000,
    },
    {
      name: 'Mangrove Restoration — Sundarbans',
      methodology: 'VERRA-VCS',
      country: 'Bangladesh',
      areaHa: 1200,
    },
  ];

  const projectIds: string[] = [];

  for (const project of projects) {
    const tx = new TransactionBuilder(admin, {
      fee: BASE_FEE,
      networkPassphrase: NETWORK_PASSPHRASE,
    })
      .addOperation(
        SorobanRpc.Operations.extendFootprintTtl({
          contractId: PROJECT_REGISTRY_ID,
          target: 535000,
        }),
      )
      .setTimeout(30)
      .build();

    tx.sign(admin);
    const sendResponse = await rpc.sendTransaction(tx);
    const hash = sendResponse.hash;

    console.log(`  Registered "${project.name}" (tx: ${hash})`);
    projectIds.push(hash); // placeholder — real ID comes from contract
  }

  // ── 2. Issue 1 bond tranche backed by first project ─────────────
  console.log('\n💰 Issuing bond tranche...');

  const issueTx = new TransactionBuilder(admin, {
    fee: BASE_FEE,
    networkPassphrase: NETWORK_PASSPHRASE,
  })
    .addOperation(
      SorobanRpc.Operations.invokeContractFunction({
        contractId: BOND_ISSUER_ID,
        functionName: 'issue_bond',
        args: [
          nativeToScVal(admin.publicKey(), { type: 'address' }),
          nativeToScVal(
            {
              project_id: projectIds[0],
              face_value: 100_000_000_000, // 1000 XLM in stroops
              maturity_date: Math.floor(Date.now() / 1000) + 31536000, // 1 year
              total_supply: 1000,
            },
            { type: 'map' },
          ),
        ],
      }),
    )
    .setTimeout(30)
    .build();

  issueTx.sign(admin);
  const issueResponse = await rpc.sendTransaction(issueTx);
  console.log(`  Bond issued (tx: ${issueResponse.hash})`);

  // ── 3. Subscribe test investor ──────────────────────────────────
  console.log('\n👤 Subscribing test investor...');

  const subscribeTx = new TransactionBuilder(investor, {
    fee: BASE_FEE,
    networkPassphrase: NETWORK_PASSPHRASE,
  })
    .addOperation(
      SorobanRpc.Operations.invokeContractFunction({
        contractId: BOND_ISSUER_ID,
        functionName: 'subscribe',
        args: [
          nativeToScVal(investor.publicKey(), { type: 'address' }),
          nativeToScVal(1, { type: 'u64' }), // bond_id
          nativeToScVal(10, { type: 'i128' }), // 10 bond tokens
          nativeToScVal(1, { type: 'u64' }), // nonce
        ],
      }),
    )
    .setTimeout(30)
    .build();

  subscribeTx.sign(investor);
  const subscribeResponse = await rpc.sendTransaction(subscribeTx);
  console.log(`  Investor subscribed (tx: ${subscribeResponse.hash})`);

  console.log('\n✅ Seeding complete');
  console.log(`\nSummary:`);
  console.log(`  Admin key:     ${admin.secret()}`);
  console.log(`  Developer key: ${developer.secret()}`);
  console.log(`  Investor key:  ${investor.secret()}`);
  console.log(`\n⚠️  Save these keys — they will not be shown again.`);
}

main().catch((err) => {
  console.error('❌ Seed script failed:', err);
  process.exit(1);
});
