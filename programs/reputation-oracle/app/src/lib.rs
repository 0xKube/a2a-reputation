#![no_std]

use sails_rs::cell::RefCell;
use sails_rs::gstd::{CommandReply, exec, msg};
use sails_rs::prelude::*;

const AS_OF: &str = "2026-05-20T12:00:00.000Z";
const MAX_ATTESTATIONS: usize = 64;
const MAX_EXPORT_CHUNK: usize = 64;
const MAX_READ_MODELS: usize = 32;
const MAX_OPERATORS: usize = 8;
const MAX_EVIDENCE_LABEL_CHARS: usize = 96;
const MAX_ECONOMIC_PROFILES: usize = 64;
const MAX_SIGNAL_RECEIPTS: usize = 128;
const MAX_USAGE_PREDICTIONS: usize = 128;
const THREE_HOURS_MS: u64 = 3 * 60 * 60 * 1_000;
const VARA_UNIT: u128 = 1_000_000_000_000;
const MIN_USAGE_PREDICTION_STAKE: u128 = 10 * VARA_UNIT;
const MAX_USAGE_PREDICTION_STAKE: u128 = 10_000 * VARA_UNIT;

#[derive(Clone, Debug, Decode, Encode, PartialEq, Eq, TypeInfo)]
pub struct ReputationScores {
    pub trust_score: u8,
    pub activity_score: u8,
    pub integration_score: u8,
    pub identity_score: u8,
    pub spam_risk: u8,
    pub confidence: u8,
    pub verdict: String,
}

#[derive(Clone, Debug, Decode, Encode, PartialEq, Eq, TypeInfo)]
pub struct ReputationSignals {
    pub incoming_unique_callers: u32,
    pub outgoing_meaningful_calls: u32,
    pub chat_board_updates: u32,
    pub has_identity_card: bool,
    pub has_verified_social_proof: bool,
    pub circular_call_signals: u32,
    pub missing_required_metadata: u32,
    pub positive_attestations: u32,
    pub negative_attestations: u32,
}

#[derive(Clone, Debug, Decode, Encode, PartialEq, Eq, TypeInfo)]
pub enum ApplicationStatusV2 {
    Unknown,
    Draft,
    Submitted,
    Approved,
    Rejected,
    Suspended,
}

#[derive(Clone, Debug, Decode, Encode, PartialEq, Eq, TypeInfo)]
pub enum ParticipantStatusV2 {
    Unknown,
    ParticipantLike,
    VerifiedParticipant,
    NonParticipant,
}

#[derive(Clone, Debug, Decode, Encode, PartialEq, Eq, TypeInfo)]
pub struct ReputationSignalsV2 {
    pub inbound_unique_participants: u32,
    pub inbound_call_count: u32,
    pub outbound_unique_participants: u32,
    pub outbound_call_count: u32,
    pub valid_participant_inbound_count: u32,
    pub valid_participant_outbound_count: u32,
    pub non_participant_call_count: u32,
    pub self_loop_call_count: u32,
    pub same_owner_call_count: u32,
    pub mention_count: u32,
    pub messages_sent: u32,
    pub posts_active: u32,
    pub has_clear_board_description: bool,
    pub has_identity_card: bool,
    pub has_actual_github_repo: bool,
    pub has_skills_url: bool,
    pub has_idl_url: bool,
    pub has_frontend_url: bool,
    pub handle_claimed: bool,
    pub application_status: ApplicationStatusV2,
    pub participant_status: ParticipantStatusV2,
    pub call_graph_density_bps: u32,
    pub low_diversity_volume_count: u32,
    pub reciprocal_farming_signals: u32,
    pub positive_third_party_attestations: u32,
    pub negative_third_party_attestations: u32,
    pub metrics_updated_at: u64,
    pub identity_updated_at: u64,
    pub registered_at: u64,
}

#[derive(Clone, Debug, Decode, Encode, PartialEq, Eq, TypeInfo)]
pub struct ReputationScoresV2 {
    pub ecosystem_value_score: u8,
    pub real_integration_score: u8,
    pub counterparty_diversity_score: u8,
    pub identity_provenance_score: u8,
    pub demo_readiness_score: u8,
    pub safety_score: u8,
    pub spam_risk: u8,
    pub confidence_score: u8,
    pub overall_score: u8,
    pub verdict: String,
}

#[derive(Clone, Debug, Decode, Encode, PartialEq, Eq, TypeInfo)]
pub struct ReputationReportV2 {
    pub subject: String,
    pub as_of: String,
    pub report_version: u32,
    pub signals: ReputationSignalsV2,
    pub scores: ReputationScoresV2,
    pub evidence: Vec<String>,
}

#[derive(Clone, Debug, Decode, Encode, PartialEq, Eq, TypeInfo)]
pub struct IntegrationDecision {
    pub action: String,
    pub risk_level: String,
    pub max_attestation_weight: i32,
    pub next_steps: Vec<String>,
}

#[derive(Clone, Debug, Decode, Encode, PartialEq, Eq, TypeInfo)]
pub struct ReputationReport {
    pub subject: String,
    pub as_of: String,
    pub report_version: u32,
    pub scores: ReputationScores,
    pub signals: ReputationSignals,
    pub integration_decision: IntegrationDecision,
    pub evidence: Vec<String>,
}

#[derive(Clone, Debug, Decode, Encode, PartialEq, Eq, TypeInfo)]
pub struct ComparableAgent {
    pub input: String,
    pub subject: String,
    pub verdict: String,
    pub trust_score: u8,
    pub confidence: u8,
    pub spam_risk: u8,
    pub reason: String,
    pub report: ReputationReport,
}

#[derive(Clone, Debug, Decode, Encode, PartialEq, Eq, TypeInfo)]
pub struct ComparisonDecision {
    pub winner: String,
    pub runner_up: Option<String>,
    pub trust_margin: i32,
    pub spam_risk_advantage: i32,
    pub summary: String,
}

#[derive(Clone, Debug, Decode, Encode, PartialEq, Eq, TypeInfo)]
pub struct AgentComparison {
    pub track: String,
    pub need: String,
    pub winner: ComparableAgent,
    pub candidates: Vec<ComparableAgent>,
    pub decision: ComparisonDecision,
}

#[derive(Clone, Debug, Decode, Encode, PartialEq, Eq, TypeInfo)]
pub struct RecommendationExplanation {
    pub summary: String,
    pub positive_signals: Vec<String>,
    pub negative_signals: Vec<String>,
}

#[derive(Clone, Debug, Decode, Encode, PartialEq, Eq, TypeInfo)]
pub struct AgentRecommendation {
    pub subject: String,
    pub track: String,
    pub need: String,
    pub verdict: String,
    pub trust_score: u8,
    pub confidence: u8,
    pub spam_risk: u8,
    pub reason: String,
    pub explanation: RecommendationExplanation,
    pub report: ReputationReport,
}

#[derive(Clone, Debug, Decode, Encode, PartialEq, Eq, TypeInfo)]
pub struct AttestationReceipt {
    pub receipt_id: String,
    pub subject: String,
    pub issuer: String,
    pub kind: String,
    pub weight: i32,
    pub evidence_hash: String,
    pub issued_at: String,
}

#[derive(Clone, Debug, Decode, Encode, PartialEq, Eq, TypeInfo)]
pub struct AgentReadModel {
    pub subject: String,
    pub signals: ReputationSignals,
    pub evidence: Vec<String>,
}

#[derive(Clone, Debug, Decode, Encode, PartialEq, Eq, TypeInfo)]
pub struct ReadModelReceipt {
    pub subject: String,
    pub replaced: bool,
    pub evicted_subject: Option<String>,
    pub stored_subjects: u32,
    pub report: ReputationReport,
}

#[derive(Clone, Debug, Decode, Encode, PartialEq, Eq, TypeInfo)]
pub struct ReadModelPolicy {
    pub owner: ActorId,
    pub caller: ActorId,
    pub caller_can_upsert: bool,
    pub operator_count: u32,
    pub max_operators: u32,
    pub max_read_models: u32,
    pub max_evidence_labels: u32,
    pub max_evidence_label_chars: u32,
}

#[derive(Clone, Debug, Decode, Encode, PartialEq, Eq, TypeInfo)]
pub struct OperatorReceipt {
    pub operator: ActorId,
    pub added: bool,
    pub removed: bool,
    pub operator_count: u32,
}

#[derive(Clone, Debug, Decode, Encode, PartialEq, Eq, TypeInfo)]
pub enum OracleStatus {
    Active,
    ReadOnly,
}

#[derive(Clone, Debug, Decode, Encode, PartialEq, Eq, TypeInfo)]
pub struct ReadOnlyReceipt {
    pub owner: ActorId,
    pub previous_status: OracleStatus,
    pub status: OracleStatus,
    pub read_only: bool,
}

#[derive(Clone, Debug, Decode, Encode, PartialEq, Eq, TypeInfo)]
pub struct MigrationBounds {
    pub max_operators: u32,
    pub max_read_models: u32,
    pub max_attestations: u32,
    pub max_evidence_labels: u32,
    pub max_evidence_label_chars: u32,
    pub max_export_chunk: u32,
}

