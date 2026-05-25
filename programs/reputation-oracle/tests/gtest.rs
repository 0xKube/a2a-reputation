use reputation_oracle_client::{
    ApplicationStatusV2, OracleStatus, ParticipantStatusV2, PredictionStatus,
    ReputationOracleClient, ReputationOracleClientCtors, ReputationSignals, ReputationSignalsV2,
    reputation_oracle::*,
};
use sails_rs::{client::*, gtest::*, prelude::ActorId};

const ACTOR_ID: u64 = 42;
const UNAUTHORIZED_ACTOR_ID: u64 = 7;
const VARA_UNIT: u128 = 1_000_000_000_000;
const PREDICTION_MIN_STAKE: u128 = 10 * VARA_UNIT;
const PREDICTION_STAKE: u128 = 1_000 * VARA_UNIT;
const PREDICTION_MAX_STAKE: u128 = 10_000 * VARA_UNIT;

async fn deploy_oracle()
-> sails_rs::client::Actor<reputation_oracle_client::ReputationOracleClientProgram, GtestEnv> {
    deploy_oracle_with_env().await.0
}

async fn deploy_oracle_with_env() -> (
    sails_rs::client::Actor<reputation_oracle_client::ReputationOracleClientProgram, GtestEnv>,
    GtestEnv,
) {
    let system = System::new();
    system.init_logger_with_default_filter("gwasm=debug,gtest=info,sails_rs=debug");
    system.mint_to(ACTOR_ID, 100_000_000_000_000_000_000);
    system.mint_to(UNAUTHORIZED_ACTOR_ID, 100_000_000_000_000_000_000);
    let program_code_id = system.submit_code(reputation_oracle::WASM_BINARY);

    let env = GtestEnv::new(system, ACTOR_ID.into());

    let program = env
        .deploy::<reputation_oracle_client::ReputationOracleClientProgram>(
            program_code_id,
            b"salt".to_vec(),
        )
        .create()
        .await
        .unwrap();

    (program, env)
}

#[tokio::test]
async fn score_agent_returns_fixture_report() {
    let program = deploy_oracle().await;
    let service_client = program.reputation_oracle();

    let report = service_client
        .score_agent("@registry-helper".to_string())
        .await
        .unwrap();

    assert_eq!(report.subject, "@registry-helper");
    assert_eq!(report.report_version, 2);
    assert_eq!(report.scores.verdict, "review");
    assert_eq!(report.scores.trust_score, 48);
    assert_eq!(report.scores.spam_risk, 0);
    assert_eq!(report.signals.incoming_unique_callers, 3);
    assert!(report.signals.has_identity_card);
    assert!(report.signals.has_verified_social_proof);
    assert_eq!(report.integration_decision.action, "manual_review");
    assert_eq!(report.integration_decision.risk_level, "medium");
    assert_eq!(report.integration_decision.max_attestation_weight, 20);
}

#[tokio::test]
async fn score_agent_flags_circular_spam_risk() {
    let program = deploy_oracle().await;
    let service_client = program.reputation_oracle();

    let report = service_client
        .score_agent("@loop-farm".to_string())
        .await
        .unwrap();

    assert_eq!(report.scores.verdict, "avoid_or_wait");
    assert_eq!(report.scores.spam_risk, 68);
    assert_eq!(report.signals.circular_call_signals, 2);
    assert_eq!(report.signals.missing_required_metadata, 3);
    assert_eq!(report.signals.negative_attestations, 1);
    assert_eq!(report.integration_decision.action, "wait");
    assert_eq!(report.integration_decision.max_attestation_weight, 0);
}

#[tokio::test]
async fn score_agent_v2_returns_component_scores() {
    let program = deploy_oracle().await;
    let service_client = program.reputation_oracle();

    let report = service_client
        .score_agent_v_2(
            "  @useful-router  ".to_string(),
            ReputationSignalsV2 {
                inbound_unique_participants: 10,
                inbound_call_count: 47,
                outbound_unique_participants: 6,
                outbound_call_count: 30,
                valid_participant_inbound_count: 0,
                valid_participant_outbound_count: 0,
                non_participant_call_count: 0,
                self_loop_call_count: 0,
                same_owner_call_count: 0,
                mention_count: 1,
                messages_sent: 1,
                posts_active: 2,
                has_clear_board_description: true,
                has_identity_card: true,
                has_actual_github_repo: true,
                has_skills_url: true,
                has_idl_url: true,
                has_frontend_url: false,
                handle_claimed: true,
                application_status: ApplicationStatusV2::Submitted,
                participant_status: ParticipantStatusV2::ParticipantLike,
                call_graph_density_bps: 1_000,
                low_diversity_volume_count: 0,
                reciprocal_farming_signals: 0,
                positive_third_party_attestations: 0,
                negative_third_party_attestations: 0,
                metrics_updated_at: 1,
                identity_updated_at: 1,
                registered_at: 1,
            },
            vec![
                " observed:unique-inbound:10 ".to_string(),
                "observed:unique-inbound:10".to_string(),
                "unknown:self-loop-filtering".to_string(),
            ],
        )
        .await
        .unwrap();

    assert_eq!(report.subject, "@useful-router");
    assert_eq!(report.report_version, 3);
    assert_eq!(report.scores.ecosystem_value_score, 97);
    assert_eq!(report.scores.real_integration_score, 83);
    assert_eq!(report.scores.counterparty_diversity_score, 100);
    assert_eq!(report.scores.identity_provenance_score, 100);
    assert_eq!(report.scores.demo_readiness_score, 75);
    assert_eq!(report.scores.spam_risk, 0);
    assert_eq!(report.scores.safety_score, 100);
    assert_eq!(report.scores.confidence_score, 95);
    assert_eq!(report.scores.overall_score, 91);
    assert_eq!(report.scores.verdict, "recommended");
    assert_eq!(report.evidence.len(), 2);
}

#[tokio::test]
async fn score_agent_v2_keeps_polished_metadata_only_in_review() {
    let program = deploy_oracle().await;
    let service_client = program.reputation_oracle();

    let report = service_client
        .score_agent_v_2(
            "@metadata-only".to_string(),
            ReputationSignalsV2 {
                inbound_unique_participants: 0,
                inbound_call_count: 0,
                outbound_unique_participants: 0,
                outbound_call_count: 0,
                valid_participant_inbound_count: 0,
                valid_participant_outbound_count: 0,
                non_participant_call_count: 0,
                self_loop_call_count: 0,
                same_owner_call_count: 0,
                mention_count: 0,
                messages_sent: 0,
                posts_active: 1,
                has_clear_board_description: true,
                has_identity_card: true,
                has_actual_github_repo: true,
                has_skills_url: true,
                has_idl_url: true,
                has_frontend_url: false,
                handle_claimed: true,
                application_status: ApplicationStatusV2::Submitted,
                participant_status: ParticipantStatusV2::ParticipantLike,
                call_graph_density_bps: 0,
                low_diversity_volume_count: 0,
                reciprocal_farming_signals: 0,
                positive_third_party_attestations: 0,
                negative_third_party_attestations: 0,
                metrics_updated_at: 1,
                identity_updated_at: 1,
                registered_at: 1,
            },
            vec!["unknown:frontend-url".to_string()],
        )
        .await
        .unwrap();

    assert_eq!(report.scores.real_integration_score, 0);
    assert_eq!(report.scores.overall_score, 56);
    assert_eq!(report.scores.verdict, "review");
}

#[tokio::test]
async fn score_agent_rejects_blank_and_trims_subject() {
    let program = deploy_oracle().await;
    let service_client = program.reputation_oracle();

    let blank_result = service_client.score_agent("   ".to_string()).await;
    assert!(blank_result.is_err());

    let report = service_client
        .score_agent("  @registry-helper  ".to_string())
        .await
        .unwrap();

    assert_eq!(report.subject, "@registry-helper");
    assert_eq!(report.scores.verdict, "review");
    assert_eq!(report.evidence[0], "board:identity");
}

#[tokio::test]
async fn compare_agents_ranks_fixture_candidates() {
    let program = deploy_oracle().await;
    let service_client = program.reputation_oracle();

    let comparison = service_client
        .compare_agents(
            vec![
                "@registry-helper".to_string(),
                "@quiet-indexer".to_string(),
                "@loop-farm".to_string(),
            ],
            "agent-services".to_string(),
            "safe integration target".to_string(),
        )
        .await
        .unwrap();

    assert_eq!(comparison.track, "agent-services");
    assert_eq!(comparison.need, "safe integration target");
    assert_eq!(comparison.winner.subject, "@registry-helper");
    assert_eq!(comparison.winner.trust_score, 48);
    assert_eq!(
        comparison.winner.reason,
        "candidate is usable but needs manual review"
    );
    assert_eq!(comparison.candidates.len(), 3);
    assert_eq!(comparison.candidates[0].subject, "@registry-helper");
    assert_eq!(comparison.candidates[1].subject, "@quiet-indexer");
    assert_eq!(comparison.candidates[2].subject, "@loop-farm");
    assert_eq!(comparison.decision.winner, "@registry-helper");
    assert_eq!(
        comparison.decision.runner_up,
        Some("@quiet-indexer".to_string())
    );
    assert_eq!(comparison.decision.trust_margin, 18);
    assert_eq!(comparison.decision.spam_risk_advantage, 0);
    assert_eq!(
        comparison.decision.summary,
        "@registry-helper leads @quiet-indexer by 18 trust points and 0 spam-risk points"
    );
}

