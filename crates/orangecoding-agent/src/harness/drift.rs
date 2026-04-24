use crate::harness::types::{HarnessAction, MissionContract, StepOutcome};

pub fn classify_outcome(_contract: &MissionContract, _outcome: &StepOutcome) -> HarnessAction {
    HarnessAction::Continue
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::harness::types::{HarnessConfig, MissionContract, ReviewGatePolicy, StepOutcome};
    use crate::harness::HarnessAction;

    fn contract() -> MissionContract {
        MissionContract::new(
            "实现 Harness 对齐层".into(),
            vec!["保持主目标".into()],
            ReviewGatePolicy::MajorPlanChange,
            HarnessConfig::default(),
        )
    }

    #[test]
    fn 偏航时必须升级而不是继续() {
        let outcome = StepOutcome {
            summary: "顺手去修一个无关 UI bug".into(),
            touched_files: vec!["crates/orangecoding-tui/src/app.rs".into()],
            decisions: vec!["先解决 UI 再回来".into()],
            rationale: "这个问题也挺重要".into(),
            blockers: vec![],
            proposed_plan_change: None,
        };

        let decision = classify_outcome(&contract(), &outcome);
        assert!(matches!(decision, HarnessAction::Escalate { .. }));
    }

    #[test]
    fn 重大计划变更时进入受控重规划() {
        let outcome = StepOutcome {
            summary: "需要把目标从 harness 改为全量 workflow 重写".into(),
            touched_files: vec![],
            decisions: vec!["放弃渐进接入".into()],
            rationale: "当前方案需要完全改写".into(),
            blockers: vec![],
            proposed_plan_change: Some("将首版范围扩大到重写所有 workflow".into()),
        };

        let decision = classify_outcome(&contract(), &outcome);
        assert!(matches!(decision, HarnessAction::Replan { .. }));
    }
}