#[derive(Clone, Debug, Decode, Encode, PartialEq, Eq, TypeInfo)]
pub struct MigrationStats {
    pub operator_count: u32,
    pub read_model_count: u32,
    pub attestation_count: u32,
}

#[derive(Clone, Debug, Decode, Encode, PartialEq, Eq, TypeInfo)]
pub struct MigrationConfig {
    pub owner: ActorId,
    pub status: OracleStatus,
    pub read_only: bool,
    pub bounds: MigrationBounds,
    pub operator_count: u32,
    pub read_model_count: u32,
    pub attestation_count: u32,
    pub as_of: String,
    pub report_version: u32,
}

#[derive(Clone, Debug, Decode, Encode, PartialEq, Eq, TypeInfo)]
pub struct ReadModelExportChunk {
    pub cursor: u32,
    pub limit: u32,
    pub next_cursor: u32,
    pub done: bool,
    pub total: u32,
    pub items: Vec<AgentReadModel>,
}

#[derive(Clone, Debug, Decode, Encode, PartialEq, Eq, TypeInfo)]
pub struct AttestationExportChunk {
    pub cursor: u32,
    pub limit: u32,
    pub next_cursor: u32,
    pub done: bool,
    pub total: u32,
    pub items: Vec<AttestationReceipt>,
}

#[derive(Clone, Debug, Decode, Encode, PartialEq, Eq, TypeInfo)]
pub struct EconomicProfile {
    pub agent: String,
    pub oracle_points: i32,
    pub total_value_staked: u128,
    pub total_value_rewarded: u128,
    pub total_value_slashed: u128,
    pub virtual_stake: u32,
    pub useful_signals: u32,
    pub spam_signals: u32,
    pub last_verdict: String,
}

#[derive(Clone, Debug, Decode, Encode, PartialEq, Eq, TypeInfo)]
pub struct SignalReceipt {
    pub receipt_id: String,
    pub agent: String,
    pub subject: String,
    pub counterparty: String,
    pub evidence_hash: String,
    pub observed_value: u32,
    pub virtual_stake: u32,
    pub value_attached: u128,
    pub value_rewarded: u128,
    pub value_slashed: u128,
    pub quality_score: u8,
    pub points_delta: i32,
    pub profile_points: i32,
    pub reward_pool_balance: u128,
    pub verdict: String,
    pub reason: String,
    pub as_of: String,
}

#[derive(Clone, Debug, Decode, Encode, PartialEq, Eq, TypeInfo)]
pub struct RewardPoolReceipt {
    pub funder: ActorId,
    pub value_added: u128,
    pub reward_pool_balance: u128,
}

#[derive(Clone, Debug, Decode, Encode, PartialEq, Eq, TypeInfo)]
pub struct SignalReceiptExportChunk {
    pub cursor: u32,
    pub limit: u32,
    pub next_cursor: u32,
    pub done: bool,
    pub total: u32,
    pub items: Vec<SignalReceipt>,
}

#[derive(Clone, Debug, Decode, Encode, PartialEq, Eq, TypeInfo)]
pub enum PredictionStatus {
    Open,
    Won,
    Lost,
}

#[derive(Clone, Debug, Decode, Encode, PartialEq, Eq, TypeInfo)]
pub struct UsagePrediction {
    pub position_id: String,
    pub predictor: ActorId,
    pub epoch_id: u32,
    pub subject: String,
    pub window_start_ms: u64,
    pub window_end_ms: u64,
    pub opened_at_ms: u64,
    pub predicted_delta_calls: u32,
    pub evidence_hash: String,
    pub stake: u128,
    pub effective_stake: u128,
    pub late_penalty_bps: u32,
    pub late_penalty_value: u128,
    pub opened_at: String,
    pub status: PredictionStatus,
    pub actual_delta_calls: Option<u32>,
    pub settlement_snapshot_hash: Option<String>,
    pub error_bps: Option<u32>,
    pub payout: u128,
}

#[derive(Clone, Debug, Decode, Encode, PartialEq, Eq, TypeInfo)]
pub struct UsagePredictionReceipt {
    pub position_id: String,
    pub predictor: ActorId,
    pub epoch_id: u32,
    pub subject: String,
    pub window_start_ms: u64,
    pub window_end_ms: u64,
    pub opened_at_ms: u64,
    pub predicted_delta_calls: u32,
    pub stake: u128,
    pub effective_stake: u128,
    pub late_penalty_bps: u32,
    pub late_penalty_value: u128,
    pub reward_pool_balance: u128,
}

#[derive(Clone, Debug, Decode, Encode, PartialEq, Eq, TypeInfo)]
pub struct PredictionSettlement {
    pub position_id: String,
    pub subject: String,
    pub predicted_delta_calls: u32,
    pub actual_delta_calls: u32,
    pub settlement_snapshot_hash: String,
    pub error_bps: u32,
    pub status: PredictionStatus,
    pub stake: u128,
    pub effective_stake: u128,
    pub late_penalty_bps: u32,
    pub payout: u128,
    pub reward_pool_balance: u128,
}

#[derive(Clone, Debug, Decode, Encode, PartialEq, Eq, TypeInfo)]
pub struct UsagePredictionExportChunk {
    pub cursor: u32,
    pub limit: u32,
    pub next_cursor: u32,
    pub done: bool,
    pub total: u32,
    pub items: Vec<UsagePrediction>,
}

#[derive(Default)]
pub struct OracleState {
    owner: ActorId,
    read_only: bool,
    operators: Vec<ActorId>,
    attestations: Vec<AttestationReceipt>,
    read_models: Vec<AgentReadModel>,
    economic_profiles: Vec<EconomicProfile>,
    signal_receipts: Vec<SignalReceipt>,
    usage_predictions: Vec<UsagePrediction>,
    reward_pool: u128,
}

struct ReputationOracle<'a> {
    state: &'a RefCell<OracleState>,
}

impl<'a> ReputationOracle<'a> {
    pub fn create(state: &'a RefCell<OracleState>) -> Self {
        Self { state }
    }
}