#[tokio::test]
async fn compare_agents_rejects_single_or_blank_subject_lists() {
    let program = deploy_oracle().await;
    let service_client = program.reputation_oracle();

    let single_result = service_client
        .compare_agents(
            vec!["@registry-helper".to_string()],
            "agent-services".to_string(),
            "safe integration target".to_string(),
        )
        .await;
    assert!(single_result.is_err());

    let blank_result = service_client
        .compare_agents(
            vec!["@registry-helper".to_string(), " ".to_string()],
            "agent-services".to_string(),
            "safe integration target".to_string(),
        )
        .await;
    assert!(blank_result.is_err());
}

#[tokio::test]
async fn compare_agents_trims_subjects_before_scoring() {
    let program = deploy_oracle().await;
    let service_client = program.reputation_oracle();

    let comparison = service_client
        .compare_agents(
            vec![
                "  @loop-farm  ".to_string(),
                "  @registry-helper  ".to_string(),
            ],
            "agent-services".to_string(),
            "safe integration target".to_string(),
        )
        .await
        .unwrap();

    assert_eq!(comparison.winner.input, "@registry-helper");
    assert_eq!(comparison.winner.subject, "@registry-helper");
    assert_eq!(comparison.candidates[1].input, "@loop-farm");
    assert_eq!(comparison.candidates[1].subject, "@loop-farm");
}

#[tokio::test]
async fn recommend_agents_returns_ranked_safe_targets() {
    let program = deploy_oracle().await;
    let service_client = program.reputation_oracle();

    let recommendations = service_client
        .recommend_agents(
            "agent-services".to_string(),
            "safe integration target".to_string(),
            3,
            35,
            false,
        )
        .await
        .unwrap();

    assert_eq!(recommendations.len(), 1);
    let recommendation = &recommendations[0];
    assert_eq!(recommendation.subject, "@registry-helper");
    assert_eq!(recommendation.track, "agent-services");
    assert_eq!(recommendation.need, "safe integration target");
    assert_eq!(recommendation.verdict, "review");
    assert_eq!(recommendation.trust_score, 48);
    assert_eq!(recommendation.confidence, 80);
    assert_eq!(recommendation.spam_risk, 0);
    assert_eq!(
        recommendation.reason,
        "usable candidate with incomplete confidence or mixed signals"
    );
    assert_eq!(
        recommendation.explanation.summary,
        "usable candidate with incomplete confidence or mixed signals"
    );
    assert_eq!(
        recommendation.explanation.positive_signals,
        vec![
            "published identity card".to_string(),
            "declared external metadata".to_string(),
            "3 unique inbound callers".to_string(),
        ]
    );
    assert!(recommendation.explanation.negative_signals.is_empty());
    assert_eq!(recommendation.report.subject, "@registry-helper");
    assert_eq!(recommendation.report.scores.trust_score, 48);
}

#[tokio::test]
async fn recommend_agents_can_include_watch_candidates() {
    let program = deploy_oracle().await;
    let service_client = program.reputation_oracle();

    let recommendations = service_client
        .recommend_agents(
            "agent-services".to_string(),
            "safe integration target".to_string(),
            3,
            0,
            true,
        )
        .await
        .unwrap();

    assert_eq!(recommendations.len(), 3);
    assert_eq!(recommendations[0].subject, "@registry-helper");
    assert_eq!(recommendations[1].subject, "@quiet-indexer");
    assert_eq!(recommendations[2].subject, "@loop-farm");
    assert_eq!(recommendations[2].verdict, "avoid_or_wait");
    assert_eq!(
        recommendations[2].explanation.negative_signals,
        vec![
            "spam risk 68".to_string(),
            "2 circular call signals".to_string(),
            "3 missing metadata fields".to_string(),
        ]
    );
}

#[tokio::test]
async fn submit_integration_signal_rewards_useful_evidence() {
    let program = deploy_oracle().await;
    let mut service_client = program.reputation_oracle();

    let pool = service_client
        .fund_reward_pool()
        .with_value(500)
        .await
        .unwrap();
    assert_eq!(pool.value_added, 500);
    assert_eq!(pool.reward_pool_balance, 500);

    let receipt = service_client
        .submit_integration_signal(
            "@scout-agent".to_string(),
            "@registry-helper".to_string(),
            "@router-agent".to_string(),
            "0xuseful-evidence".to_string(),
            80,
            500,
        )
        .with_value(1_000)
        .await
        .unwrap();

    assert_eq!(receipt.agent, "@scout-agent");
    assert_eq!(receipt.subject, "@registry-helper");
    assert!(matches!(receipt.verdict.as_str(), "accepted" | "rewarded"));
    assert!(receipt.quality_score >= 35);
    assert!(receipt.points_delta > 0);
    assert_eq!(receipt.value_attached, 1_000);
    assert_eq!(receipt.value_rewarded, 1_500);
    assert_eq!(receipt.value_slashed, 0);
    assert_eq!(receipt.reward_pool_balance, 0);

    let profile = service_client
        .economic_profile("@scout-agent".to_string())
        .await
        .unwrap();
    assert_eq!(profile.oracle_points, receipt.profile_points);
    assert_eq!(profile.useful_signals, 1);
    assert_eq!(profile.spam_signals, 0);
    assert_eq!(profile.total_value_staked, 1_000);
    assert_eq!(profile.total_value_rewarded, 1_500);
    assert_eq!(profile.total_value_slashed, 0);
}

#[tokio::test]
async fn submit_integration_signal_penalizes_self_loops() {
    let program = deploy_oracle().await;
    let mut service_client = program.reputation_oracle();

    let receipt = service_client
        .submit_integration_signal(
            "@loop-farm".to_string(),
            "@loop-farm".to_string(),
            "@loop-farm".to_string(),
            "0xloop".to_string(),
            100,
            900,
        )
        .with_value(1_000)
        .await
        .unwrap();

    assert_eq!(receipt.verdict, "slashed");
    assert!(receipt.points_delta < 0);
    assert_eq!(receipt.value_attached, 1_000);
    assert_eq!(receipt.value_rewarded, 0);
    assert_eq!(receipt.value_slashed, 1_000);
    assert!(receipt.reason.contains("self-loop"));

    let profile = service_client
        .economic_profile("@loop-farm".to_string())
        .await
        .unwrap();
    assert_eq!(profile.spam_signals, 1);
    assert!(profile.oracle_points < 0);
    assert_eq!(profile.total_value_staked, 1_000);
    assert_eq!(profile.total_value_rewarded, 0);
    assert_eq!(profile.total_value_slashed, 1_000);
}

#[tokio::test]
async fn economic_leaderboard_ranks_oracle_points() {
    let program = deploy_oracle().await;
    let mut service_client = program.reputation_oracle();

    service_client
        .submit_integration_signal(
            "@good-scout".to_string(),
            "@registry-helper".to_string(),
            "@router-agent".to_string(),
            "0xgood".to_string(),
            90,
            1_000,
        )
        .with_value(1_000)
        .await
        .unwrap();
    service_client
        .submit_integration_signal(
            "@bad-scout".to_string(),
            "@loop-farm".to_string(),
            "@loop-farm".to_string(),
            "0xbad".to_string(),
            90,
            1_000,
        )
        .with_value(1_000)
        .await
        .unwrap();

    let leaderboard = service_client.economic_leaderboard(2).await.unwrap();
    assert_eq!(leaderboard.len(), 2);
    assert_eq!(leaderboard[0].agent, "@good-scout");
    assert_eq!(leaderboard[1].agent, "@bad-scout");
}

#[tokio::test]
async fn usage_prediction_market_pays_accurate_window_delta_forecasts() {
    let program = deploy_oracle().await;
    let mut service_client = program.reputation_oracle();

    service_client
        .fund_reward_pool()
        .with_value(PREDICTION_STAKE / 2)
        .await
        .unwrap();
    let opened = service_client
        .open_usage_prediction(
            1,
            "@registry-helper".to_string(),
            0,
            u64::MAX,
            100,
            "0xprediction-source".to_string(),
        )
        .with_value(PREDICTION_STAKE)
        .await
        .unwrap();
    assert_eq!(opened.stake, PREDICTION_STAKE);
    assert_eq!(opened.effective_stake, PREDICTION_STAKE);
    assert_eq!(opened.late_penalty_bps, 0);
    assert_eq!(opened.predicted_delta_calls, 100);

    let settled = service_client
        .settle_usage_prediction(
            opened.position_id.clone(),
            105,
            "0xsnapshot-105".to_string(),
        )
        .await
        .unwrap();
    assert_eq!(settled.status, PredictionStatus::Won);
    assert_eq!(settled.settlement_snapshot_hash, "0xsnapshot-105");
    assert!(settled.error_bps <= 1_000);
    assert_eq!(settled.effective_stake, PREDICTION_STAKE);
    assert_eq!(settled.late_penalty_bps, 0);
    assert_eq!(settled.payout, PREDICTION_STAKE + PREDICTION_STAKE / 2);
    assert_eq!(settled.reward_pool_balance, 0);
}

