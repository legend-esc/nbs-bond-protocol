# Oracle Design

## Architecture
Multi-source, multi-layer: Auditors + Satellite + IoT → OracleConsumer contract

## Provider Lifecycle
Register → Whitelisted → Submit Reports → Challenge Window → Verify/Reject

## Report Format
```
{
  project_id: BytesN<32>,
  period_start: u64,
  period_end: u64,
  carbon_sequestered: i128,
  methodology: Symbol,
  provider_signature: BytesN<64>,
  ipfs_evidence_hash: BytesN<32>,
}
```

## Challenge Mechanism
- 72-hour window from submission
- Any address can challenge with counter-evidence (IPFS hash)
- Admin resolves via on-chain vote

## Security Model
- Provider whitelist (admin-managed)
- Stake requirement (future)
- Multi-sig for high-value reports