#[sails_rs::service]
impl ReputationOracle<'_> {
    #[export]
    pub fn score_agent(&self, subject: String) -> ReputationReport {
        let subject = require_trimmed_string(subject, "score_agent subject must be non-empty");
        self.report_for_subject(subject)
    }

    #[export]
    pub fn score_agent_v2(
        &self,
        subject: String,
        signals: ReputationSignalsV2,
        evidence: Vec<String>,
    ) -> ReputationReportV2 {
        let subject = require_trimmed_string(subject, "score_agent_v2 subject must be non-empty");
        let evidence = normalize_evidence_labels(evidence);
        let scores = scores_v2_from_signals(&signals);

        ReputationReportV2 {
            subject,
            as_of: AS_OF.into(),
            report_version: 3,
            signals,
            scores,
            evidence,
        }
    }

    #[export]
    pub fn compare_agents(
        &self,
        subjects: Vec<String>,
        track: String,
        need: String,
    ) -> AgentComparison {
        assert!(
            subjects.len() >= 2,
            "compare_agents requires at least two subjects"
        );
        assert!(
            subjects.iter().all(|subject| !subject.trim().is_empty()),
            "compare_agents subjects must be non-empty"
        );

        let subjects = subjects
            .into_iter()
            .map(|subject| {
                require_trimmed_string(subject, "compare_agents subjects must be non-empty")
            })
            .collect::<Vec<_>>();

        let mut candidates = subjects
            .into_iter()
            .map(|subject| self.comparable_agent(subject))
            .collect::<Vec<_>>();
        candidates.sort_by(compare_candidates);

        let winner = candidates
            .first()
            .cloned()
            .unwrap_or_else(|| self.comparable_agent("@unresolved".into()));
        let runner_up = candidates.get(1);
        let trust_margin = runner_up
            .map(|runner_up| winner.trust_score as i32 - runner_up.trust_score as i32)
            .unwrap_or(winner.trust_score as i32);
        let spam_risk_advantage = runner_up
            .map(|runner_up| runner_up.spam_risk as i32 - winner.spam_risk as i32)
            .unwrap_or(100 - winner.spam_risk as i32);
        let runner_up_subject = runner_up.map(|runner_up| runner_up.subject.clone());
        let summary = match runner_up {
            Some(runner_up) => format!(
                "{} leads {} by {} trust points and {} spam-risk points",
                winner.subject, runner_up.subject, trust_margin, spam_risk_advantage
            ),
            None => format!("{} is the only comparable candidate", winner.subject),
        };

        AgentComparison {
            track,
            need,
            winner: winner.clone(),
            candidates,
            decision: ComparisonDecision {
                winner: winner.subject,
                runner_up: runner_up_subject,
                trust_margin,
                spam_risk_advantage,
                summary,
            },
        }
    }

    #[export]
    pub fn recommend_agents(
        &self,
        track: String,
        need: String,
        limit: u32,
        min_confidence: u8,
        include_avoid: bool,
    ) -> Vec<AgentRecommendation> {
        let mut recommendations = vec![
            self.recommendation_for_subject("@registry-helper".into(), track.clone(), need.clone()),
            self.recommendation_for_subject("@quiet-indexer".into(), track.clone(), need.clone()),
            self.recommendation_for_subject("@loop-farm".into(), track.clone(), need.clone()),
        ];

        let state = self.state.borrow();
        for read_model in state.read_models.iter() {
            let read_model_key = normalized_subject_key(&read_model.subject);
            if !recommendations.iter().any(|recommendation| {
                normalized_subject_key(&recommendation.subject) == read_model_key
            }) {
                recommendations.push(self.recommendation_for_subject(
                    read_model.subject.clone(),
                    track.clone(),
                    need.clone(),
                ));
            }
        }
        drop(state);

        let mut recommendations = recommendations
            .into_iter()
            .filter(|recommendation| recommendation.confidence >= min_confidence)
            .filter(|recommendation| include_avoid || recommendation.verdict != "avoid_or_wait")
            .collect::<Vec<_>>();
        recommendations.sort_by(compare_recommendations);

        let limit = core::cmp::max(1, limit) as usize;
        recommendations.truncate(limit);

        recommendations
    }

    #[export]
    pub fn read_model_policy(&self) -> ReadModelPolicy {
        let state = self.state.borrow();
        let caller = msg::source();

        ReadModelPolicy {
            owner: state.owner,
            caller,
            caller_can_upsert: state.can_upsert(caller),
            operator_count: state.operators.len() as u32,
            max_operators: MAX_OPERATORS as u32,
            max_read_models: MAX_READ_MODELS as u32,
            max_evidence_labels: 12,
            max_evidence_label_chars: MAX_EVIDENCE_LABEL_CHARS as u32,
        }
    }

    #[export]
    pub fn oracle_status(&self) -> OracleStatus {
        self.state.borrow().status()
    }

    #[export]
    pub fn export_migration_config(&self) -> MigrationConfig {
        let state = self.state.borrow();
        let stats = state.stats();

        MigrationConfig {
            owner: state.owner,
            status: state.status(),
            read_only: state.read_only,
            bounds: migration_bounds(),
            operator_count: stats.operator_count,
            read_model_count: stats.read_model_count,
            attestation_count: stats.attestation_count,
            as_of: AS_OF.into(),
            report_version: 2,
        }
    }

    #[export]
    pub fn export_stats(&self) -> MigrationStats {
        self.state.borrow().stats()
    }

    #[export]
    pub fn export_operators(&self) -> Vec<ActorId> {
        let mut operators = self.state.borrow().operators.clone();
        operators.sort();
        operators
    }

    #[export]
    pub fn export_read_models_chunk(&self, cursor: u32, limit: u32) -> ReadModelExportChunk {
        let mut read_models = self.state.borrow().read_models.clone();
        read_models.sort_by(compare_read_models_for_export);

        let total = read_models.len();
        let start = (cursor as usize).min(total);
        let limit = normalized_export_limit(limit);
        let end = start.saturating_add(limit).min(total);

        ReadModelExportChunk {
            cursor: start as u32,
            limit: limit as u32,
            next_cursor: end as u32,
            done: end >= total,
            total: total as u32,
            items: read_models[start..end].to_vec(),
        }
    }

    #[export]
    pub fn export_attestations_chunk(&self, cursor: u32, limit: u32) -> AttestationExportChunk {
        let mut attestations = self.state.borrow().attestations.clone();
        attestations.sort_by(compare_attestations_for_export);

        let total = attestations.len();
        let start = (cursor as usize).min(total);
        let limit = normalized_export_limit(limit);
        let end = start.saturating_add(limit).min(total);

        AttestationExportChunk {
            cursor: start as u32,
            limit: limit as u32,
            next_cursor: end as u32,
            done: end >= total,
            total: total as u32,
            items: attestations[start..end].to_vec(),
        }
    }

    #[export]
    pub fn submit_integration_signal(
        &mut self,
        agent: String,
        subject: String,
        counterparty: String,
        evidence_hash: String,
        observed_value: u32,
        virtual_stake: u32,
    ) -> CommandReply<SignalReceipt> {
        self.assert_active("oracle is read-only; economic signal submissions are disabled");
        let value_attached = msg::value();
        assert!(
            value_attached > 0,
            "integration signal requires an attached VARA stake"
        );
        let agent = require_trimmed_string(agent, "signal agent must be non-empty");
        let subject = require_trimmed_string(subject, "signal subject must be non-empty");
        let counterparty =
            require_trimmed_string(counterparty, "signal counterparty must be non-empty");
        let evidence_hash =
            require_trimmed_string(evidence_hash, "signal evidence hash must be non-empty");
        let observed_value = observed_value.min(100);
        let virtual_stake = virtual_stake.min(1_000);
        let report = self.report_for_subject(subject.clone());
        let self_loop = normalized_subject_key(&agent) == normalized_subject_key(&counterparty)
            || normalized_subject_key(&subject) == normalized_subject_key(&counterparty);
        let quality_score = signal_quality_score(&report, observed_value, self_loop);
        let points_delta = signal_points_delta(quality_score, virtual_stake, self_loop);
        let verdict = signal_verdict(quality_score, self_loop).to_string();
        let reason = signal_reason(quality_score, self_loop).to_string();
        let receipt_id = [
            agent.as_str(),
            subject.as_str(),
            counterparty.as_str(),
            evidence_hash.as_str(),
            AS_OF,
        ]
        .join("|");

        let mut state = self.state.borrow_mut();
        let value_slashed = if points_delta < 0 { value_attached } else { 0 };
        let value_bonus = if points_delta > 0 && quality_score >= 70 {
            state.reward_pool.min(value_attached / 2)
        } else {
            0
        };
        let value_rewarded = if value_slashed > 0 {
            0
        } else {
            value_attached.saturating_add(value_bonus)
        };
        if value_slashed > 0 {
            state.reward_pool = state.reward_pool.saturating_add(value_slashed);
        } else {
            state.reward_pool = state.reward_pool.saturating_sub(value_bonus);
        }
        let profile = state.upsert_economic_profile(
            agent.clone(),
            points_delta,
            virtual_stake,
            value_attached,
            value_rewarded,
            value_slashed,
            verdict.clone(),
        );
        let reward_pool_balance = state.reward_pool;

        let receipt = SignalReceipt {
            receipt_id,
            agent,
            subject,
            counterparty,
            evidence_hash,
            observed_value,
            virtual_stake,
            value_attached,
            value_rewarded,
            value_slashed,
            quality_score,
            points_delta,
            profile_points: profile.oracle_points,
            reward_pool_balance,
            verdict,
            reason,
            as_of: AS_OF.into(),
        };
        if state.signal_receipts.len() >= MAX_SIGNAL_RECEIPTS {
            state.signal_receipts.remove(0);
        }
        state.signal_receipts.push(receipt.clone());

        CommandReply::new(receipt).with_value(value_rewarded)
    }

    #[export]
    pub fn fund_reward_pool(&mut self) -> RewardPoolReceipt {
        self.assert_active("oracle is read-only; reward-pool funding is disabled");
        let value_added = msg::value();
        assert!(
            value_added > 0,
            "reward-pool funding requires attached VARA"
        );
        let mut state = self.state.borrow_mut();
        state.reward_pool = state.reward_pool.saturating_add(value_added);

        RewardPoolReceipt {
            funder: msg::source(),
            value_added,
            reward_pool_balance: state.reward_pool,
        }
    }

    #[export]
    pub fn economic_profile(&self, agent: String) -> EconomicProfile {
        let agent = require_trimmed_string(agent, "economic profile agent must be non-empty");
        let agent_key = normalized_subject_key(&agent);
        self.state
            .borrow()
            .economic_profiles
            .iter()
            .find(|profile| normalized_subject_key(&profile.agent) == agent_key)
            .cloned()
            .unwrap_or(EconomicProfile {
                agent,
                oracle_points: 0,
                total_value_staked: 0,
                total_value_rewarded: 0,
                total_value_slashed: 0,
                virtual_stake: 0,
                useful_signals: 0,
                spam_signals: 0,
                last_verdict: "unseen".into(),
            })
    }

    #[export]
    pub fn economic_leaderboard(&self, limit: u32) -> Vec<EconomicProfile> {
        let mut profiles = self.state.borrow().economic_profiles.clone();
        profiles.sort_by(compare_economic_profiles);
        profiles.truncate(core::cmp::max(1, limit).min(MAX_ECONOMIC_PROFILES as u32) as usize);
        profiles
    }

    #[export]
    pub fn export_signal_receipts_chunk(
        &self,
        cursor: u32,
        limit: u32,
    ) -> SignalReceiptExportChunk {
        let receipts = self.state.borrow().signal_receipts.clone();
        let total = receipts.len();
        let start = (cursor as usize).min(total);
        let limit = normalized_export_limit(limit);
        let end = start.saturating_add(limit).min(total);

        SignalReceiptExportChunk {
            cursor: start as u32,
            limit: limit as u32,
            next_cursor: end as u32,
            done: end >= total,
            total: total as u32,
            items: receipts[start..end].to_vec(),
        }
    }

    #[export]
    pub fn open_usage_prediction(
        &mut self,
        epoch_id: u32,
        subject: String,
        window_start_ms: u64,
        window_end_ms: u64,
        predicted_delta_calls: u32,
        evidence_hash: String,
    ) -> UsagePredictionReceipt {
        self.assert_active("oracle is read-only; usage predictions are disabled");
        let stake = msg::value();
        assert!(stake > 0, "usage prediction requires attached VARA stake");
        assert!(
            stake >= MIN_USAGE_PREDICTION_STAKE,
            "usage prediction stake is below the 10 VARA minimum"
        );
        assert!(
            stake <= MAX_USAGE_PREDICTION_STAKE,
            "usage prediction stake exceeds the 10000 VARA maximum"
        );
        assert!(
            window_end_ms > window_start_ms,
            "prediction window end must be after start"
        );
        let opened_at_ms = exec::block_timestamp();
        assert!(
            opened_at_ms < window_end_ms,
            "prediction window is already closed"
        );
        let subject = require_trimmed_string(subject, "prediction subject must be non-empty");
        let evidence_hash =
            require_trimmed_string(evidence_hash, "prediction evidence hash must be non-empty");
        let late_penalty_bps =
            prediction_late_penalty_bps(opened_at_ms, window_start_ms, window_end_ms);
        let late_penalty_value = stake
            .saturating_mul(late_penalty_bps as u128)
            .saturating_div(10_000);
        let effective_stake = stake.saturating_sub(late_penalty_value);
        let predictor = msg::source();
        let stake_text = stake.to_string();
        let epoch_text = epoch_id.to_string();
        let predicted_text = predicted_delta_calls.to_string();
        let opened_text = opened_at_ms.to_string();
        let position_id = [
            epoch_text.as_str(),
            subject.as_str(),
            predicted_text.as_str(),
            evidence_hash.as_str(),
            stake_text.as_str(),
            opened_text.as_str(),
            AS_OF,
        ]
        .join("|");

        let mut state = self.state.borrow_mut();
        state.reward_pool = state.reward_pool.saturating_add(late_penalty_value);
        if state.usage_predictions.len() >= MAX_USAGE_PREDICTIONS {
            let settled_index = state
                .usage_predictions
                .iter()
                .position(|position| !matches!(position.status, PredictionStatus::Open))
                .expect("usage prediction capacity reached; settle existing positions before opening more");
            state.usage_predictions.remove(settled_index);
        }
        state.usage_predictions.push(UsagePrediction {
            position_id: position_id.clone(),
            predictor,
            epoch_id,
            subject: subject.clone(),
            window_start_ms,
            window_end_ms,
            opened_at_ms,
            predicted_delta_calls,
            evidence_hash,
            stake,
            effective_stake,
            late_penalty_bps,
            late_penalty_value,
            opened_at: AS_OF.into(),
            status: PredictionStatus::Open,
            actual_delta_calls: None,
            settlement_snapshot_hash: None,
            error_bps: None,
            payout: 0,
        });

        UsagePredictionReceipt {
            position_id,
            predictor,
            epoch_id,
            subject,
            window_start_ms,
            window_end_ms,
            opened_at_ms,
            predicted_delta_calls,
            stake,
            effective_stake,
            late_penalty_bps,
            late_penalty_value,
            reward_pool_balance: state.reward_pool,
        }
    }

    #[export]
    pub fn settle_usage_prediction(
        &mut self,
        position_id: String,
        actual_delta_calls: u32,
        settlement_snapshot_hash: String,
    ) -> CommandReply<PredictionSettlement> {
        self.assert_can_upsert();
        self.assert_active("oracle is read-only; prediction settlement is disabled");
        let position_id = require_trimmed_string(position_id, "position id must be non-empty");
        let settlement_snapshot_hash = require_trimmed_string(
            settlement_snapshot_hash,
            "settlement snapshot hash must be non-empty",
        );
        let mut state = self.state.borrow_mut();
        let index = state
            .usage_predictions
            .iter()
            .position(|position| position.position_id == position_id)
            .expect("usage prediction position not found");
        assert!(
            matches!(
                state.usage_predictions[index].status,
                PredictionStatus::Open
            ),
            "usage prediction already settled"
        );
        let window_duration_ms = state.usage_predictions[index]
            .window_end_ms
            .saturating_sub(state.usage_predictions[index].window_start_ms);
        assert!(
            window_duration_ms > THREE_HOURS_MS
                || exec::block_timestamp() >= state.usage_predictions[index].window_end_ms,
            "prediction window is still open"
        );

        let predicted_delta_calls = state.usage_predictions[index].predicted_delta_calls;
        let predictor = state.usage_predictions[index].predictor;
        let stake = state.usage_predictions[index].stake;
        let effective_stake = state.usage_predictions[index].effective_stake;
        let late_penalty_bps = state.usage_predictions[index].late_penalty_bps;
        let error_bps = prediction_error_bps(predicted_delta_calls, actual_delta_calls);
        let won = error_bps <= 1_000;
        let bonus = if won {
            state.reward_pool.min(effective_stake / 2)
        } else {
            0
        };
        let payout = if won {
            effective_stake.saturating_add(bonus)
        } else {
            0
        };
        if won {
            state.reward_pool = state.reward_pool.saturating_sub(bonus);
        } else {
            state.reward_pool = state.reward_pool.saturating_add(effective_stake);
        }
        let status = if won {
            PredictionStatus::Won
        } else {
            PredictionStatus::Lost
        };

        state.usage_predictions[index].status = status.clone();
        state.usage_predictions[index].actual_delta_calls = Some(actual_delta_calls);
        state.usage_predictions[index].settlement_snapshot_hash =
            Some(settlement_snapshot_hash.clone());
        state.usage_predictions[index].error_bps = Some(error_bps);
        state.usage_predictions[index].payout = payout;

        let settlement = PredictionSettlement {
            position_id,
            subject: state.usage_predictions[index].subject.clone(),
            predicted_delta_calls,
            actual_delta_calls,
            settlement_snapshot_hash,
            error_bps,
            status,
            stake,
            effective_stake,
            late_penalty_bps,
            payout,
            reward_pool_balance: state.reward_pool,
        };

        if payout > 0 {
            msg::send(predictor, (), payout).expect("failed to send prediction payout to predictor");
        }

        CommandReply::new(settlement)
    }

    #[export]
    pub fn export_usage_predictions_chunk(
        &self,
        cursor: u32,
        limit: u32,
    ) -> UsagePredictionExportChunk {
        let predictions = self.state.borrow().usage_predictions.clone();
        let total = predictions.len();
        let start = (cursor as usize).min(total);
        let limit = normalized_export_limit(limit);
        let end = start.saturating_add(limit).min(total);

        UsagePredictionExportChunk {
            cursor: start as u32,
            limit: limit as u32,
            next_cursor: end as u32,
            done: end >= total,
            total: total as u32,
            items: predictions[start..end].to_vec(),
        }
    }

    #[export]
    pub fn set_read_only(&mut self, read_only: bool) -> ReadOnlyReceipt {
        self.assert_owner("only oracle owner can set read-only status");
        let mut state = self.state.borrow_mut();
        let previous_status = state.status();

        state.read_only = read_only;

        ReadOnlyReceipt {
            owner: state.owner,
            previous_status,
            status: state.status(),
            read_only: state.read_only,
        }
    }

    #[export]
    pub fn add_read_model_operator(&mut self, operator: ActorId) -> OperatorReceipt {
        self.assert_owner("only read-model owner can add operators");
        self.assert_active("oracle is read-only; read-model operator updates are disabled");
        assert!(
            operator != ActorId::default(),
            "read-model operator must be a non-zero actor id"
        );
        let mut state = self.state.borrow_mut();
        let mut added = false;

        if operator != state.owner && !state.operators.contains(&operator) {
            assert!(
                state.operators.len() < MAX_OPERATORS,
                "read-model operator list is full"
            );
            state.operators.push(operator);
            added = true;
        }

        OperatorReceipt {
            operator,
            added,
            removed: false,
            operator_count: state.operators.len() as u32,
        }
    }

    #[export]
    pub fn remove_read_model_operator(&mut self, operator: ActorId) -> OperatorReceipt {
        self.assert_owner("only read-model owner can remove operators");
        self.assert_active("oracle is read-only; read-model operator updates are disabled");
        let mut state = self.state.borrow_mut();
        let before = state.operators.len();
        state
            .operators
            .retain(|existing_operator| *existing_operator != operator);
        let removed = state.operators.len() != before;

        OperatorReceipt {
            operator,
            added: false,
            removed,
            operator_count: state.operators.len() as u32,
        }
    }

    #[export]
    pub fn upsert_read_model(
        &mut self,
        subject: String,
        signals: ReputationSignals,
        evidence: Vec<String>,
    ) -> ReadModelReceipt {
        self.assert_can_upsert();
        self.assert_active("oracle is read-only; read-model updates are disabled");
        let subject = require_trimmed_string(subject, "read-model subject must be non-empty");
        let evidence = normalize_evidence_labels(evidence);
        let read_model = AgentReadModel {
            subject: subject.clone(),
            signals,
            evidence,
        };
        let report = report_from_read_model(&read_model);
        let mut state = self.state.borrow_mut();

        let mut replaced = false;
        let mut evicted_subject = None;
        let subject_key = normalized_subject_key(&subject);
        if let Some(existing) = state
            .read_models
            .iter_mut()
            .find(|existing| normalized_subject_key(&existing.subject) == subject_key)
        {
            *existing = read_model;
            replaced = true;
        } else {
            if state.read_models.len() >= MAX_READ_MODELS {
                evicted_subject = Some(state.read_models.remove(0).subject);
            }
            state.read_models.push(read_model);
        }

        ReadModelReceipt {
            subject,
            replaced,
            evicted_subject,
            stored_subjects: state.read_models.len() as u32,
            report,
        }
    }

    #[export]
    pub fn record_attestation(
        &mut self,
        subject: String,
        issuer: String,
        kind: String,
        weight: i32,
        evidence_hash: String,
    ) -> AttestationReceipt {
        self.assert_active("oracle is read-only; attestation writes are disabled");
        let subject = require_trimmed_string(subject, "attestation subject must be non-empty");
        let issuer = require_trimmed_string(issuer, "attestation issuer must be non-empty");
        let kind = require_trimmed_string(kind, "attestation kind must be non-empty");
        let evidence_hash =
            require_trimmed_string(evidence_hash, "attestation evidence hash must be non-empty");
        let weight = weight.clamp(-100, 100);
        let issued_at = AS_OF.to_string();
        let weight_text = weight.to_string();
        let receipt_id = [
            subject.as_str(),
            issuer.as_str(),
            kind.as_str(),
            weight_text.as_str(),
            evidence_hash.as_str(),
            issued_at.as_str(),
        ]
        .join("|");
        let receipt = AttestationReceipt {
            receipt_id,
            subject,
            issuer,
            kind,
            weight,
            evidence_hash,
            issued_at,
        };

        let mut state = self.state.borrow_mut();
        if state.attestations.len() >= MAX_ATTESTATIONS {
            state.attestations.remove(0);
        }
        state.attestations.push(receipt.clone());

        receipt
    }

    #[export]
    pub fn get_attestations(&self, subject: String) -> Vec<AttestationReceipt> {
        let subject = require_trimmed_string(subject, "get_attestations subject must be non-empty");
        let state = self.state.borrow();
        let mut attestations = state
            .attestations
            .iter()
            .filter(|attestation| attestation.subject == subject)
            .cloned()
            .collect::<Vec<_>>();

        attestations.sort_by(|left, right| {
            right
                .issued_at
                .cmp(&left.issued_at)
                .then_with(|| left.issuer.cmp(&right.issuer))
                .then_with(|| left.kind.cmp(&right.kind))
                .then_with(|| left.evidence_hash.cmp(&right.evidence_hash))
        });

        attestations
    }

    fn report_for_subject(&self, subject: String) -> ReputationReport {
        let subject_key = normalized_subject_key(&subject);
        self.state
            .borrow()
            .read_models
            .iter()
            .find(|read_model| normalized_subject_key(&read_model.subject) == subject_key)
            .map(report_from_read_model)
            .unwrap_or_else(|| report_for_fixture_subject(subject))
    }

    fn comparable_agent(&self, input: String) -> ComparableAgent {
        let report = self.report_for_subject(input.clone());
        comparable_agent_from_report(input, report)
    }

    fn recommendation_for_subject(
        &self,
        subject: String,
        track: String,
        need: String,
    ) -> AgentRecommendation {
        let report = self.report_for_subject(subject);
        recommendation_from_report(report, track, need)
    }

    fn assert_owner(&self, message: &str) {
        let state = self.state.borrow();
        assert_eq!(msg::source(), state.owner, "{message}");
    }

    fn assert_active(&self, message: &str) {
        let state = self.state.borrow();
        assert!(!state.read_only, "{message}");
    }

    fn assert_can_upsert(&self) {
        let state = self.state.borrow();
        assert!(
            state.can_upsert(msg::source()),
            "only read-model owner or approved operator can upsert network evidence"
        );
    }
}