#[tokio::test]
async fn usage_prediction_settlement_uses_window_delta_not_cumulative_total() {
    let program = deploy_oracle().await;
    let mut service_client = program.reputation_oracle();

    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    let opened = service_client
        .open_usage_prediction(
            9,
            "@registry-helper".to_string(),
            now_ms.saturating_sub(1_000),
            now_ms.saturating_add(10_800_000),
            120,
            "0xwindow-delta-semantics".to_string(),
        )
        .with_value(PREDICTION_STAKE)
        .await
        .unwrap();

    let settled = service_client
        .settle_usage_prediction(
            opened.position_id.clone(),
            118,
            "0xsnapshot-window-delta-118-not-cumulative-50118".to_string(),
        )
        .await
        .unwrap();

    assert_eq!(settled.status, PredictionStatus::Won);
    assert_eq!(settled.predicted_delta_calls, 120);
    assert_eq!(settled.actual_delta_calls, 118);
    assert!(settled.error_bps <= 1_000);
}

#[tokio::test]
async fn usage_prediction_market_slashes_bad_forecasts_into_pool() {
    let program = deploy_oracle().await;
    let mut service_client = program.reputation_oracle();

    let opened = service_client
        .open_usage_prediction(
            1,
            "@registry-helper".to_string(),
            0,
            u64::MAX,
            100,
            "0xprediction-source".to_string(),
        )
        .with_value(PREDICTION_STAKE)
        .await
        .unwrap();

    let settled = service_client
        .settle_usage_prediction(
            opened.position_id.clone(),
            200,
            "0xsnapshot-200".to_string(),
        )
        .await
        .unwrap();
    assert_eq!(settled.status, PredictionStatus::Lost);
    assert!(settled.error_bps > 1_000);
    assert_eq!(settled.payout, 0);
    assert_eq!(settled.reward_pool_balance, PREDICTION_STAKE);

    let chunk = service_client
        .export_usage_predictions_chunk(0, 10)
        .await
        .unwrap();
    assert_eq!(chunk.total, 1);
    assert_eq!(chunk.items[0].status, PredictionStatus::Lost);
    assert_eq!(chunk.items[0].actual_delta_calls, Some(200));
    assert_eq!(
        chunk.items[0].settlement_snapshot_hash,
        Some("0xsnapshot-200".to_string())
    );
}

#[tokio::test]
async fn usage_prediction_market_works_without_prefunded_pool() {
    let program = deploy_oracle().await;
    let mut service_client = program.reputation_oracle();

    let opened = service_client
        .open_usage_prediction(
            1,
            "@registry-helper".to_string(),
            0,
            u64::MAX,
            100,
            "0xprediction-no-prefund".to_string(),
        )
        .with_value(PREDICTION_STAKE)
        .await
        .unwrap();

    let settled = service_client
        .settle_usage_prediction(
            opened.position_id.clone(),
            100,
            "0xsnapshot-no-prefund".to_string(),
        )
        .await
        .unwrap();
    assert_eq!(settled.status, PredictionStatus::Won);
    assert_eq!(settled.payout, PREDICTION_STAKE);
    assert_eq!(settled.reward_pool_balance, 0);
}

#[tokio::test]
async fn usage_prediction_late_entries_pay_rising_penalty() {
    let program = deploy_oracle().await;
    let mut service_client = program.reputation_oracle();

    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    let opened = service_client
        .open_usage_prediction(
            7,
            "@registry-helper".to_string(),
            0,
            now_ms.saturating_add(86_400_000),
            100,
            "0xlate-entry".to_string(),
        )
        .with_value(PREDICTION_STAKE)
        .await
        .unwrap();

    assert!(opened.late_penalty_bps >= 4_000);
    assert!(opened.late_penalty_value >= PREDICTION_STAKE * 4 / 10);
    assert!(opened.effective_stake <= PREDICTION_STAKE * 6 / 10);
    assert_eq!(opened.reward_pool_balance, opened.late_penalty_value);

    let settled = service_client
        .settle_usage_prediction(
            opened.position_id.clone(),
            100,
            "0xsnapshot-100".to_string(),
        )
        .await
        .unwrap();
    assert_eq!(settled.status, PredictionStatus::Won);
    assert_eq!(settled.effective_stake, opened.effective_stake);
    assert_eq!(
        settled.payout,
        opened.effective_stake + opened.effective_stake / 2
    );
    assert_eq!(
        settled.reward_pool_balance,
        opened.late_penalty_value - opened.effective_stake / 2
    );
}


#[tokio::test]
async fn usage_prediction_rejects_zero_stake_and_invalid_windows() {
    let program = deploy_oracle().await;
    let mut service_client = program.reputation_oracle();

    let zero_stake = service_client
        .open_usage_prediction(
            1,
            "@registry-helper".to_string(),
            0,
            u64::MAX,
            100,
            "0xzero-stake".to_string(),
        )
        .await;
    assert!(zero_stake.is_err());

    let below_min_stake = service_client
        .open_usage_prediction(
            1,
            "@registry-helper".to_string(),
            0,
            u64::MAX,
            100,
            "0xbelow-min-stake".to_string(),
        )
        .with_value(PREDICTION_MIN_STAKE - 1)
        .await;
    assert!(below_min_stake.is_err());

    let max_stake = service_client
        .open_usage_prediction(
            1,
            "@registry-helper".to_string(),
            0,
            u64::MAX,
            100,
            "0xmax-stake".to_string(),
        )
        .with_value(PREDICTION_MAX_STAKE)
        .await;
    assert!(max_stake.is_ok());

    let above_max_stake = service_client
        .open_usage_prediction(
            1,
            "@registry-helper".to_string(),
            0,
            u64::MAX,
            100,
            "0xabove-max-stake".to_string(),
        )
        .with_value(PREDICTION_MAX_STAKE + 1)
        .await;
    assert!(above_max_stake.is_err());

    let invalid_window = service_client
        .open_usage_prediction(
            1,
            "@registry-helper".to_string(),
            10,
            10,
            100,
            "0xinvalid-window".to_string(),
        )
        .with_value(PREDICTION_STAKE)
        .await;
    assert!(invalid_window.is_err());

    let closed_window = service_client
        .open_usage_prediction(
            1,
            "@registry-helper".to_string(),
            0,
            1,
            100,
            "0xclosed-window".to_string(),
        )
        .with_value(PREDICTION_STAKE)
        .await;
    assert!(closed_window.is_err());
}

#[tokio::test]
async fn usage_prediction_settlement_is_owner_or_operator_only_and_single_use() {
    let program = deploy_oracle().await;
    let mut service_client = program.reputation_oracle();

    let opened = service_client
        .open_usage_prediction(
            1,
            "@registry-helper".to_string(),
            0,
            u64::MAX,
            100,
            "0xauth-and-single-use".to_string(),
        )
        .with_value(PREDICTION_STAKE)
        .await
        .unwrap();

    let unauthorized = service_client
        .settle_usage_prediction(
            opened.position_id.clone(),
            100,
            "0xunauthorized-snapshot".to_string(),
        )
        .with_actor_id(UNAUTHORIZED_ACTOR_ID.into())
        .await;
    assert!(unauthorized.is_err());

    let settled = service_client
        .settle_usage_prediction(
            opened.position_id.clone(),
            100,
            "0xowner-snapshot".to_string(),
        )
        .await
        .unwrap();
    assert_eq!(settled.status, PredictionStatus::Won);
    assert_eq!(settled.payout, PREDICTION_STAKE);

    let duplicate = service_client
        .settle_usage_prediction(
            opened.position_id.clone(),
            100,
            "0xduplicate-snapshot".to_string(),
        )
        .await;
    assert!(duplicate.is_err());
}

#[tokio::test]
async fn usage_prediction_bonus_is_capped_by_available_reward_pool() {
    let program = deploy_oracle().await;
    let mut service_client = program.reputation_oracle();

    service_client
        .fund_reward_pool()
        .with_value(100)
        .await
        .unwrap();
    let opened = service_client
        .open_usage_prediction(
            1,
            "@registry-helper".to_string(),
            0,
            u64::MAX,
            100,
            "0xunderfunded-bonus".to_string(),
        )
        .with_value(PREDICTION_STAKE)
        .await
        .unwrap();

    let settled = service_client
        .settle_usage_prediction(
            opened.position_id.clone(),
            100,
            "0xunderfunded-snapshot".to_string(),
        )
        .await
        .unwrap();
    assert_eq!(settled.status, PredictionStatus::Won);
    assert_eq!(settled.payout, PREDICTION_STAKE + 100);
    assert_eq!(settled.reward_pool_balance, 0);
}

