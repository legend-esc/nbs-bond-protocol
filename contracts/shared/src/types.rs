use soroban_sdk::{contracttype, BytesN, Symbol, Vec};

#[derive(Clone, Copy, Debug, PartialEq)]
#[contracttype]
pub enum CreditType {
    Carbon,
    Biodiversity,
    Basket,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct BondConfig {
    pub project_id: BytesN<32>,
    pub face_value: i128,
    pub coupon_schedule: Vec<u64>,
    pub credit_type: CreditType,
    pub maturity_date: u64,
    pub total_supply: i128,
}

pub type BondId = u64;
pub type ReportId = u64;
pub type OrderId = u64;

#[derive(Clone)]
#[contracttype]
pub struct OracleReport {
    pub project_id: BytesN<32>,
    pub period_start: u64,
    pub period_end: u64,
    pub carbon_sequestered: i128,
    pub methodology: Symbol,
    pub provider_signature: BytesN<64>,
    pub ipfs_evidence_hash: BytesN<32>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[contracttype]
pub enum BondStatus {
    Active,
    Matured,
    Defaulted,
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[contracttype]
pub enum ProjectStatus {
    Pending,
    Approved,
    Rejected,
    Inactive,
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[contracttype]
pub enum ReportStatus {
    Pending,
    Verified,
    Challenged,
    Rejected,
}