impl OracleState {
    fn status(&self) -> OracleStatus {
        if self.read_only {
            OracleStatus::ReadOnly
        } else {
            OracleStatus::Active
        }
    }

    fn stats(&self) -> MigrationStats {
        MigrationStats {
            operator_count: self.operators.len() as u32,
            read_model_count: self.read_models.len() as u32,
            attestation_count: self.attestations.len() as u32,
        }
    }

    fn can_upsert(&self, caller: ActorId) -> bool {
        caller == self.owner || self.operators.contains(&caller)
    }

    fn upsert_economic_profile(
        &mut self,
        agent: String,
        points_delta: i32,
        virtual_stake: u32,
        value_attached: u128,
        value_rewarded: u128,
        value_slashed: u128,
        verdict: String,
    ) -> EconomicProfile {
        let agent_key = normalized_subject_key(&agent);
        if let Some(profile) = self
            .economic_profiles
            .iter_mut()
            .find(|profile| normalized_subject_key(&profile.agent) == agent_key)
        {
            profile.oracle_points = profile.oracle_points.saturating_add(points_delta);
            profile.total_value_staked = profile.total_value_staked.saturating_add(value_attached);
            profile.total_value_rewarded =
                profile.total_value_rewarded.saturating_add(value_rewarded);
            profile.total_value_slashed = profile.total_value_slashed.saturating_add(value_slashed);
            profile.virtual_stake = profile.virtual_stake.saturating_add(virtual_stake);
            if points_delta >= 0 {
                profile.useful_signals = profile.useful_signals.saturating_add(1);
            } else {
                profile.spam_signals = profile.spam_signals.saturating_add(1);
            }
            profile.last_verdict = verdict;
            return profile.clone();
        }

        if self.economic_profiles.len() >= MAX_ECONOMIC_PROFILES {
            self.economic_profiles.sort_by(compare_economic_profiles);
            self.economic_profiles.pop();
        }

        let profile = EconomicProfile {
            agent,
            oracle_points: points_delta,
            total_value_staked: value_attached,
            total_value_rewarded: value_rewarded,
            total_value_slashed: value_slashed,
            virtual_stake,
            useful_signals: if points_delta >= 0 { 1 } else { 0 },
            spam_signals: if points_delta < 0 { 1 } else { 0 },
            last_verdict: verdict,
        };
        self.economic_profiles.push(profile.clone());
        profile
    }
}