#[tokio::test]
async fn usage_prediction_multiple_winners_cannot_overdraw_reward_pool() {
    let program = deploy_oracle().await;
    let mut service_client = program.reputation_oracle();

    service_client
        .fund_reward_pool()
        .with_value(PREDICTION_STAKE / 2)
        .await
        .unwrap();
    let first = service_client
        .open_usage_prediction(
            1,
            "@registry-helper".to_string(),
            0,
            u64::MAX,
            100,
            "0xwinner-one".to_string(),
        )
        .with_value(PREDICTION_STAKE)
        .await
        .unwrap();
    let second = service_client
        .open_usage_prediction(
            1,
            "@registry-helper".to_string(),
            0,
            u64::MAX,
            100,
            "0xwinner-two".to_string(),
        )
        .with_value(PREDICTION_STAKE)
        .await
        .unwrap();

    let first_settled = service_client
        .settle_usage_prediction(first.position_id.clone(), 100, "0xwinner-one-snapshot".to_string())
        .await
        .unwrap();
    assert_eq!(first_settled.payout, PREDICTION_STAKE + PREDICTION_STAKE / 2);
    assert_eq!(first_settled.reward_pool_balance, 0);

    let second_settled = service_client
        .settle_usage_prediction(second.position_id.clone(), 100, "0xwinner-two-snapshot".to_string())
        .await
        .unwrap();
    assert_eq!(second_settled.payout, PREDICTION_STAKE);
    assert_eq!(second_settled.reward_pool_balance, 0);
}

#[tokio::test]
async fn usage_prediction_zero_actual_edge_cases_are_bounded() {
    let program = deploy_oracle().await;
    let mut service_client = program.reputation_oracle();

    let exact_zero = service_client
        .open_usage_prediction(
            1,
            "@registry-helper".to_string(),
            0,
            u64::MAX,
            0,
            "0xzero-zero".to_string(),
        )
        .with_value(PREDICTION_STAKE)
        .await
        .unwrap();
    let exact_zero_settled = service_client
        .settle_usage_prediction(exact_zero.position_id.clone(), 0, "0xzero-zero-snapshot".to_string())
        .await
        .unwrap();
    assert_eq!(exact_zero_settled.status, PredictionStatus::Won);
    assert_eq!(exact_zero_settled.error_bps, 0);
    assert_eq!(exact_zero_settled.payout, PREDICTION_STAKE);

    let off_by_one = service_client
        .open_usage_prediction(
            1,
            "@registry-helper".to_string(),
            0,
            u64::MAX,
            0,
            "0xzero-one".to_string(),
        )
        .with_value(PREDICTION_STAKE)
        .await
        .unwrap();
    let off_by_one_settled = service_client
        .settle_usage_prediction(off_by_one.position_id.clone(), 1, "0xzero-one-snapshot".to_string())
        .await
        .unwrap();
    assert_eq!(off_by_one_settled.status, PredictionStatus::Lost);
    assert_eq!(off_by_one_settled.error_bps, 10_000);
    assert_eq!(off_by_one_settled.payout, 0);
}

#[tokio::test]
async fn usage_prediction_capacity_does_not_evict_open_positions() {
    let program = deploy_oracle().await;
    let mut service_client = program.reputation_oracle();

    for index in 0..128 {
        service_client
            .open_usage_prediction(
                1,
                "@registry-helper".to_string(),
                0,
                u64::MAX,
                index,
                format!("0xcapacity-open-{index}"),
            )
            .with_value(PREDICTION_MIN_STAKE)
            .await
            .unwrap();
    }

    let overflow = service_client
        .open_usage_prediction(
            1,
            "@registry-helper".to_string(),
            0,
            u64::MAX,
            129,
            "0xcapacity-overflow".to_string(),
        )
        .with_value(PREDICTION_MIN_STAKE)
        .await;
    assert!(overflow.is_err());

    let chunk = service_client
        .export_usage_predictions_chunk(0, 200)
        .await
        .unwrap();
    assert_eq!(chunk.total, 128);
    assert!(chunk.items.iter().all(|position| matches!(position.status, PredictionStatus::Open)));
}

#[tokio::test]
async fn usage_prediction_concurrent_predictions_across_multiple_subjects() {
    let program = deploy_oracle().await;
    let mut service_client = program.reputation_oracle();

    let subjects = vec!["@subject-a", "@subject-b", "@subject-c", "@subject-d", "@subject-e"];
    let mut positions = vec![];

    for (i, subject) in subjects.iter().enumerate() {
        let opened = service_client
            .open_usage_prediction(
                1,
                subject.to_string(),
                0,
                u64::MAX,
                (i + 1) as u32 * 100,
                format!("0xconcurrent-{i}"),
            )
            .with_value(PREDICTION_STAKE)
            .await
            .unwrap();
        positions.push(opened);
    }

    assert_eq!(positions.len(), 5);
    for (i, pos) in positions.iter().enumerate() {
        assert_eq!(pos.subject, subjects[i]);
        assert_eq!(pos.predicted_delta_calls, (i + 1) as u32 * 100);
    }

    for (i, pos) in positions.iter().enumerate() {
        let settled = service_client
            .settle_usage_prediction(
                pos.position_id.clone(),
                (i + 1) as u32 * 100,
                format!("0xsnapshot-{i}"),
            )
            .await
            .unwrap();
        assert_eq!(settled.status, PredictionStatus::Won);
        assert_eq!(settled.actual_delta_calls, (i + 1) as u32 * 100);
        assert_eq!(settled.predicted_delta_calls, (i + 1) as u32 * 100);
    }

    let chunk = service_client
        .export_usage_predictions_chunk(0, 100)
        .await
        .unwrap();
    assert_eq!(chunk.total, 5);
    assert!(chunk.items.iter().all(|position| matches!(position.status, PredictionStatus::Won)));
}

#[tokio::test]
async fn usage_prediction_can_roll_to_next_three_hour_epoch_after_settlement() {
    let (program, env) = deploy_oracle_with_env().await;
    let mut service_client = program.reputation_oracle();

    let three_hours_ms = 3 * 60 * 60 * 1_000;
    let first_window_end = env.system().block_timestamp().saturating_add(9_000);
    let first_window_start = first_window_end.saturating_sub(three_hours_ms);
    let second_window_start = first_window_end;
    let second_window_end = second_window_start.saturating_add(three_hours_ms);

    let first = service_client
        .open_usage_prediction(
            1,
            "@registry-helper".to_string(),
            first_window_start,
            first_window_end,
            120,
            "0xepoch-1-window-delta".to_string(),
        )
        .with_value(PREDICTION_STAKE)
        .await
        .unwrap();
    env.run_next_block();
    env.run_next_block();
    let first_settled = service_client
        .settle_usage_prediction(first.position_id.clone(), 118, "0xepoch-1-snapshot".to_string())
        .await
        .unwrap();
    assert_eq!(first_settled.status, PredictionStatus::Won);

    let second = service_client
        .open_usage_prediction(
            2,
            "@registry-helper".to_string(),
            second_window_start,
            second_window_end,
            90,
            "0xepoch-2-window-delta".to_string(),
        )
        .with_value(PREDICTION_STAKE)
        .await
        .unwrap();
    assert_ne!(first.position_id, second.position_id);
    assert_eq!(second.epoch_id, 2);
    assert_eq!(second.window_start_ms, second_window_start);
    assert_eq!(second.window_end_ms, second_window_end);

    let early_second_settlement = service_client
        .settle_usage_prediction(second.position_id.clone(), 92, "0xepoch-2-snapshot".to_string())
        .await;
    assert!(early_second_settlement.is_err());

    let chunk = service_client
        .export_usage_predictions_chunk(0, 10)
        .await
        .unwrap();
    assert_eq!(chunk.total, 2);
    assert_eq!(chunk.items[0].epoch_id, 1);
    assert_eq!(chunk.items[1].epoch_id, 2);
    assert!(matches!(chunk.items[0].status, PredictionStatus::Won));
    assert!(matches!(chunk.items[1].status, PredictionStatus::Open));
}

#[tokio::test]
async fn export_signal_receipts_chunks_submissions() {
    let program = deploy_oracle().await;
    let mut service_client = program.reputation_oracle();

    service_client
        .submit_integration_signal(
            "@scout-a".to_string(),
            "@registry-helper".to_string(),
            "@router-agent".to_string(),
            "0xa".to_string(),
            40,
            100,
        )
        .with_value(PREDICTION_STAKE)
        .await
        .unwrap();
    service_client
        .submit_integration_signal(
            "@scout-b".to_string(),
            "@quiet-indexer".to_string(),
            "@research-agent".to_string(),
            "0xb".to_string(),
            40,
            100,
        )
        .with_value(PREDICTION_STAKE)
        .await
        .unwrap();

    let chunk = service_client
        .export_signal_receipts_chunk(0, 1)
        .await
        .unwrap();
    assert_eq!(chunk.total, 2);
    assert_eq!(chunk.items.len(), 1);
    assert_eq!(chunk.next_cursor, 1);
    assert!(!chunk.done);
}

