use orangecoding_agent::harness::types::{HarnessConfig, MissionContract, ReviewGatePolicy};

#[test]
fn 任务契约会保留成功条件() {
    let contract = MissionContract::new(
        "实现 Harness 对齐层".into(),
        vec!["出现检查点".into(), "重大变更触发门控".into()],
        ReviewGatePolicy::MajorPlanChange,
        HarnessConfig::default(),
    );

    assert_eq!(contract.success_criteria.len(), 2);
}