fn migration_bounds() -> MigrationBounds {
    MigrationBounds {
        max_operators: MAX_OPERATORS as u32,
        max_read_models: MAX_READ_MODELS as u32,
        max_attestations: MAX_ATTESTATIONS as u32,
        max_evidence_labels: 12,
        max_evidence_label_chars: MAX_EVIDENCE_LABEL_CHARS as u32,
        max_export_chunk: MAX_EXPORT_CHUNK as u32,
    }
}

fn normalized_export_limit(limit: u32) -> usize {
    limit.clamp(1, MAX_EXPORT_CHUNK as u32) as usize
}

fn prediction_error_bps(predicted_delta_calls: u32, actual_delta_calls: u32) -> u32 {
    let denominator = actual_delta_calls.max(1) as u128;
    let delta = predicted_delta_calls.abs_diff(actual_delta_calls) as u128;
    delta.saturating_mul(10_000).saturating_div(denominator) as u32
}

fn prediction_late_penalty_bps(now_ms: u64, window_start_ms: u64, window_end_ms: u64) -> u32 {
    let duration = window_end_ms.saturating_sub(window_start_ms).max(1);
    let elapsed = now_ms.saturating_sub(window_start_ms).min(duration);
    let elapsed_bps = (elapsed as u128)
        .saturating_mul(10_000)
        .saturating_div(duration as u128) as u32;

    if elapsed_bps <= 5_000 {
        0
    } else {
        // No penalty in the first half of a 3h epoch, then linearly ramps to 50%.
        elapsed_bps.saturating_sub(5_000).min(5_000)
    }
}

fn signal_quality_score(report: &ReputationReport, observed_value: u32, self_loop: bool) -> u8 {
    let base = report.scores.trust_score as i32;
    let confidence = report.scores.confidence as i32 / 4;
    let observed_bonus = observed_value.min(100) as i32 / 5;
    let spam_penalty = report.scores.spam_risk as i32 / 2;
    let loop_penalty = if self_loop { 45 } else { 0 };
    score_i32(base + confidence + observed_bonus - spam_penalty - loop_penalty)
}

fn signal_points_delta(quality_score: u8, virtual_stake: u32, self_loop: bool) -> i32 {
    let stake_bonus = virtual_stake.min(1_000) as i32 / 100;
    if self_loop || quality_score < 35 {
        -((35 - quality_score.min(35)) as i32 + stake_bonus + 5)
    } else if quality_score >= 70 {
        quality_score as i32 / 2 + stake_bonus
    } else {
        quality_score as i32 / 4 + stake_bonus
    }
}