#[tokio::test]
async fn read_model_policy_exposes_owner_gate() {
    let program = deploy_oracle().await;
    let service_client = program.reputation_oracle();

    let policy = service_client.read_model_policy().await.unwrap();

    assert_eq!(policy.owner, ACTOR_ID.into());
    assert_eq!(policy.caller, ACTOR_ID.into());
    assert!(policy.caller_can_upsert);
    assert_eq!(policy.operator_count, 0);
    assert_eq!(policy.max_operators, 8);
    assert_eq!(policy.max_read_models, 32);
    assert_eq!(policy.max_evidence_labels, 12);
    assert_eq!(policy.max_evidence_label_chars, 96);
}

#[tokio::test]
async fn migration_config_exposes_status_bounds_and_counts() {
    let program = deploy_oracle().await;
    let mut service_client = program.reputation_oracle();

    let config = service_client.export_migration_config().await.unwrap();
    assert_eq!(config.owner, ACTOR_ID.into());
    assert_eq!(config.status, OracleStatus::Active);
    assert!(!config.read_only);
    assert_eq!(config.operator_count, 0);
    assert_eq!(config.read_model_count, 0);
    assert_eq!(config.attestation_count, 0);
    assert_eq!(config.bounds.max_operators, 8);
    assert_eq!(config.bounds.max_read_models, 32);
    assert_eq!(config.bounds.max_attestations, 64);
    assert_eq!(config.bounds.max_evidence_labels, 12);
    assert_eq!(config.bounds.max_evidence_label_chars, 96);
    assert_eq!(config.bounds.max_export_chunk, 64);
    assert_eq!(config.report_version, 2);
    assert_eq!(config.as_of, "2026-05-20T12:00:00.000Z");

    service_client
        .add_read_model_operator(UNAUTHORIZED_ACTOR_ID.into())
        .await
        .unwrap();
    service_client
        .upsert_read_model(
            "@counted-service".to_string(),
            ReputationSignals {
                incoming_unique_callers: 5,
                outgoing_meaningful_calls: 4,
                chat_board_updates: 3,
                has_identity_card: true,
                has_verified_social_proof: true,
                circular_call_signals: 0,
                missing_required_metadata: 0,
                positive_attestations: 1,
                negative_attestations: 0,
            },
            vec!["registry:application".to_string()],
        )
        .await
        .unwrap();
    service_client
        .record_attestation(
            "@counted-service".to_string(),
            "@reviewer".to_string(),
            "integration_success".to_string(),
            10,
            "0xcounted".to_string(),
        )
        .await
        .unwrap();

    let config = service_client.export_migration_config().await.unwrap();
    let stats = service_client.export_stats().await.unwrap();

    assert_eq!(config.status, OracleStatus::Active);
    assert_eq!(config.operator_count, 1);
    assert_eq!(config.read_model_count, 1);
    assert_eq!(config.attestation_count, 1);
    assert_eq!(stats.operator_count, 1);
    assert_eq!(stats.read_model_count, 1);
    assert_eq!(stats.attestation_count, 1);
}

#[tokio::test]
async fn owner_can_toggle_read_only_and_non_owner_cannot() {
    let program = deploy_oracle().await;
    let mut service_client = program.reputation_oracle();

    assert_eq!(
        service_client.oracle_status().await.unwrap(),
        OracleStatus::Active
    );

    let read_only_receipt = service_client.set_read_only(true).await.unwrap();
    assert_eq!(read_only_receipt.owner, ACTOR_ID.into());
    assert_eq!(read_only_receipt.previous_status, OracleStatus::Active);
    assert_eq!(read_only_receipt.status, OracleStatus::ReadOnly);
    assert!(read_only_receipt.read_only);

    let non_owner_result = service_client
        .set_read_only(false)
        .with_actor_id(UNAUTHORIZED_ACTOR_ID.into())
        .await;
    assert!(non_owner_result.is_err());
    assert_eq!(
        service_client.oracle_status().await.unwrap(),
        OracleStatus::ReadOnly
    );

    let active_receipt = service_client.set_read_only(false).await.unwrap();
    assert_eq!(active_receipt.previous_status, OracleStatus::ReadOnly);
    assert_eq!(active_receipt.status, OracleStatus::Active);
    assert!(!active_receipt.read_only);
}

#[tokio::test]
async fn read_only_rejects_mutations_but_allows_queries_and_exports() {
    let program = deploy_oracle().await;
    let mut service_client = program.reputation_oracle();

    service_client
        .add_read_model_operator(UNAUTHORIZED_ACTOR_ID.into())
        .await
        .unwrap();
    service_client
        .upsert_read_model(
            "@read-only-service".to_string(),
            ReputationSignals {
                incoming_unique_callers: 8,
                outgoing_meaningful_calls: 5,
                chat_board_updates: 4,
                has_identity_card: true,
                has_verified_social_proof: true,
                circular_call_signals: 0,
                missing_required_metadata: 0,
                positive_attestations: 2,
                negative_attestations: 0,
            },
            vec!["registry:application".to_string()],
        )
        .await
        .unwrap();
    let attestation = service_client
        .record_attestation(
            "@read-only-service".to_string(),
            "@reviewer".to_string(),
            "integration_success".to_string(),
            20,
            "0xreadonly".to_string(),
        )
        .await
        .unwrap();

    service_client.set_read_only(true).await.unwrap();

    assert!(
        service_client
            .add_read_model_operator(ActorId::from(99))
            .await
            .is_err()
    );
    assert!(
        service_client
            .remove_read_model_operator(UNAUTHORIZED_ACTOR_ID.into())
            .await
            .is_err()
    );
    assert!(
        service_client
            .upsert_read_model(
                "@blocked-service".to_string(),
                ReputationSignals {
                    incoming_unique_callers: 1,
                    outgoing_meaningful_calls: 1,
                    chat_board_updates: 1,
                    has_identity_card: true,
                    has_verified_social_proof: false,
                    circular_call_signals: 0,
                    missing_required_metadata: 0,
                    positive_attestations: 0,
                    negative_attestations: 0,
                },
                vec!["registry:application".to_string()],
            )
            .await
            .is_err()
    );
    assert!(
        service_client
            .record_attestation(
                "@read-only-service".to_string(),
                "@reviewer-2".to_string(),
                "integration_success".to_string(),
                10,
                "0xblocked".to_string(),
            )
            .await
            .is_err()
    );

    let report = service_client
        .score_agent("@read-only-service".to_string())
        .await
        .unwrap();
    assert_eq!(report.subject, "@read-only-service");
    assert_eq!(report.scores.trust_score, 69);

    let attestations = service_client
        .get_attestations("@read-only-service".to_string())
        .await
        .unwrap();
    assert_eq!(attestations, vec![attestation.clone()]);

    let config = service_client.export_migration_config().await.unwrap();
    let operators = service_client.export_operators().await.unwrap();
    let read_models = service_client
        .export_read_models_chunk(0, 10)
        .await
        .unwrap();
    let exported_attestations = service_client
        .export_attestations_chunk(0, 10)
        .await
        .unwrap();

    assert_eq!(config.status, OracleStatus::ReadOnly);
    assert_eq!(config.operator_count, 1);
    assert_eq!(config.read_model_count, 1);
    assert_eq!(config.attestation_count, 1);
    assert_eq!(operators, vec![ActorId::from(UNAUTHORIZED_ACTOR_ID)]);
    assert_eq!(read_models.items.len(), 1);
    assert_eq!(read_models.items[0].subject, "@read-only-service");
    assert_eq!(exported_attestations.items, vec![attestation]);
}

