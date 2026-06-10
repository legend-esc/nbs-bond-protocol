#[cfg(test)]
mod integration {
    use soroban_sdk::{
        testutils::Address as _, Address, BytesN, Env, Symbol,
    };
    use nbbs_project_registry::{ProjectRegistry, ProjectRegistryClient};
    use nbbs_bond_issuer::{BondIssuer, BondIssuerClient};
    use nbbs_coupon_engine::{CouponEngine, CouponEngineClient};
    use nbbs_oracle_consumer::{OracleConsumer, OracleConsumerClient};
    use nbbs_dex_router::{DEXRouter, DEXRouterClient};
    use nbbs_credit_retirement::{CreditRetirement, CreditRetirementClient};
    use nbbs_shared::{
        BondConfig, BondError, CreditType, OracleError, OracleReport,
        ProjectStatus, RegistryError, ReportStatus,
    };

    fn make_project_id(env: &Env, value: u8) -> BytesN<32> {
        let mut arr = [0u8; 32];
        arr[31] = value;
        BytesN::from_array(env, &arr)
    }

    fn make_ipfs_hash(env: &Env, value: u8) -> BytesN<32> {
        let mut arr = [0u8; 32];
        arr[0] = value;
        BytesN::from_array(env, &arr)
    }

    fn make_signature(env: &Env) -> BytesN<64> {
        BytesN::from_array(env, &[0u8; 64])
    }

    fn make_report(
        env: &Env,
        project_id: BytesN<32>,
        carbon_sequestered: i128,
    ) -> OracleReport {
        OracleReport {
            project_id,
            period_start: 1000,
            period_end: 2000,
            carbon_sequestered,
            methodology: Symbol::new(env, "verra_vcs"),
            provider_signature: make_signature(env),
            ipfs_evidence_hash: make_ipfs_hash(env, 1),
        }
    }

    fn make_bond_config(
        env: &Env,
        project_id: BytesN<32>,
        total_supply: i128,
    ) -> BondConfig {
        BondConfig {
            project_id,
            face_value: 1000,
            coupon_schedule: soroban_sdk::vec![env, 1_000_000u64, 2_000_000u64],
            credit_type: CreditType::Carbon,
            maturity_date: 3_000_000,
            total_supply,
        }
    }