fn signal_verdict(quality_score: u8, self_loop: bool) -> &'static str {
    if self_loop || quality_score < 35 {
        "slashed"
    } else if quality_score >= 70 {
        "rewarded"
    } else {
        "accepted"
    }
}

fn signal_reason(quality_score: u8, self_loop: bool) -> &'static str {
    if self_loop {
        "self-loop or same-counterparty signal is penalized"
    } else if quality_score >= 70 {
        "high-quality evidence signal earned oracle points"
    } else if quality_score >= 35 {
        "usable evidence signal accepted with a modest reward"
    } else {
        "low-quality or spam-risk signal was penalized"
    }
}

fn compare_economic_profiles(
    left: &EconomicProfile,
    right: &EconomicProfile,
) -> core::cmp::Ordering {
    right
        .oracle_points
        .cmp(&left.oracle_points)
        .then_with(|| right.useful_signals.cmp(&left.useful_signals))
        .then_with(|| left.spam_signals.cmp(&right.spam_signals))
        .then_with(|| left.agent.cmp(&right.agent))
}

fn compare_read_models_for_export(
    left: &AgentReadModel,
    right: &AgentReadModel,
) -> core::cmp::Ordering {
    normalized_subject_key(&left.subject)
        .cmp(&normalized_subject_key(&right.subject))
        .then_with(|| left.subject.cmp(&right.subject))
}

fn compare_attestations_for_export(
    left: &AttestationReceipt,
    right: &AttestationReceipt,
) -> core::cmp::Ordering {
    normalized_subject_key(&left.subject)
        .cmp(&normalized_subject_key(&right.subject))
        .then_with(|| left.issued_at.cmp(&right.issued_at))
        .then_with(|| left.issuer.cmp(&right.issuer))
        .then_with(|| left.kind.cmp(&right.kind))
        .then_with(|| left.evidence_hash.cmp(&right.evidence_hash))
        .then_with(|| left.receipt_id.cmp(&right.receipt_id))
}

fn report_for_fixture_subject(subject: String) -> ReputationReport {
    match subject.as_str() {
        "@registry-helper" => registry_helper_report(subject),
        "@quiet-indexer" => quiet_indexer_report(subject),
        "@loop-farm" => loop_farm_report(subject),
        _ => unresolved_report(subject),
    }
}

fn comparable_agent_from_report(input: String, report: ReputationReport) -> ComparableAgent {
    let reason = match report.scores.verdict.as_str() {
        "recommended" => "best available candidate for this need",
        "review" => "candidate is usable but needs manual review",
        _ => "candidate should wait until risk or missing evidence improves",
    }
    .into();

    ComparableAgent {
        input,
        subject: report.subject.clone(),
        verdict: report.scores.verdict.clone(),
        trust_score: report.scores.trust_score,
        confidence: report.scores.confidence,
        spam_risk: report.scores.spam_risk,
        reason,
        report,
    }
}

fn recommendation_from_report(
    report: ReputationReport,
    track: String,
    need: String,
) -> AgentRecommendation {
    let reason = recommendation_reason(&report);

    AgentRecommendation {
        subject: report.subject.clone(),
        track,
        need,
        verdict: report.scores.verdict.clone(),
        trust_score: report.scores.trust_score,
        confidence: report.scores.confidence,
        spam_risk: report.scores.spam_risk,
        explanation: RecommendationExplanation {
            summary: reason.clone(),
            positive_signals: positive_signals(&report),
            negative_signals: negative_signals(&report),
        },
        reason,
        report,
    }
}

fn report_from_read_model(read_model: &AgentReadModel) -> ReputationReport {
    let scores = scores_from_signals(&read_model.signals, read_model.evidence.len() as u32);
    ReputationReport {
        subject: read_model.subject.clone(),
        as_of: AS_OF.into(),
        report_version: 2,
        integration_decision: decision_for_scores(&scores, &read_model.signals),
        scores,
        signals: read_model.signals.clone(),
        evidence: read_model.evidence.clone(),
    }
}

fn scores_from_signals(signals: &ReputationSignals, _evidence_count: u32) -> ReputationScores {
    let incoming_activity = capped_linear_scaled(signals.incoming_unique_callers, 16, 80);
    let outgoing_activity = capped_linear_scaled(signals.outgoing_meaningful_calls, 8, 20);
    let chat_activity = capped_linear_scaled(signals.chat_board_updates, 10, 10);
    let activity = incoming_activity
        .saturating_add(outgoing_activity)
        .saturating_add(chat_activity)
        .min(100_000);

    let identity = ((if signals.has_identity_card {
        55u32
    } else {
        0u32
    }) + (if signals.has_verified_social_proof {
        25u32
    } else {
        0u32
    }))
    .saturating_sub(signals.missing_required_metadata.saturating_mul(10))
    .min(100);
    let identity_scaled = identity.saturating_mul(1_000);

    let integration = capped_linear_scaled(signals.incoming_unique_callers, 16, 80)
        .saturating_add(capped_linear_scaled(
            signals.outgoing_meaningful_calls,
            8,
            40,
        ))
        .min(100_000);

    let outgoing_imbalance = signals
        .outgoing_meaningful_calls
        .saturating_sub(signals.incoming_unique_callers.saturating_mul(3));
    let spam_risk = signals
        .circular_call_signals
        .saturating_mul(22)
        .saturating_add(outgoing_imbalance.saturating_mul(4))
        .saturating_add(signals.missing_required_metadata.saturating_mul(8))
        .min(100);
    let spam_risk_scaled = spam_risk.saturating_mul(1_000);

    let confidence = 35_000u32
        .saturating_add(capped_linear_scaled(
            signals.incoming_unique_callers,
            10,
            25,
        ))
        .saturating_add(capped_linear_scaled(
            signals.outgoing_meaningful_calls,
            8,
            20,
        ))
        .saturating_add(if signals.has_identity_card { 15_000 } else { 0 })
        .saturating_add(if signals.has_verified_social_proof {
            15_000
        } else {
            0
        })
        .saturating_sub(signals.missing_required_metadata.saturating_mul(8_000))
        .min(100_000);

    let trust_scaled = 10_000i64
        + (activity as i64 * 35 / 100)
        + (identity_scaled as i64 * 25 / 100)
        + (integration as i64 * 30 / 100)
        - (spam_risk_scaled as i64 * 25 / 100);

    let trust_score = round_scaled_score(trust_scaled);
    let activity_score = round_scaled_score(activity as i64);
    let identity_score = round_scaled_score(identity_scaled as i64);
    let integration_score = round_scaled_score(integration as i64);
    let spam_risk = round_scaled_score(spam_risk_scaled as i64);
    let confidence = round_scaled_score(confidence as i64);
    let verdict = if spam_risk >= 70 {
        "avoid_or_wait"
    } else if trust_score >= 70 && spam_risk < 35 {
        "recommended"
    } else if trust_score >= 35 {
        "review"
    } else {
        "avoid_or_wait"
    };

    ReputationScores {
        trust_score,
        activity_score,
        integration_score,
        identity_score,
        spam_risk,
        confidence,
        verdict: verdict.into(),
    }
}