#[tokio::test]
async fn chunked_exports_are_deterministic_and_complete() {
    let program = deploy_oracle().await;
    let mut service_client = program.reputation_oracle();

    service_client
        .add_read_model_operator(ActorId::from(100))
        .await
        .unwrap();
    service_client
        .add_read_model_operator(ActorId::from(50))
        .await
        .unwrap();

    for subject in ["@gamma-service", "@Alpha-service", "@beta-service"] {
        service_client
            .upsert_read_model(
                subject.to_string(),
                ReputationSignals {
                    incoming_unique_callers: 5,
                    outgoing_meaningful_calls: 4,
                    chat_board_updates: 3,
                    has_identity_card: true,
                    has_verified_social_proof: true,
                    circular_call_signals: 0,
                    missing_required_metadata: 0,
                    positive_attestations: 1,
                    negative_attestations: 0,
                },
                vec![format!("registry:{subject}")],
            )
            .await
            .unwrap();
    }

    for (subject, issuer, hash) in [
        ("@gamma-service", "@issuer-c", "0x03"),
        ("@Alpha-service", "@issuer-a", "0x01"),
        ("@beta-service", "@issuer-b", "0x02"),
    ] {
        service_client
            .record_attestation(
                subject.to_string(),
                issuer.to_string(),
                "integration_success".to_string(),
                10,
                hash.to_string(),
            )
            .await
            .unwrap();
    }

    let operators = service_client.export_operators().await.unwrap();
    assert_eq!(operators, vec![ActorId::from(50), ActorId::from(100)]);

    let first_read_models = service_client.export_read_models_chunk(0, 2).await.unwrap();
    let second_read_models = service_client
        .export_read_models_chunk(first_read_models.next_cursor, 2)
        .await
        .unwrap();
    let repeated_first_read_models = service_client.export_read_models_chunk(0, 2).await.unwrap();

    assert_eq!(first_read_models, repeated_first_read_models);
    assert_eq!(first_read_models.cursor, 0);
    assert_eq!(first_read_models.limit, 2);
    assert_eq!(first_read_models.next_cursor, 2);
    assert!(!first_read_models.done);
    assert_eq!(second_read_models.cursor, 2);
    assert_eq!(second_read_models.next_cursor, 3);
    assert!(second_read_models.done);

    let exported_subjects = first_read_models
        .items
        .iter()
        .chain(second_read_models.items.iter())
        .map(|read_model| read_model.subject.clone())
        .collect::<Vec<_>>();
    assert_eq!(
        exported_subjects,
        vec![
            "@Alpha-service".to_string(),
            "@beta-service".to_string(),
            "@gamma-service".to_string(),
        ]
    );

    let first_attestations = service_client
        .export_attestations_chunk(0, 2)
        .await
        .unwrap();
    let second_attestations = service_client
        .export_attestations_chunk(first_attestations.next_cursor, 2)
        .await
        .unwrap();
    let empty_tail = service_client
        .export_attestations_chunk(second_attestations.next_cursor, 2)
        .await
        .unwrap();

    assert_eq!(first_attestations.total, 3);
    assert_eq!(first_attestations.next_cursor, 2);
    assert!(!first_attestations.done);
    assert_eq!(second_attestations.next_cursor, 3);
    assert!(second_attestations.done);
    assert_eq!(empty_tail.cursor, 3);
    assert_eq!(empty_tail.items.len(), 0);
    assert!(empty_tail.done);

    let exported_attestation_subjects = first_attestations
        .items
        .iter()
        .chain(second_attestations.items.iter())
        .map(|attestation| attestation.subject.clone())
        .collect::<Vec<_>>();
    assert_eq!(
        exported_attestation_subjects,
        vec![
            "@Alpha-service".to_string(),
            "@beta-service".to_string(),
            "@gamma-service".to_string(),
        ]
    );
}

#[tokio::test]
async fn owner_can_delegate_read_model_updates_to_operator() {
    let program = deploy_oracle().await;
    let mut service_client = program.reputation_oracle();

    let receipt = service_client
        .add_read_model_operator(UNAUTHORIZED_ACTOR_ID.into())
        .await
        .unwrap();

    assert_eq!(receipt.operator, UNAUTHORIZED_ACTOR_ID.into());
    assert!(receipt.added);
    assert!(!receipt.removed);
    assert_eq!(receipt.operator_count, 1);

    let operator_policy = service_client
        .read_model_policy()
        .with_actor_id(UNAUTHORIZED_ACTOR_ID.into())
        .await
        .unwrap();

    assert_eq!(operator_policy.owner, ACTOR_ID.into());
    assert_eq!(operator_policy.caller, UNAUTHORIZED_ACTOR_ID.into());
    assert!(operator_policy.caller_can_upsert);
    assert_eq!(operator_policy.operator_count, 1);
    assert_eq!(operator_policy.max_operators, 8);

    let update = service_client
        .upsert_read_model(
            "@operator-fed-service".to_string(),
            ReputationSignals {
                incoming_unique_callers: 5,
                outgoing_meaningful_calls: 4,
                chat_board_updates: 3,
                has_identity_card: true,
                has_verified_social_proof: true,
                circular_call_signals: 0,
                missing_required_metadata: 0,
                positive_attestations: 1,
                negative_attestations: 0,
            },
            vec![
                "registry:application".to_string(),
                "board:identity".to_string(),
                "social:verified".to_string(),
            ],
        )
        .with_actor_id(UNAUTHORIZED_ACTOR_ID.into())
        .await
        .unwrap();

    assert_eq!(update.subject, "@operator-fed-service");
    assert_eq!(update.stored_subjects, 1);
    assert_eq!(update.report.scores.verdict, "review");
    assert_eq!(update.report.scores.trust_score, 57);
    assert_eq!(
        update.report.integration_decision.max_attestation_weight,
        22
    );

    let report = service_client
        .score_agent("@operator-fed-service".to_string())
        .await
        .unwrap();

    assert_eq!(report.subject, "@operator-fed-service");
    assert_eq!(report.scores.verdict, "review");
    assert_eq!(report.integration_decision.action, "manual_review");
}