    struct TestContracts<'a> {
        pr_client: ProjectRegistryClient<'a>,
        bi_client: BondIssuerClient<'a>,
        ce_client: CouponEngineClient<'a>,
        oc_client: OracleConsumerClient<'a>,
        dr_client: DEXRouterClient<'a>,
        cr_client: CreditRetirementClient<'a>,
    }

    fn deploy_contracts<'a>(env: &'a Env, admin: &Address) -> TestContracts<'a> {
        let pr_addr = env.register(ProjectRegistry, (admin.clone(),));
        let pr_client = ProjectRegistryClient::new(env, &pr_addr);

        let bi_addr = env.register(BondIssuer, (admin.clone(),));
        let bi_client = BondIssuerClient::new(env, &bi_addr);

        let oc_addr = env.register(OracleConsumer, (admin.clone(),));
        let oc_client = OracleConsumerClient::new(env, &oc_addr);

        let ce_addr = env.register(
            CouponEngine,
            (admin.clone(), bi_addr.clone(), oc_addr.clone()),
        );
        let ce_client = CouponEngineClient::new(env, &ce_addr);

        let dr_addr = env.register(
            DEXRouter,
            (admin.clone(), bi_addr.clone(), ce_addr.clone()),
        );
        let dr_client = DEXRouterClient::new(env, &dr_addr);

        let cr_addr = env.register(CreditRetirement, (admin.clone(),));
        let cr_client = CreditRetirementClient::new(env, &cr_addr);

        TestContracts {
            pr_client,
            bi_client,
            ce_client,
            oc_client,
            dr_client,
            cr_client,
        }
    }

    mod full_lifecycle {
        use super::*;

        #[test]
        fn test_happy_path() {
            let env = Env::default();
            env.mock_all_auths();

            let admin = Address::generate(&env);
            let alice = Address::generate(&env);
            let bob = Address::generate(&env);
            let oracle = Address::generate(&env);
            let contracts = deploy_contracts(&env, &admin);

            let project_id = make_project_id(&env, 1);

            let pid = contracts.pr_client.register_project(
                &alice,
                &make_ipfs_hash(&env, 1),
                &Symbol::new(&env, "VCS"),
                &Symbol::new(&env, "US"),
                &0,
            );
            assert_eq!(pid, 1);

            contracts.pr_client.approve_project(&admin, &pid, &0);

            let project = contracts.pr_client.get_project(&pid);
            assert_eq!(project.status, ProjectStatus::Approved);

            let config = make_bond_config(&env, project_id.clone(), 10_000);
            let bond_id = contracts.bi_client.issue_bond(&admin, &config, &0);
            assert_eq!(bond_id, 1);

            contracts.bi_client.subscribe(&bob, &bond_id, &1_000, &0);
            let balance = contracts.bi_client.get_holder_balance(&bond_id, &bob);
            assert_eq!(balance, 1_000);

            contracts.oc_client.register_provider(
                &admin,
                &oracle,
                &Symbol::new(&env, "verra_vcs"),
                &0,
            );

            let report_id = contracts.oc_client.submit_report(
                &oracle,
                &project_id,
                &1000u64,
                &2000u64,
                &100_000i128,
                &Symbol::new(&env, "verra_vcs"),
                &make_ipfs_hash(&env, 1),
                &0,
            );
            assert_eq!(report_id, 1);

            contracts.oc_client.verify_report(&admin, &report_id, &1);

            let report = contracts.oc_client.get_report(&report_id);
            assert_eq!(report.status, ReportStatus::Verified);

            contracts.ce_client.register_bond(&admin, &bond_id, &project_id, &0);

            let holders = soroban_sdk::vec![&env, bob.clone()];
            let oracle_report = make_report(&env, project_id, 100_000);
            let result = contracts.ce_client.distribute_coupon(
                &admin,
                &bond_id,
                &0,
                &holders,
                &oracle_report,
                &1,
            );
            assert!(result.total_credits > 0);
            assert_eq!(result.holder_count, 1);

            let accrued = contracts.ce_client.accrued_credits(&bond_id, &bob);
            assert!(accrued > 0);

            let credit_hash = make_ipfs_hash(&env, 42);
            let retirement_id = contracts.cr_client.retire_credits(
                &bob,
                &bond_id,
                &accrued,
                &CreditType::Carbon,
                &credit_hash,
                &0,
            );
            assert_eq!(retirement_id, 1);

            let record = contracts.cr_client.get_retirement_record(&retirement_id);
            assert_eq!(record.holder, bob);
            assert_eq!(record.amount, accrued);
            assert_eq!(record.credit_type, CreditType::Carbon);
            assert_eq!(record.certificate_ipfs_hash, credit_hash);

            let total_retired = contracts.cr_client.get_total_retired(&bob);
            assert_eq!(total_retired, accrued);

            assert_eq!(contracts.cr_client.total_retirements(), 1);
        }

        #[test]
        fn test_insufficient_supply() {
            let env = Env::default();
            env.mock_all_auths();

            let admin = Address::generate(&env);
            let alice = Address::generate(&env);
            let bob = Address::generate(&env);
            let contracts = deploy_contracts(&env, &admin);

            let project_id = make_project_id(&env, 1);

            let pid = contracts.pr_client.register_project(
                &alice,
                &make_ipfs_hash(&env, 1),
                &Symbol::new(&env, "VCS"),
                &Symbol::new(&env, "US"),
                &0,
            );
            contracts.pr_client.approve_project(&admin, &pid, &0);

            let config = make_bond_config(&env, project_id, 1_000);
            let bond_id = contracts.bi_client.issue_bond(&admin, &config, &0);

            contracts.bi_client.subscribe(&alice, &bond_id, &1_000, &0);

            let result = contracts
                .bi_client
                .try_subscribe(&bob, &bond_id, &1, &0);
            assert_eq!(result, Err(Ok(BondError::InsufficientSupply)));
        }
    }

    mod oracle {
        use super::*;

        #[test]
        fn test_challenge_flow() {
            let env = Env::default();
            env.mock_all_auths();

            let admin = Address::generate(&env);
            let alice = Address::generate(&env);
            let oracle = Address::generate(&env);
            let challenger = Address::generate(&env);
            let contracts = deploy_contracts(&env, &admin);

            let project_id = make_project_id(&env, 1);

            let pid = contracts.pr_client.register_project(
                &alice,
                &make_ipfs_hash(&env, 1),
                &Symbol::new(&env, "VCS"),
                &Symbol::new(&env, "US"),
                &0,
            );
            contracts.pr_client.approve_project(&admin, &pid, &0);

            contracts.oc_client.register_provider(
                &admin,
                &oracle,
                &Symbol::new(&env, "verra_vcs"),
                &0,
            );

            let report_id = contracts.oc_client.submit_report(
                &oracle,
                &project_id,
                &1000u64,
                &2000u64,
                &100_000i128,
                &Symbol::new(&env, "verra_vcs"),
                &make_ipfs_hash(&env, 1),
                &0,
            );

            contracts.oc_client.challenge_report(
                &challenger,
                &report_id,
                &make_ipfs_hash(&env, 2),
                &0,
            );

            let report = contracts.oc_client.get_report(&report_id);
            assert_eq!(report.status, ReportStatus::Challenged);

            contracts.oc_client.resolve_challenge(
                &admin,
                &report_id,
                &ReportStatus::Rejected,
                &1,
            );

            let resolved = contracts.oc_client.get_report(&report_id);
            assert_eq!(resolved.status, ReportStatus::Rejected);
        }
    }

    mod dex {
        use super::*;

        #[test]
        fn test_order_full_fill() {
            let env = Env::default();
            env.mock_all_auths();

            let admin = Address::generate(&env);
            let alice = Address::generate(&env);
            let bob = Address::generate(&env);
            let contracts = deploy_contracts(&env, &admin);

            let project_id = make_project_id(&env, 1);

            let pid = contracts.pr_client.register_project(
                &alice,
                &make_ipfs_hash(&env, 1),
                &Symbol::new(&env, "VCS"),
                &Symbol::new(&env, "US"),
                &0,
            );
            contracts.pr_client.approve_project(&admin, &pid, &0);

            let config = make_bond_config(&env, project_id, 10_000);
            let bond_id = contracts.bi_client.issue_bond(&admin, &config, &0);
            contracts.bi_client.subscribe(&alice, &bond_id, &5_000, &0);

            let order_id = contracts.dr_client.list_bond_tokens(
                &alice,
                &bond_id,
                &1_000i128,
                &100i128,
                &Symbol::new(&env, "USDC"),
                &3600u64,
                &0,
            );
            assert_eq!(order_id, 1);

            contracts
                .dr_client
                .execute_purchase(&bob, &order_id, &100i128, &1_000i128, &0);

            let order = contracts.dr_client.get_order(&order_id);
            assert_eq!(order.status, nbbs_dex_router::OrderStatus::Filled);
        }

        #[test]
        fn test_order_partial_fill() {
            let env = Env::default();
            env.mock_all_auths();

            let admin = Address::generate(&env);
            let alice = Address::generate(&env);
            let bob = Address::generate(&env);
            let contracts = deploy_contracts(&env, &admin);

            let project_id = make_project_id(&env, 1);

            let pid = contracts.pr_client.register_project(
                &alice,
                &make_ipfs_hash(&env, 1),
                &Symbol::new(&env, "VCS"),
                &Symbol::new(&env, "US"),
                &0,
            );
            contracts.pr_client.approve_project(&admin, &pid, &0);

            let config = make_bond_config(&env, project_id, 10_000);
            let bond_id = contracts.bi_client.issue_bond(&admin, &config, &0);
            contracts.bi_client.subscribe(&alice, &bond_id, &5_000, &0);

            let order_id = contracts.dr_client.list_bond_tokens(
                &alice,
                &bond_id,
                &1_000i128,
                &100i128,
                &Symbol::new(&env, "USDC"),
                &3600u64,
                &0,
            );

            contracts
                .dr_client
                .execute_purchase(&bob, &order_id, &100i128, &400i128, &0);

            let order = contracts.dr_client.get_order(&order_id);
            assert_eq!(order.status, nbbs_dex_router::OrderStatus::PartiallyFilled);
            assert_eq!(order.amount, 600);

            contracts
                .dr_client
                .execute_purchase(&bob, &order_id, &100i128, &600i128, &1);

            let order = contracts.dr_client.get_order(&order_id);
            assert_eq!(order.status, nbbs_dex_router::OrderStatus::Filled);
        }
    }

    mod security {
        use super::*;

        #[test]
        fn test_nonce_replay() {
            let env = Env::default();
            env.mock_all_auths();

            let admin = Address::generate(&env);
            let alice = Address::generate(&env);
            let contracts = deploy_contracts(&env, &admin);

            contracts.pr_client.register_project(
                &alice,
                &make_ipfs_hash(&env, 1),
                &Symbol::new(&env, "VCS"),
                &Symbol::new(&env, "US"),
                &0,
            );

            let result = contracts.pr_client.try_register_project(
                &alice,
                &make_ipfs_hash(&env, 2),
                &Symbol::new(&env, "GS"),
                &Symbol::new(&env, "BR"),
                &0,
            );
            assert_eq!(result, Err(Ok(RegistryError::InvalidNonce)));

            let id = contracts.pr_client.register_project(
                &alice,
                &make_ipfs_hash(&env, 2),
                &Symbol::new(&env, "GS"),
                &Symbol::new(&env, "BR"),
                &1,
            );
            assert_eq!(id, 2);
        }

        #[test]
        fn test_permission_checks() {
            let env = Env::default();
            env.mock_all_auths();

            let admin = Address::generate(&env);
            let alice = Address::generate(&env);
            let bob = Address::generate(&env);
            let _oracle = Address::generate(&env);
            let contracts = deploy_contracts(&env, &admin);

            let project_id = make_project_id(&env, 1);

            let pid = contracts.pr_client.register_project(
                &alice,
                &make_ipfs_hash(&env, 1),
                &Symbol::new(&env, "VCS"),
                &Symbol::new(&env, "US"),
                &0,
            );

            let result = contracts
                .pr_client
                .try_approve_project(&bob, &pid, &0);
            assert_eq!(result, Err(Ok(RegistryError::Unauthorized)));

            let config = make_bond_config(&env, project_id.clone(), 10_000);
            let result = contracts.bi_client.try_issue_bond(&alice, &config, &0);
            assert_eq!(result, Err(Ok(BondError::Unauthorized)));

            let result = contracts.oc_client.try_submit_report(
                &bob,
                &project_id,
                &1000u64,
                &2000u64,
                &100_000i128,
                &Symbol::new(&env, "verra_vcs"),
                &make_ipfs_hash(&env, 1),
                &0,
            );
            assert_eq!(result, Err(Ok(OracleError::ProviderNotFound)));
        }

        #[test]
        fn test_unauthorized_oracle_operations() {
            let env = Env::default();
            env.mock_all_auths();

            let admin = Address::generate(&env);
            let alice = Address::generate(&env);
            let rogue = Address::generate(&env);
            let contracts = deploy_contracts(&env, &admin);

            let project_id = make_project_id(&env, 1);

            let pid = contracts.pr_client.register_project(
                &alice,
                &make_ipfs_hash(&env, 1),
                &Symbol::new(&env, "VCS"),
                &Symbol::new(&env, "US"),
                &0,
            );
            contracts.pr_client.approve_project(&admin, &pid, &0);

            let result = contracts.oc_client.try_submit_report(
                &rogue,
                &project_id,
                &1000u64,
                &2000u64,
                &100_000i128,
                &Symbol::new(&env, "verra_vcs"),
                &make_ipfs_hash(&env, 1),
                &0,
            );
            assert_eq!(result, Err(Ok(OracleError::ProviderNotFound)));
        }
    }
}