fn scores_v2_from_signals(signals: &ReputationSignalsV2) -> ReputationScoresV2 {
    let coordination = signals
        .posts_active
        .saturating_mul(5)
        .saturating_add(
            signals
                .mention_count
                .saturating_add(signals.messages_sent)
                .min(5),
        )
        .min(15);
    let ecosystem_value_score = score_u32(
        (if signals.has_clear_board_description {
            35
        } else {
            0
        }) + (if signals.has_idl_url { 20 } else { 0 })
            + (if signals.has_skills_url { 15 } else { 0 })
            + (if signals.has_identity_card { 15 } else { 0 })
            + coordination,
    );

    let real_integration_score = score_u32(
        signals
            .inbound_unique_participants
            .min(10)
            .saturating_mul(4)
            + signals
                .outbound_unique_participants
                .min(10)
                .saturating_mul(3)
            + signals.inbound_call_count.min(50) / 5
            + signals.outbound_call_count.min(50) / 5
            + if signals.inbound_unique_participants >= 2
                && signals.outbound_unique_participants >= 2
            {
                10
            } else {
                0
            },
    );

    let unique_total = signals
        .inbound_unique_participants
        .max(signals.outbound_unique_participants);
    let balance_score =
        if signals.inbound_unique_participants >= 2 && signals.outbound_unique_participants >= 2 {
            20
        } else if unique_total >= 2 {
            10
        } else {
            0
        };
    let counterparty_diversity_score = score_u32(
        unique_total.min(10).saturating_mul(7)
            + balance_score
            + if signals.call_graph_density_bps < 5_000 {
                10
            } else {
                0
            },
    );

    let identity_provenance_score = score_u32(
        (if signals.handle_claimed { 25 } else { 0 })
            + (if signals.has_identity_card { 25 } else { 0 })
            + (if matches!(
                signals.application_status,
                ApplicationStatusV2::Submitted | ApplicationStatusV2::Approved
            ) {
                20
            } else {
                0
            })
            + (if signals.registered_at > 0 { 15 } else { 0 })
            + (if signals.identity_updated_at > 0 {
                15
            } else {
                0
            }),
    );

    let demo_readiness_score = score_u32(
        (if signals.has_clear_board_description {
            30
        } else {
            0
        }) + (if signals.has_actual_github_repo {
            25
        } else {
            0
        }) + (if signals.has_frontend_url { 25 } else { 0 })
            + (if signals.has_idl_url { 10 } else { 0 })
            + (if signals.has_skills_url { 10 } else { 0 }),
    );

    let valid_participant_calls = signals
        .valid_participant_inbound_count
        .saturating_add(signals.valid_participant_outbound_count);
    let mut risk_points = 0u32;
    if signals.self_loop_call_count > 0 {
        risk_points = risk_points.saturating_add(35);
    }
    if signals.same_owner_call_count > 0 {
        risk_points = risk_points.saturating_add(25);
    }
    if valid_participant_calls > 0 && signals.non_participant_call_count > valid_participant_calls {
        risk_points = risk_points.saturating_add(25);
    }
    if signals.call_graph_density_bps >= 7_500 {
        risk_points = risk_points.saturating_add(20);
    }
    if signals.low_diversity_volume_count >= 25 {
        risk_points = risk_points.saturating_add(15);
    }
    if signals.outbound_call_count >= 30 && signals.outbound_unique_participants < 3 {
        risk_points = risk_points.saturating_add(15);
    }
    if signals.inbound_call_count >= 30 && signals.inbound_unique_participants < 3 {
        risk_points = risk_points.saturating_add(10);
    }
    if !signals.has_clear_board_description
        && !signals.has_identity_card
        && !signals.has_idl_url
        && !signals.has_skills_url
        && signals
            .inbound_call_count
            .saturating_add(signals.outbound_call_count)
            >= 30
    {
        risk_points = risk_points.saturating_add(10);
    }
    risk_points = risk_points.saturating_add(
        signals
            .negative_third_party_attestations
            .saturating_mul(25)
            .min(50),
    );
    risk_points = risk_points.saturating_sub(
        signals
            .positive_third_party_attestations
            .saturating_mul(5)
            .min(15),
    );
    let spam_risk = score_u32(risk_points);
    let safety_score = 100u8.saturating_sub(spam_risk);

    let metadata_fields = (if signals.has_clear_board_description {
        1
    } else {
        0
    }) + (if signals.has_actual_github_repo { 1 } else { 0 })
        + (if signals.has_skills_url { 1 } else { 0 })
        + (if signals.has_idl_url { 1 } else { 0 });
    let confidence_score = score_u32(
        25 + if signals.metrics_updated_at > 0 {
            20
        } else {
            0
        } + if signals.has_identity_card { 15 } else { 0 }
            + if signals.handle_claimed { 10 } else { 0 }
            + metadata_fields * 15 / 4
            + if !matches!(signals.participant_status, ParticipantStatusV2::Unknown) {
                10
            } else {
                0
            },
    );

    let weighted_overall = (ecosystem_value_score as u32).saturating_mul(20)
        + (real_integration_score as u32).saturating_mul(25)
        + (counterparty_diversity_score as u32).saturating_mul(15)
        + (identity_provenance_score as u32).saturating_mul(10)
        + (demo_readiness_score as u32).saturating_mul(15)
        + (safety_score as u32).saturating_mul(15);
    let overall_score = score_u32((weighted_overall + 50) / 100);

    let verdict = if matches!(
        signals.application_status,
        ApplicationStatusV2::Rejected | ApplicationStatusV2::Suspended
    ) || spam_risk >= 70
    {
        "avoid_or_wait"
    } else if overall_score >= 75
        && spam_risk < 30
        && confidence_score >= 65
        && real_integration_score >= 55
    {
        "recommended"
    } else if overall_score >= 45 && spam_risk < 70 {
        "review"
    } else {
        "avoid_or_wait"
    };

    ReputationScoresV2 {
        ecosystem_value_score,
        real_integration_score,
        counterparty_diversity_score,
        identity_provenance_score,
        demo_readiness_score,
        safety_score,
        spam_risk,
        confidence_score,
        overall_score,
        verdict: verdict.into(),
    }
}

fn score_u32(value: u32) -> u8 {
    value.min(100) as u8
}

fn score_i32(value: i32) -> u8 {
    value.clamp(0, 100) as u8
}

fn capped_linear_scaled(value: u32, cap: u32, max_score: u32) -> u32 {
    if value == 0 || cap == 0 {
        return 0;
    }
    value
        .min(cap)
        .saturating_mul(max_score)
        .saturating_mul(1_000)
        / cap
}

fn round_scaled_score(value: i64) -> u8 {
    ((value + 500) / 1_000).clamp(0, 100) as u8
}

fn require_trimmed_string(value: String, message: &str) -> String {
    let trimmed = value.trim().to_string();
    assert!(!trimmed.is_empty(), "{message}");
    trimmed
}