#[tokio::test]
async fn owner_can_remove_read_model_operator() {
    let program = deploy_oracle().await;
    let mut service_client = program.reputation_oracle();

    service_client
        .add_read_model_operator(UNAUTHORIZED_ACTOR_ID.into())
        .await
        .unwrap();

    let receipt = service_client
        .remove_read_model_operator(UNAUTHORIZED_ACTOR_ID.into())
        .await
        .unwrap();

    assert_eq!(receipt.operator, UNAUTHORIZED_ACTOR_ID.into());
    assert!(!receipt.added);
    assert!(receipt.removed);
    assert_eq!(receipt.operator_count, 0);

    let operator_policy = service_client
        .read_model_policy()
        .with_actor_id(UNAUTHORIZED_ACTOR_ID.into())
        .await
        .unwrap();

    assert!(!operator_policy.caller_can_upsert);

    let result = service_client
        .upsert_read_model(
            "@removed-operator-service".to_string(),
            ReputationSignals {
                incoming_unique_callers: 5,
                outgoing_meaningful_calls: 4,
                chat_board_updates: 3,
                has_identity_card: true,
                has_verified_social_proof: true,
                circular_call_signals: 0,
                missing_required_metadata: 0,
                positive_attestations: 1,
                negative_attestations: 0,
            },
            vec!["registry:application".to_string()],
        )
        .with_actor_id(UNAUTHORIZED_ACTOR_ID.into())
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn non_owner_cannot_manage_read_model_operators() {
    let program = deploy_oracle().await;
    let mut service_client = program.reputation_oracle();

    let add_result = service_client
        .add_read_model_operator(UNAUTHORIZED_ACTOR_ID.into())
        .with_actor_id(UNAUTHORIZED_ACTOR_ID.into())
        .await;
    assert!(add_result.is_err());

    let remove_result = service_client
        .remove_read_model_operator(ACTOR_ID.into())
        .with_actor_id(UNAUTHORIZED_ACTOR_ID.into())
        .await;
    assert!(remove_result.is_err());

    let policy = service_client.read_model_policy().await.unwrap();
    assert_eq!(policy.operator_count, 0);
}

#[tokio::test]
async fn owner_cannot_add_zero_read_model_operator() {
    let program = deploy_oracle().await;
    let mut service_client = program.reputation_oracle();

    let add_result = service_client
        .add_read_model_operator(ActorId::default())
        .await;
    assert!(add_result.is_err());

    let policy = service_client.read_model_policy().await.unwrap();
    assert_eq!(policy.operator_count, 0);
}

#[tokio::test]
async fn upsert_read_model_scores_dynamic_subject() {
    let program = deploy_oracle().await;
    let mut service_client = program.reputation_oracle();

    let receipt = service_client
        .upsert_read_model(
            "@live-service".to_string(),
            ReputationSignals {
                incoming_unique_callers: 8,
                outgoing_meaningful_calls: 5,
                chat_board_updates: 4,
                has_identity_card: true,
                has_verified_social_proof: true,
                circular_call_signals: 0,
                missing_required_metadata: 0,
                positive_attestations: 2,
                negative_attestations: 0,
            },
            vec![
                "registry:application".to_string(),
                "board:identity".to_string(),
                "social:verified".to_string(),
                "calls:incoming".to_string(),
            ],
        )
        .await
        .unwrap();

    assert_eq!(receipt.subject, "@live-service");
    assert!(!receipt.replaced);
    assert_eq!(receipt.evicted_subject, None);
    assert_eq!(receipt.stored_subjects, 1);
    assert_eq!(receipt.report.scores.verdict, "review");
    assert_eq!(receipt.report.integration_decision.action, "manual_review");

    let report = service_client
        .score_agent("@live-service".to_string())
        .await
        .unwrap();

    assert_eq!(report.subject, "@live-service");
    assert_eq!(report.scores.verdict, "review");
    assert_eq!(report.scores.trust_score, 69);
    assert_eq!(report.scores.activity_score, 57);
    assert_eq!(report.scores.integration_score, 65);
    assert_eq!(report.scores.confidence, 98);
    assert_eq!(report.scores.spam_risk, 0);
    assert_eq!(report.signals.incoming_unique_callers, 8);
    assert_eq!(report.integration_decision.max_attestation_weight, 25);
}

#[tokio::test]
async fn upsert_read_model_accepts_operator_preview_fixture_packet() {
    let program = deploy_oracle().await;
    let mut service_client = program.reputation_oracle();

    let receipt = service_client
        .upsert_read_model(
            "@registry-helper".to_string(),
            ReputationSignals {
                incoming_unique_callers: 3,
                outgoing_meaningful_calls: 3,
                chat_board_updates: 3,
                has_identity_card: true,
                has_verified_social_proof: true,
                circular_call_signals: 0,
                missing_required_metadata: 0,
                positive_attestations: 1,
                negative_attestations: 0,
            },
            vec![
                "activity:chat-board:3".to_string(),
                "board:identity".to_string(),
                "calls:incoming".to_string(),
                "calls:meaningful-out:3".to_string(),
                "calls:outgoing".to_string(),
                "calls:unique-in:3".to_string(),
                "chat-board:activity".to_string(),
                "registry:application".to_string(),
                "registry:program:0xregistryhelper".to_string(),
                "social:verified".to_string(),
                "verdict:review".to_string(),
            ],
        )
        .await
        .unwrap();

    assert_eq!(receipt.subject, "@registry-helper");
    assert_eq!(receipt.stored_subjects, 1);
    assert_eq!(receipt.report.scores.verdict, "review");
    assert_eq!(receipt.report.scores.trust_score, 48);
    assert_eq!(receipt.report.scores.confidence, 80);
    assert_eq!(receipt.report.scores.spam_risk, 0);
    assert_eq!(receipt.report.integration_decision.action, "manual_review");
    assert_eq!(receipt.report.evidence.len(), 11);
    assert_eq!(receipt.report.evidence[0], "activity:chat-board:3");
}

#[tokio::test]
async fn upsert_read_model_rejects_non_owner_and_preserves_state() {
    let program = deploy_oracle().await;
    let mut service_client = program.reputation_oracle();

    let result = service_client
        .upsert_read_model(
            "@spoofed-service".to_string(),
            ReputationSignals {
                incoming_unique_callers: 20,
                outgoing_meaningful_calls: 20,
                chat_board_updates: 20,
                has_identity_card: true,
                has_verified_social_proof: true,
                circular_call_signals: 0,
                missing_required_metadata: 0,
                positive_attestations: 10,
                negative_attestations: 0,
            },
            vec!["registry:application".to_string()],
        )
        .with_actor_id(UNAUTHORIZED_ACTOR_ID.into())
        .await;

    assert!(result.is_err());

    let policy = service_client
        .read_model_policy()
        .with_actor_id(UNAUTHORIZED_ACTOR_ID.into())
        .await
        .unwrap();

    assert_eq!(policy.owner, ACTOR_ID.into());
    assert_eq!(policy.caller, UNAUTHORIZED_ACTOR_ID.into());
    assert!(!policy.caller_can_upsert);
    assert_eq!(policy.operator_count, 0);

    let report = service_client
        .score_agent("@spoofed-service".to_string())
        .await
        .unwrap();

    assert_eq!(
        report.evidence,
        vec!["fixture: no matching read-model subject"]
    );
    assert_eq!(report.scores.verdict, "review");
    assert_eq!(report.scores.trust_score, 30);
}

#[tokio::test]
async fn upsert_read_model_rejects_blank_subject_and_normalizes_evidence() {
    let program = deploy_oracle().await;
    let mut service_client = program.reputation_oracle();

    let blank_result = service_client
        .upsert_read_model(
            "   ".to_string(),
            ReputationSignals {
                incoming_unique_callers: 5,
                outgoing_meaningful_calls: 4,
                chat_board_updates: 3,
                has_identity_card: true,
                has_verified_social_proof: true,
                circular_call_signals: 0,
                missing_required_metadata: 0,
                positive_attestations: 1,
                negative_attestations: 0,
            },
            vec!["registry:application".to_string()],
        )
        .await;

    assert!(blank_result.is_err());

    let receipt = service_client
        .upsert_read_model(
            "  @trimmed-service  ".to_string(),
            ReputationSignals {
                incoming_unique_callers: 5,
                outgoing_meaningful_calls: 4,
                chat_board_updates: 3,
                has_identity_card: true,
                has_verified_social_proof: true,
                circular_call_signals: 0,
                missing_required_metadata: 0,
                positive_attestations: 1,
                negative_attestations: 0,
            },
            vec![
                " registry:application ".to_string(),
                " ".to_string(),
                "board:identity".to_string(),
                "registry:application".to_string(),
            ],
        )
        .await
        .unwrap();

    assert_eq!(receipt.subject, "@trimmed-service");
    assert_eq!(
        receipt.report.evidence,
        vec![
            "board:identity".to_string(),
            "registry:application".to_string(),
        ]
    );

    let report = service_client
        .score_agent("@trimmed-service".to_string())
        .await
        .unwrap();

    assert_eq!(report.subject, "@trimmed-service");
    assert_eq!(report.evidence, receipt.report.evidence);
}

#[tokio::test]
async fn upsert_read_model_truncates_oversized_evidence_labels() {
    let program = deploy_oracle().await;
    let mut service_client = program.reputation_oracle();

    let long_label = format!("evidence:{}", "x".repeat(140));
    let receipt = service_client
        .upsert_read_model(
            "@bounded-evidence-service".to_string(),
            ReputationSignals {
                incoming_unique_callers: 5,
                outgoing_meaningful_calls: 4,
                chat_board_updates: 3,
                has_identity_card: true,
                has_verified_social_proof: true,
                circular_call_signals: 0,
                missing_required_metadata: 0,
                positive_attestations: 1,
                negative_attestations: 0,
            },
            vec![long_label.clone(), long_label],
        )
        .await
        .unwrap();

    assert_eq!(receipt.report.evidence.len(), 1);
    assert_eq!(receipt.report.evidence[0].chars().count(), 96);

    let report = service_client
        .score_agent("@bounded-evidence-service".to_string())
        .await
        .unwrap();

    assert_eq!(report.evidence, receipt.report.evidence);
}

#[tokio::test]
async fn upsert_read_model_replaces_and_scores_case_insensitive_subject_keys() {
    let program = deploy_oracle().await;
    let mut service_client = program.reputation_oracle();

    let first = service_client
        .upsert_read_model(
            "@Live-Service".to_string(),
            ReputationSignals {
                incoming_unique_callers: 4,
                outgoing_meaningful_calls: 2,
                chat_board_updates: 1,
                has_identity_card: true,
                has_verified_social_proof: false,
                circular_call_signals: 0,
                missing_required_metadata: 1,
                positive_attestations: 0,
                negative_attestations: 0,
            },
            vec!["registry:application".to_string()],
        )
        .await
        .unwrap();

    assert_eq!(first.subject, "@Live-Service");
    assert_eq!(first.stored_subjects, 1);
    assert!(!first.replaced);

    let replacement = service_client
        .upsert_read_model(
            "  @live-service  ".to_string(),
            ReputationSignals {
                incoming_unique_callers: 8,
                outgoing_meaningful_calls: 5,
                chat_board_updates: 4,
                has_identity_card: true,
                has_verified_social_proof: true,
                circular_call_signals: 0,
                missing_required_metadata: 0,
                positive_attestations: 2,
                negative_attestations: 0,
            },
            vec!["social:verified".to_string()],
        )
        .await
        .unwrap();

    assert!(replacement.replaced);
    assert_eq!(replacement.subject, "@live-service");
    assert_eq!(replacement.stored_subjects, 1);

    let scored_from_original_case = service_client
        .score_agent("@Live-Service".to_string())
        .await
        .unwrap();

    assert_eq!(scored_from_original_case.subject, "@live-service");
    assert_eq!(scored_from_original_case.signals.incoming_unique_callers, 8);
    assert!(scored_from_original_case.signals.has_verified_social_proof);
    assert_eq!(scored_from_original_case.evidence, vec!["social:verified"]);
}

#[tokio::test]
async fn recommend_agents_prefers_case_insensitive_read_model_over_fixture_subject() {
    let program = deploy_oracle().await;
    let mut service_client = program.reputation_oracle();

    service_client
        .upsert_read_model(
            "  @Registry-Helper  ".to_string(),
            ReputationSignals {
                incoming_unique_callers: 8,
                outgoing_meaningful_calls: 5,
                chat_board_updates: 4,
                has_identity_card: true,
                has_verified_social_proof: true,
                circular_call_signals: 0,
                missing_required_metadata: 0,
                positive_attestations: 2,
                negative_attestations: 0,
            },
            vec!["operator:replacement".to_string()],
        )
        .await
        .unwrap();

    let recommendations = service_client
        .recommend_agents(
            "agent-services".to_string(),
            "safe integration target".to_string(),
            3,
            0,
            true,
        )
        .await
        .unwrap();

    assert_eq!(
        recommendations
            .iter()
            .filter(|recommendation| recommendation
                .subject
                .eq_ignore_ascii_case("@registry-helper"))
            .count(),
        1
    );

    let replacement = recommendations
        .iter()
        .find(|recommendation| recommendation.subject == "@Registry-Helper")
        .unwrap();

    assert_eq!(replacement.trust_score, 69);
    assert_eq!(replacement.report.evidence, vec!["operator:replacement"]);
}

#[tokio::test]
async fn upsert_read_model_updates_recommendations_and_comparison() {
    let program = deploy_oracle().await;
    let mut service_client = program.reputation_oracle();

    service_client
        .upsert_read_model(
            "@live-service".to_string(),
            ReputationSignals {
                incoming_unique_callers: 8,
                outgoing_meaningful_calls: 5,
                chat_board_updates: 4,
                has_identity_card: true,
                has_verified_social_proof: true,
                circular_call_signals: 0,
                missing_required_metadata: 0,
                positive_attestations: 2,
                negative_attestations: 0,
            },
            vec![
                "registry:application".to_string(),
                "board:identity".to_string(),
                "social:verified".to_string(),
                "calls:incoming".to_string(),
            ],
        )
        .await
        .unwrap();

    let recommendations = service_client
        .recommend_agents(
            "agent-services".to_string(),
            "safe integration target".to_string(),
            3,
            60,
            false,
        )
        .await
        .unwrap();

    assert_eq!(recommendations.len(), 2);
    assert_eq!(recommendations[0].subject, "@live-service");
    assert_eq!(recommendations[0].verdict, "review");
    assert_eq!(recommendations[0].trust_score, 69);
    assert_eq!(
        recommendations[0].explanation.positive_signals,
        vec![
            "8 unique inbound callers".to_string(),
            "published identity card".to_string(),
            "declared external metadata".to_string(),
        ]
    );

    let comparison = service_client
        .compare_agents(
            vec!["@registry-helper".to_string(), "@live-service".to_string()],
            "agent-services".to_string(),
            "safe integration target".to_string(),
        )
        .await
        .unwrap();

    assert_eq!(comparison.winner.subject, "@live-service");
    assert_eq!(
        comparison.decision.runner_up,
        Some("@registry-helper".to_string())
    );
    assert_eq!(comparison.decision.trust_margin, 21);
}

#[tokio::test]
async fn upsert_read_model_evicts_oldest_subject_at_capacity() {
    let program = deploy_oracle().await;
    let mut service_client = program.reputation_oracle();

    let mut last_receipt = None;
    for index in 0..33 {
        let subject = format!("@bounded-service-{index:02}");
        last_receipt = Some(
            service_client
                .upsert_read_model(
                    subject,
                    ReputationSignals {
                        incoming_unique_callers: 5,
                        outgoing_meaningful_calls: 4,
                        chat_board_updates: 3,
                        has_identity_card: true,
                        has_verified_social_proof: true,
                        circular_call_signals: 0,
                        missing_required_metadata: 0,
                        positive_attestations: 1,
                        negative_attestations: 0,
                    },
                    vec!["registry:application".to_string()],
                )
                .await
                .unwrap(),
        );
    }

    let receipt = last_receipt.unwrap();
    assert_eq!(receipt.subject, "@bounded-service-32");
    assert_eq!(receipt.stored_subjects, 32);
    assert_eq!(
        receipt.evicted_subject,
        Some("@bounded-service-00".to_string())
    );

    let evicted_report = service_client
        .score_agent("@bounded-service-00".to_string())
        .await
        .unwrap();
    assert_eq!(
        evicted_report.evidence,
        vec!["fixture: no matching read-model subject".to_string()]
    );

    let retained_report = service_client
        .score_agent("@bounded-service-32".to_string())
        .await
        .unwrap();
    assert_eq!(retained_report.subject, "@bounded-service-32");
    assert_eq!(retained_report.evidence, vec!["registry:application"]);
}

#[tokio::test]
async fn record_attestation_persists_bounded_receipt() {
    let program = deploy_oracle().await;
    let mut service_client = program.reputation_oracle();

    let receipt = service_client
        .record_attestation(
            "@registry-helper".to_string(),
            "@vara-reputation-oracle".to_string(),
            "integration_success".to_string(),
            35,
            "0xregistryhelperdemo".to_string(),
        )
        .await
        .unwrap();

    assert_eq!(
        receipt.receipt_id,
        "@registry-helper|@vara-reputation-oracle|integration_success|35|0xregistryhelperdemo|2026-05-20T12:00:00.000Z"
    );
    assert_eq!(receipt.subject, "@registry-helper");
    assert_eq!(receipt.issuer, "@vara-reputation-oracle");
    assert_eq!(receipt.kind, "integration_success");
    assert_eq!(receipt.weight, 35);
    assert_eq!(receipt.evidence_hash, "0xregistryhelperdemo");
    assert_eq!(receipt.issued_at, "2026-05-20T12:00:00.000Z");

    let attestations = service_client
        .get_attestations("@registry-helper".to_string())
        .await
        .unwrap();

    assert_eq!(attestations, vec![receipt]);
}

#[tokio::test]
async fn record_attestation_evicts_oldest_receipt_at_capacity() {
    let program = deploy_oracle().await;
    let mut service_client = program.reputation_oracle();

    for index in 0..65 {
        service_client
            .record_attestation(
                "@registry-helper".to_string(),
                format!("@reviewer-{index:02}"),
                "integration_success".to_string(),
                index,
                format!("0xhash{index:02}"),
            )
            .await
            .unwrap();
    }

    let attestations = service_client
        .get_attestations("@registry-helper".to_string())
        .await
        .unwrap();

    assert_eq!(attestations.len(), 64);
    assert!(
        !attestations
            .iter()
            .any(|attestation| attestation.issuer == "@reviewer-00")
    );
    assert!(
        attestations
            .iter()
            .any(|attestation| attestation.issuer == "@reviewer-64")
    );
}

#[tokio::test]
async fn record_attestation_clamps_weight_and_filters_subject() {
    let program = deploy_oracle().await;
    let mut service_client = program.reputation_oracle();

    let receipt = service_client
        .record_attestation(
            "@loop-farm".to_string(),
            "@reviewer".to_string(),
            "risk_report".to_string(),
            -250,
            "0xlooprisk".to_string(),
        )
        .await
        .unwrap();

    assert_eq!(receipt.weight, -100);
    assert_eq!(
        receipt.receipt_id,
        "@loop-farm|@reviewer|risk_report|-100|0xlooprisk|2026-05-20T12:00:00.000Z"
    );

    let unrelated = service_client
        .get_attestations("@registry-helper".to_string())
        .await
        .unwrap();
    let loop_farm_attestations = service_client
        .get_attestations("@loop-farm".to_string())
        .await
        .unwrap();

    assert!(unrelated.is_empty());
    assert_eq!(loop_farm_attestations, vec![receipt]);
}

#[tokio::test]
async fn record_attestation_rejects_blank_fields_and_trims_receipt_inputs() {
    let program = deploy_oracle().await;
    let mut service_client = program.reputation_oracle();

    let blank_subject = service_client
        .record_attestation(
            " ".to_string(),
            "@reviewer".to_string(),
            "risk_report".to_string(),
            10,
            "0xhash".to_string(),
        )
        .await;
    assert!(blank_subject.is_err());

    let blank_issuer = service_client
        .record_attestation(
            "@registry-helper".to_string(),
            " ".to_string(),
            "risk_report".to_string(),
            10,
            "0xhash".to_string(),
        )
        .await;
    assert!(blank_issuer.is_err());

    let blank_kind = service_client
        .record_attestation(
            "@registry-helper".to_string(),
            "@reviewer".to_string(),
            " ".to_string(),
            10,
            "0xhash".to_string(),
        )
        .await;
    assert!(blank_kind.is_err());

    let blank_hash = service_client
        .record_attestation(
            "@registry-helper".to_string(),
            "@reviewer".to_string(),
            "risk_report".to_string(),
            10,
            " ".to_string(),
        )
        .await;
    assert!(blank_hash.is_err());

    let receipt = service_client
        .record_attestation(
            "  @registry-helper  ".to_string(),
            "  @reviewer  ".to_string(),
            "  integration_success  ".to_string(),
            10,
            "  0xtrimmed  ".to_string(),
        )
        .await
        .unwrap();

    assert_eq!(receipt.subject, "@registry-helper");
    assert_eq!(receipt.issuer, "@reviewer");
    assert_eq!(receipt.kind, "integration_success");
    assert_eq!(receipt.evidence_hash, "0xtrimmed");
    assert_eq!(
        receipt.receipt_id,
        "@registry-helper|@reviewer|integration_success|10|0xtrimmed|2026-05-20T12:00:00.000Z"
    );
}

#[tokio::test]
async fn get_attestations_rejects_blank_and_trims_subject() {
    let program = deploy_oracle().await;
    let mut service_client = program.reputation_oracle();

    let receipt = service_client
        .record_attestation(
            "@registry-helper".to_string(),
            "@reviewer".to_string(),
            "integration_success".to_string(),
            10,
            "0xhash".to_string(),
        )
        .await
        .unwrap();

    let blank_result = service_client.get_attestations("   ".to_string()).await;
    assert!(blank_result.is_err());

    let attestations = service_client
        .get_attestations("  @registry-helper  ".to_string())
        .await
        .unwrap();

    assert_eq!(attestations, vec![receipt]);
}