fn normalized_subject_key(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

fn normalize_evidence_labels(evidence: Vec<String>) -> Vec<String> {
    let mut labels = evidence
        .into_iter()
        .map(|label| {
            label
                .trim()
                .chars()
                .take(MAX_EVIDENCE_LABEL_CHARS)
                .collect::<String>()
        })
        .filter(|label| !label.is_empty())
        .collect::<Vec<_>>();
    labels.sort();
    labels.dedup();
    labels.truncate(12);
    labels
}

fn decision_for_scores(
    scores: &ReputationScores,
    signals: &ReputationSignals,
) -> IntegrationDecision {
    match scores.verdict.as_str() {
        "recommended" => IntegrationDecision {
            action: "integrate".into(),
            risk_level: if scores.spam_risk >= 20 {
                "medium".into()
            } else {
                "low".into()
            },
            max_attestation_weight: round_div_i32(scores.confidence as i32, 2).clamp(10, 50),
            next_steps: vec![
                "call read-only route first".into(),
                "record bounded integration_success attestation after verified outcome".into(),
            ],
        },
        "review" => IntegrationDecision {
            action: "manual_review".into(),
            risk_level: if scores.spam_risk >= 35 {
                "high".into()
            } else {
                "medium".into()
            },
            max_attestation_weight: round_div_i32(scores.confidence as i32, 4).clamp(0, 25),
            next_steps: review_next_steps(signals),
        },
        _ => IntegrationDecision {
            action: "wait".into(),
            risk_level: "high".into(),
            max_attestation_weight: 0,
            next_steps: wait_next_steps(signals),
        },
    }
}

fn review_next_steps(signals: &ReputationSignals) -> Vec<String> {
    let mut steps = Vec::new();
    if !signals.has_identity_card {
        steps.push("verify board identity card".into());
    }
    if !signals.has_verified_social_proof {
        steps.push("verify external metadata".into());
    }
    if signals.incoming_unique_callers == 0 {
        steps.push("wait for independent inbound callers".into());
    }
    if signals.circular_call_signals > 0 {
        steps.push("inspect reciprocal call pattern".into());
    }
    if signals.negative_attestations > 0 {
        steps.push("review negative attestations".into());
    }
    steps.truncate(3);
    steps
}

fn round_div_i32(value: i32, divisor: i32) -> i32 {
    if divisor <= 0 {
        return value;
    }
    (value + divisor / 2) / divisor
}

fn wait_next_steps(signals: &ReputationSignals) -> Vec<String> {
    let mut steps = Vec::new();
    if signals.circular_call_signals > 0 {
        steps.push("reduce circular or self-call activity".into());
    }
    if signals.missing_required_metadata > 0 {
        steps.push("complete registry metadata and identity fields".into());
    }
    if signals.negative_attestations > 0 {
        steps.push("resolve negative attestations with evidence".into());
    }
    if signals.incoming_unique_callers == 0 {
        steps.push("earn independent inbound callers".into());
    }
    if steps.is_empty() {
        steps.push("collect more network evidence before integration".into());
    }
    steps.truncate(3);
    steps
}

fn compare_candidates(left: &ComparableAgent, right: &ComparableAgent) -> core::cmp::Ordering {
    verdict_rank(&right.verdict)
        .cmp(&verdict_rank(&left.verdict))
        .then_with(|| right.trust_score.cmp(&left.trust_score))
        .then_with(|| right.confidence.cmp(&left.confidence))
        .then_with(|| left.spam_risk.cmp(&right.spam_risk))
        .then_with(|| left.subject.cmp(&right.subject))
}

fn compare_recommendations(
    left: &AgentRecommendation,
    right: &AgentRecommendation,
) -> core::cmp::Ordering {
    verdict_rank(&right.verdict)
        .cmp(&verdict_rank(&left.verdict))
        .then_with(|| right.trust_score.cmp(&left.trust_score))
        .then_with(|| right.confidence.cmp(&left.confidence))
        .then_with(|| left.spam_risk.cmp(&right.spam_risk))
        .then_with(|| left.subject.cmp(&right.subject))
}

fn verdict_rank(verdict: &str) -> u8 {
    match verdict {
        "recommended" => 3,
        "review" => 2,
        _ => 1,
    }
}

fn recommendation_reason(report: &ReputationReport) -> String {
    match report.scores.verdict.as_str() {
        "recommended" => "strong trust score with acceptable spam risk".into(),
        "review" => "usable candidate with incomplete confidence or mixed signals".into(),
        _ => "high spam risk or insufficient trust signals".into(),
    }
}

fn positive_signals(report: &ReputationReport) -> Vec<String> {
    match report.subject.as_str() {
        "@registry-helper" => vec![
            "published identity card".into(),
            "declared external metadata".into(),
            "3 unique inbound callers".into(),
        ],
        "@quiet-indexer" => vec![
            "published identity card".into(),
            "1 chat/board update".into(),
            "1 meaningful outbound call".into(),
        ],
        "@loop-farm" => vec![
            "2 meaningful outbound calls".into(),
            "1 chat/board update".into(),
            "1 unique inbound caller".into(),
        ],
        _ => {
            let mut candidates = Vec::new();
            if report.signals.incoming_unique_callers > 0 {
                candidates.push((
                    report
                        .signals
                        .incoming_unique_callers
                        .min(16)
                        .saturating_mul(6),
                    format!(
                        "{} unique inbound callers",
                        report.signals.incoming_unique_callers
                    ),
                ));
            }
            if report.signals.outgoing_meaningful_calls > 0 {
                candidates.push((
                    report
                        .signals
                        .outgoing_meaningful_calls
                        .min(8)
                        .saturating_mul(5),
                    format!(
                        "{} meaningful outbound calls",
                        report.signals.outgoing_meaningful_calls
                    ),
                ));
            }
            if report.signals.chat_board_updates > 0 {
                candidates.push((
                    report.signals.chat_board_updates.min(10).saturating_mul(2),
                    format!("{} chat/board updates", report.signals.chat_board_updates),
                ));
            }
            if report.signals.has_identity_card {
                candidates.push((35, "published identity card".into()));
            }
            if report.signals.has_verified_social_proof {
                candidates.push((30, "declared external metadata".into()));
            }
            if report.signals.positive_attestations > 0 {
                candidates.push((
                    report.signals.positive_attestations.saturating_mul(12),
                    format!(
                        "{} positive attestations",
                        report.signals.positive_attestations
                    ),
                ));
            }

            sort_signal_candidates(candidates)
        }
    }
}

fn negative_signals(report: &ReputationReport) -> Vec<String> {
    match report.subject.as_str() {
        "@loop-farm" => vec![
            "spam risk 68".into(),
            "2 circular call signals".into(),
            "3 missing metadata fields".into(),
        ],
        subject
            if subject != "@registry-helper"
                && subject != "@quiet-indexer"
                && report.evidence.first().map(|evidence| evidence.as_str())
                    == Some("fixture: no matching read-model subject") =>
        {
            vec![
                "5 missing metadata fields".into(),
                "missing identity card".into(),
                "spam risk 40".into(),
            ]
        }
        _ => {
            let outgoing_imbalance = report
                .signals
                .outgoing_meaningful_calls
                .saturating_sub(report.signals.incoming_unique_callers.saturating_mul(3));
            let mut candidates = Vec::new();
            if report.scores.spam_risk >= 35 {
                candidates.push((
                    report.scores.spam_risk as u32,
                    format!("spam risk {}", report.scores.spam_risk),
                ));
            }
            if report.signals.circular_call_signals > 0 {
                candidates.push((
                    report.signals.circular_call_signals.saturating_mul(30),
                    format!(
                        "{} circular call signals",
                        report.signals.circular_call_signals
                    ),
                ));
            }
            if report.signals.missing_required_metadata > 0 {
                candidates.push((
                    report.signals.missing_required_metadata.saturating_mul(14),
                    format!(
                        "{} missing metadata fields",
                        report.signals.missing_required_metadata
                    ),
                ));
            }
            if report.signals.negative_attestations > 0 {
                candidates.push((
                    report.signals.negative_attestations.saturating_mul(18),
                    format!(
                        "{} negative attestations",
                        report.signals.negative_attestations
                    ),
                ));
            }
            if !report.signals.has_identity_card {
                candidates.push((16, "missing identity card".into()));
            }
            if outgoing_imbalance > 0 {
                candidates.push((
                    outgoing_imbalance.saturating_mul(8),
                    format!("{} excess outbound calls", outgoing_imbalance),
                ));
            }

            sort_signal_candidates(candidates)
        }
    }
}

fn sort_signal_candidates(mut candidates: Vec<(u32, String)>) -> Vec<String> {
    candidates.retain(|candidate| candidate.0 > 0);
    candidates.sort_by(|left, right| right.0.cmp(&left.0).then_with(|| left.1.cmp(&right.1)));
    candidates.truncate(3);
    candidates
        .into_iter()
        .map(|(_, label)| label)
        .collect::<Vec<_>>()
}

fn registry_helper_report(subject: String) -> ReputationReport {
    ReputationReport {
        subject,
        as_of: AS_OF.into(),
        report_version: 2,
        scores: ReputationScores {
            trust_score: 48,
            activity_score: 26,
            integration_score: 30,
            identity_score: 80,
            spam_risk: 0,
            confidence: 80,
            verdict: "review".into(),
        },
        signals: ReputationSignals {
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
        integration_decision: IntegrationDecision {
            action: "manual_review".into(),
            risk_level: "medium".into(),
            max_attestation_weight: 20,
            next_steps: vec![],
        },
        evidence: vec![
            "board:identity".into(),
            "calls:incoming".into(),
            "calls:outgoing".into(),
            "chat-board:activity".into(),
            "registry:application".into(),
            "social:verified".into(),
        ],
    }
}

fn quiet_indexer_report(subject: String) -> ReputationReport {
    ReputationReport {
        subject,
        as_of: AS_OF.into(),
        report_version: 2,
        scores: ReputationScores {
            trust_score: 30,
            activity_score: 9,
            integration_score: 10,
            identity_score: 55,
            spam_risk: 0,
            confidence: 55,
            verdict: "avoid_or_wait".into(),
        },
        signals: ReputationSignals {
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
        integration_decision: IntegrationDecision {
            action: "wait".into(),
            risk_level: "high".into(),
            max_attestation_weight: 0,
            next_steps: vec!["collect more network evidence before integration".into()],
        },
        evidence: vec![
            "board:identity".into(),
            "calls:incoming".into(),
            "calls:outgoing".into(),
            "chat-board:activity".into(),
            "registry:application".into(),
        ],
    }
}

fn loop_farm_report(subject: String) -> ReputationReport {
    ReputationReport {
        subject,
        as_of: AS_OF.into(),
        report_version: 2,
        scores: ReputationScores {
            trust_score: 1,
            activity_score: 11,
            integration_score: 15,
            identity_score: 0,
            spam_risk: 68,
            confidence: 19,
            verdict: "avoid_or_wait".into(),
        },
        signals: ReputationSignals {
            incoming_unique_callers: 1,
            outgoing_meaningful_calls: 2,
            chat_board_updates: 1,
            has_identity_card: false,
            has_verified_social_proof: false,
            circular_call_signals: 2,
            missing_required_metadata: 3,
            positive_attestations: 0,
            negative_attestations: 1,
        },
        integration_decision: IntegrationDecision {
            action: "wait".into(),
            risk_level: "high".into(),
            max_attestation_weight: 0,
            next_steps: vec![
                "wait for non-circular integration evidence".into(),
                "require identity and metadata before integration".into(),
            ],
        },
        evidence: vec![
            "calls:incoming".into(),
            "calls:outgoing".into(),
            "chat-board:activity".into(),
            "registry:application".into(),
        ],
    }
}

fn unresolved_report(subject: String) -> ReputationReport {
    ReputationReport {
        subject,
        as_of: AS_OF.into(),
        report_version: 2,
        scores: ReputationScores {
            trust_score: 30,
            activity_score: 0,
            integration_score: 0,
            identity_score: 0,
            spam_risk: 40,
            confidence: 15,
            verdict: "review".into(),
        },
        signals: ReputationSignals {
            incoming_unique_callers: 0,
            outgoing_meaningful_calls: 0,
            chat_board_updates: 0,
            has_identity_card: false,
            has_verified_social_proof: false,
            circular_call_signals: 0,
            missing_required_metadata: 5,
            positive_attestations: 0,
            negative_attestations: 0,
        },
        integration_decision: IntegrationDecision {
            action: "manual_review".into(),
            risk_level: "medium".into(),
            max_attestation_weight: 0,
            next_steps: vec!["resolve Registry handle or program ID".into()],
        },
        evidence: vec!["fixture: no matching read-model subject".into()],
    }
}

pub struct Program {
    state: RefCell<OracleState>,
}

#[sails_rs::program]
impl Program {
    // Program's constructor
    pub fn create() -> Self {
        Self {
            state: RefCell::new(OracleState {
                owner: msg::source(),
                ..OracleState::default()
            }),
        }
    }

    // Exposed service
    pub fn reputation_oracle(&self) -> ReputationOracle<'_> {
        ReputationOracle::create(&self.state)
    }
}
