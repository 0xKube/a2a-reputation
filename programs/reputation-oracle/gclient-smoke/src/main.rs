use std::{env, fs, net::TcpStream, path::PathBuf, time::Duration};

use gclient::{GearApi, WSAddress};
use reputation_oracle_client::{
    ApplicationStatusV2, ParticipantStatusV2, PredictionStatus, ReputationOracleClient,
    ReputationOracleClientCtors, ReputationOracleClientProgram, ReputationSignals,
    ReputationSignalsV2,
    reputation_oracle::{ReputationOracle, io::ScoreAgentV2},
};

#[cfg(test)]
use reputation_oracle_client::{ReputationReportV2, ReputationScoresV2};
use sails_rs::client::{GclientEnv, GearEnv};
use sails_rs::scale_codec::Decode;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct PreviewPacket {
    subject: String,
    signals: PreviewSignals,
    evidence: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct PreviewSignals {
    incoming_unique_callers: u32,
    outgoing_meaningful_calls: u32,
    chat_board_updates: u32,
    has_identity_card: bool,
    has_verified_social_proof: bool,
    circular_call_signals: u32,
    missing_required_metadata: u32,
    positive_attestations: u32,
    negative_attestations: u32,
}

#[derive(Debug, Deserialize)]
struct PreviewPacketV2 {
    subject: String,
    #[serde(rename = "signalsV2")]
    signals_v2: PreviewSignalsV2,
    #[serde(rename = "evidenceV2")]
    evidence_v2: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct PreviewSignalsV2 {
    inbound_unique_participants: u32,
    inbound_call_count: u32,
    outbound_unique_participants: u32,
    outbound_call_count: u32,
    valid_participant_inbound_count: u32,
    valid_participant_outbound_count: u32,
    non_participant_call_count: u32,
    self_loop_call_count: u32,
    same_owner_call_count: u32,
    mention_count: u32,
    messages_sent: u32,
    posts_active: u32,
    has_clear_board_description: bool,
    has_identity_card: bool,
    has_actual_github_repo: bool,
    has_skills_url: bool,
    has_idl_url: bool,
    has_frontend_url: bool,
    handle_claimed: bool,
    application_status: String,
    participant_status: String,
    call_graph_density_bps: u32,
    low_diversity_volume_count: u32,
    reciprocal_farming_signals: u32,
    positive_third_party_attestations: u32,
    negative_third_party_attestations: u32,
    metrics_updated_at: u64,
    identity_updated_at: u64,
    registered_at: u64,
}

fn application_status_v2(value: &str) -> Result<ApplicationStatusV2, String> {
    match value {
        "Unknown" => Ok(ApplicationStatusV2::Unknown),
        "Draft" => Ok(ApplicationStatusV2::Draft),
        "Submitted" => Ok(ApplicationStatusV2::Submitted),
        "Approved" => Ok(ApplicationStatusV2::Approved),
        "Rejected" => Ok(ApplicationStatusV2::Rejected),
        "Suspended" => Ok(ApplicationStatusV2::Suspended),
        other => Err(format!(
            "unsupported application_status for GraphQL V2 packet: {other}"
        )),
    }
}

fn participant_status_v2(value: &str) -> Result<ParticipantStatusV2, String> {
    match value {
        "Unknown" => Ok(ParticipantStatusV2::Unknown),
        "ParticipantLike" => Ok(ParticipantStatusV2::ParticipantLike),
        "VerifiedParticipant" => Ok(ParticipantStatusV2::VerifiedParticipant),
        "NonParticipant" => Ok(ParticipantStatusV2::NonParticipant),
        other => Err(format!(
            "unsupported participant_status for GraphQL V2 packet: {other}"
        )),
    }
}

impl TryFrom<PreviewSignalsV2> for ReputationSignalsV2 {
    type Error = String;

    fn try_from(signals: PreviewSignalsV2) -> Result<Self, Self::Error> {
        Ok(Self {
            inbound_unique_participants: signals.inbound_unique_participants,
            inbound_call_count: signals.inbound_call_count,
            outbound_unique_participants: signals.outbound_unique_participants,
            outbound_call_count: signals.outbound_call_count,
            valid_participant_inbound_count: signals.valid_participant_inbound_count,
            valid_participant_outbound_count: signals.valid_participant_outbound_count,
            non_participant_call_count: signals.non_participant_call_count,
            self_loop_call_count: signals.self_loop_call_count,
            same_owner_call_count: signals.same_owner_call_count,
            mention_count: signals.mention_count,
            messages_sent: signals.messages_sent,
            posts_active: signals.posts_active,
            has_clear_board_description: signals.has_clear_board_description,
            has_identity_card: signals.has_identity_card,
            has_actual_github_repo: signals.has_actual_github_repo,
            has_skills_url: signals.has_skills_url,
            has_idl_url: signals.has_idl_url,
            has_frontend_url: signals.has_frontend_url,
            handle_claimed: signals.handle_claimed,
            application_status: application_status_v2(&signals.application_status)?,
            participant_status: participant_status_v2(&signals.participant_status)?,
            call_graph_density_bps: signals.call_graph_density_bps,
            low_diversity_volume_count: signals.low_diversity_volume_count,
            reciprocal_farming_signals: signals.reciprocal_farming_signals,
            positive_third_party_attestations: signals.positive_third_party_attestations,
            negative_third_party_attestations: signals.negative_third_party_attestations,
            metrics_updated_at: signals.metrics_updated_at,
            identity_updated_at: signals.identity_updated_at,
            registered_at: signals.registered_at,
        })
    }
}

impl From<PreviewSignals> for ReputationSignals {
    fn from(signals: PreviewSignals) -> Self {
        Self {
            incoming_unique_callers: signals.incoming_unique_callers,
            outgoing_meaningful_calls: signals.outgoing_meaningful_calls,
            chat_board_updates: signals.chat_board_updates,
            has_identity_card: signals.has_identity_card,
            has_verified_social_proof: signals.has_verified_social_proof,
            circular_call_signals: signals.circular_call_signals,
            missing_required_metadata: signals.missing_required_metadata,
            positive_attestations: signals.positive_attestations,
            negative_attestations: signals.negative_attestations,
        }
    }
}

fn local_ws_address() -> WSAddress {
    let endpoint = env::var("VARA_LOCAL_WS").unwrap_or_else(|_| "ws://127.0.0.1:9944".into());
    let endpoint = endpoint.strip_suffix('/').unwrap_or(&endpoint);

    if endpoint == "ws://127.0.0.1:9944" || endpoint == "ws://localhost:9944" {
        return WSAddress::dev();
    }

    let (scheme_host, port) = endpoint
        .rsplit_once(':')
        .and_then(|(scheme_host, port)| port.parse::<u16>().ok().map(|port| (scheme_host, port)))
        .unwrap_or_else(|| {
            std::panic!("VARA_LOCAL_WS must look like ws://host:port, got {endpoint}")
        });

    WSAddress::new(scheme_host, port)
}

fn endpoint_is_reachable() -> bool {
    let endpoint = env::var("VARA_LOCAL_WS").unwrap_or_else(|_| "ws://127.0.0.1:9944".into());
    let endpoint = endpoint
        .trim_start_matches("ws://")
        .trim_start_matches("wss://")
        .trim_end_matches('/');
    let socket = endpoint.rsplit_once(':').unwrap_or((endpoint, "9944"));
    let Ok(port) = socket.1.parse::<u16>() else {
        return false;
    };
    let host = if socket.0 == "localhost" {
        "127.0.0.1"
    } else {
        socket.0
    };
    let Ok(addr) = format!("{host}:{port}").parse() else {
        return false;
    };

    TcpStream::connect_timeout(&addr, Duration::from_millis(250)).is_ok()
}

fn optimized_wasm_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join(".outputs/deploy/reputation_oracle.opt.wasm")
}

fn sample_signals() -> ReputationSignals {
    ReputationSignals {
        incoming_unique_callers: 12,
        outgoing_meaningful_calls: 8,
        chat_board_updates: 3,
        has_identity_card: true,
        has_verified_social_proof: true,
        circular_call_signals: 0,
        missing_required_metadata: 0,
        positive_attestations: 2,
        negative_attestations: 0,
    }
}

fn preview_packet_from_env() -> Result<Option<PreviewPacket>, Box<dyn std::error::Error>> {
    let Some(path) = env::var("GRAPHQL_PREVIEW_PACKET_PATH").ok() else {
        return Ok(None);
    };
    let text = fs::read_to_string(&path)?;
    let packet: PreviewPacket = serde_json::from_str(&text)?;
    if packet.subject.trim().is_empty() {
        return Err("GRAPHQL_PREVIEW_PACKET_PATH packet subject must be non-empty".into());
    }
    Ok(Some(packet))
}

fn preview_packet_v2_from_env() -> Result<Option<PreviewPacketV2>, Box<dyn std::error::Error>> {
    let Some(path) = env::var("GRAPHQL_PREVIEW_PACKET_V2_PATH").ok() else {
        return Ok(None);
    };
    let text = fs::read_to_string(&path)?;
    let packet: PreviewPacketV2 = serde_json::from_str(&text)?;
    if packet.subject.trim().is_empty() {
        return Err("GRAPHQL_PREVIEW_PACKET_V2_PATH packet subject must be non-empty".into());
    }
    Ok(Some(packet))
}

#[cfg(test)]
fn sample_v2_signals() -> ReputationSignalsV2 {
    ReputationSignalsV2 {
        inbound_unique_participants: 4,
        inbound_call_count: 11,
        outbound_unique_participants: 3,
        outbound_call_count: 7,
        valid_participant_inbound_count: 9,
        valid_participant_outbound_count: 6,
        non_participant_call_count: 1,
        self_loop_call_count: 0,
        same_owner_call_count: 1,
        mention_count: 5,
        messages_sent: 12,
        posts_active: 2,
        has_clear_board_description: true,
        has_identity_card: true,
        has_actual_github_repo: true,
        has_skills_url: true,
        has_idl_url: true,
        has_frontend_url: false,
        handle_claimed: true,
        application_status: ApplicationStatusV2::Submitted,
        participant_status: ParticipantStatusV2::VerifiedParticipant,
        call_graph_density_bps: 1_250,
        low_diversity_volume_count: 1,
        reciprocal_farming_signals: 0,
        positive_third_party_attestations: 2,
        negative_third_party_attestations: 0,
        metrics_updated_at: 1_782_094_400,
        identity_updated_at: 1_782_000_000,
        registered_at: 1_781_900_000,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sails_rs::scale_codec::Encode;

    #[test]
    fn v2_signals_scale_round_trip_preserves_all_fields() {
        let signals = sample_v2_signals();
        let encoded = signals.encode();
        let decoded = ReputationSignalsV2::decode(&mut &encoded[..]).expect("decode signals v2");

        assert_eq!(decoded, signals);
        assert_eq!(decoded.application_status, ApplicationStatusV2::Submitted);
        assert_eq!(
            decoded.participant_status,
            ParticipantStatusV2::VerifiedParticipant
        );
        assert!(decoded.has_actual_github_repo);
        assert!(decoded.has_idl_url);
        assert_eq!(decoded.call_graph_density_bps, 1_250);
    }

    #[test]
    fn graphql_preview_v2_packet_maps_to_score_agent_v2_payload() {
        let packet: PreviewPacketV2 = serde_json::from_str(
            r#"{
              "subject":"@fixture-agent",
              "signalsV2":{
                "inbound_unique_participants":3,
                "inbound_call_count":9,
                "outbound_unique_participants":2,
                "outbound_call_count":4,
                "valid_participant_inbound_count":0,
                "valid_participant_outbound_count":0,
                "non_participant_call_count":0,
                "self_loop_call_count":0,
                "same_owner_call_count":0,
                "mention_count":2,
                "messages_sent":1,
                "posts_active":1,
                "has_clear_board_description":true,
                "has_identity_card":true,
                "has_actual_github_repo":true,
                "has_skills_url":true,
                "has_idl_url":true,
                "has_frontend_url":false,
                "handle_claimed":true,
                "application_status":"Submitted",
                "participant_status":"ParticipantLike",
                "call_graph_density_bps":500,
                "low_diversity_volume_count":0,
                "reciprocal_farming_signals":0,
                "positive_third_party_attestations":0,
                "negative_third_party_attestations":0,
                "metrics_updated_at":1779408000000,
                "identity_updated_at":1779210381001,
                "registered_at":1779210312000
              },
              "evidenceV2":["source:vara-agent-network/graphql","unknown:self-loop-count"]
            }"#,
        )
        .expect("decode preview packet v2");
        let signals = ReputationSignalsV2::try_from(packet.signals_v2).expect("map signals v2");
        let encoded = ScoreAgentV2::encode_params(
            packet.subject.clone(),
            signals.clone(),
            packet.evidence_v2.clone(),
        );
        let mut payload = &encoded[..];
        let route = String::decode(&mut payload).expect("decode score_agent_v2 route");
        let (decoded_subject, decoded_signals, decoded_evidence) =
            <(String, ReputationSignalsV2, Vec<String>)>::decode(&mut payload)
                .expect("decode preview score_agent_v2 params");

        assert_eq!(route, "ScoreAgentV2");
        assert_eq!(decoded_subject, "@fixture-agent");
        assert_eq!(decoded_signals, signals);
        assert_eq!(
            decoded_signals.application_status,
            ApplicationStatusV2::Submitted
        );
        assert_eq!(
            decoded_signals.participant_status,
            ParticipantStatusV2::ParticipantLike
        );
        assert_eq!(decoded_evidence, packet.evidence_v2);
    }

    #[test]
    fn score_agent_v2_gclient_io_payload_is_compatible() {
        let subject = "agent://v2-gclient-compat".to_string();
        let signals = sample_v2_signals();
        let evidence = vec![
            "observed:graphql:participant-calls".to_string(),
            "inferred:demo-readiness".to_string(),
            "unknown:frontend-url".to_string(),
        ];

        let encoded =
            ScoreAgentV2::encode_params(subject.clone(), signals.clone(), evidence.clone());
        let mut payload = &encoded[..];
        let route = String::decode(&mut payload).expect("decode score_agent_v2 route");
        let (decoded_subject, decoded_signals, decoded_evidence) =
            <(String, ReputationSignalsV2, Vec<String>)>::decode(&mut payload)
                .expect("decode score_agent_v2 params");

        assert_eq!(route, "ScoreAgentV2");
        assert_eq!(decoded_subject, subject);
        assert_eq!(decoded_signals, signals);
        assert_eq!(decoded_evidence, evidence);

        let encoded_with_prefix = ScoreAgentV2::encode_params_with_prefix(
            "ReputationOracle",
            subject.clone(),
            signals,
            evidence,
        );
        let mut prefixed_payload = &encoded_with_prefix[..];
        let service_route = String::decode(&mut prefixed_payload).expect("decode service route");
        let method_route = String::decode(&mut prefixed_payload).expect("decode method route");
        assert_eq!(service_route, "ReputationOracle");
        assert_eq!(method_route, "ScoreAgentV2");
    }

    #[test]
    fn v2_report_scale_round_trip_preserves_report_shape() {
        let report = ReputationReportV2 {
            subject: "agent://v2-report-compat".to_string(),
            as_of: "local-gclient-compat".to_string(),
            report_version: 2,
            signals: sample_v2_signals(),
            scores: ReputationScoresV2 {
                ecosystem_value_score: 76,
                real_integration_score: 82,
                counterparty_diversity_score: 70,
                identity_provenance_score: 88,
                demo_readiness_score: 74,
                safety_score: 91,
                spam_risk: 9,
                confidence_score: 83,
                overall_score: 80,
                verdict: "useful-integration".to_string(),
            },
            evidence: vec![
                "observed:graphql:registry".to_string(),
                "observed:board:description".to_string(),
            ],
        };

        let encoded = report.encode();
        let decoded = ReputationReportV2::decode(&mut &encoded[..]).expect("decode report v2");

        assert_eq!(decoded, report);
        assert_eq!(decoded.report_version, 2);
        assert_eq!(decoded.scores.overall_score, 80);
        assert_eq!(decoded.scores.verdict, "useful-integration");
        assert_eq!(decoded.evidence.len(), 2);
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let self_spawn_node = env::var("GEAR_NODE_PATH").ok();
    if self_spawn_node.is_none() && !endpoint_is_reachable() {
        println!(
            "SKIP gclient local-node smoke: no local Gear/Vara node reachable at {} and GEAR_NODE_PATH is unset",
            env::var("VARA_LOCAL_WS").unwrap_or_else(|_| "ws://127.0.0.1:9944".into())
        );
        return Ok(());
    }

    let wasm_path = optimized_wasm_path();
    if !wasm_path.exists() {
        return Err(format!(
            "optimized wasm not found at {}; run `npm run deploy:wasm` first",
            wasm_path.display()
        )
        .into());
    }

    let api = if let Some(path) = self_spawn_node {
        GearApi::dev_from_path(path).await?
    } else {
        GearApi::init(local_ws_address()).await?
    };
    println!("gclient smoke: uploading code from {}", wasm_path.display());
    let (code_id, _) = api.upload_code_by_path(&wasm_path).await?;
    println!("gclient smoke: code uploaded: {:?}", code_id);
    // Use a second funded dev signer for program creation and calls. This avoids
    // nonce reuse edge cases between upload_code and create_program on local dev nodes.
    let api = api.with("//Bob")?;
    let env = GclientEnv::new(api.clone());

    println!("gclient smoke: deploying program");
    let oracle = env
        .deploy::<ReputationOracleClientProgram>(
            code_id,
            gclient::now_micros().to_le_bytes().to_vec(),
        )
        .create()
        .await?;
    println!("gclient smoke: deployed program {:?}", oracle.id());

    let preview_packet = preview_packet_from_env()?;
    let preview_packet_v2 = preview_packet_v2_from_env()?;
    if let (Some(packet), Some(packet_v2)) = (&preview_packet, &preview_packet_v2) {
        assert_eq!(
            packet.subject, packet_v2.subject,
            "GraphQL V1 and V2 preview packets should target the same subject"
        );
    }
    let subject = preview_packet
        .as_ref()
        .map(|packet| packet.subject.clone())
        .unwrap_or_else(|| "agent://gclient-smoke".to_string());
    println!("gclient smoke: querying initial score");
    let initial_report = oracle
        .reputation_oracle()
        .score_agent(subject.clone())
        .await?;
    assert_eq!(initial_report.subject, subject);

    let mut service = oracle.reputation_oracle();
    println!("gclient smoke: record_attestation");
    let attestation = service
        .record_attestation(
            subject.clone(),
            "issuer://gclient-smoke".into(),
            "integration-test".into(),
            42,
            "hash:gclient-smoke".into(),
        )
        .await?;
    assert_eq!(attestation.subject, subject);

    let (signals, evidence) = if let Some(packet) = preview_packet {
        println!("gclient smoke: upsert_read_model from GraphQL preview packet");
        (packet.signals.into(), packet.evidence)
    } else {
        println!("gclient smoke: upsert_read_model from built-in sample packet");
        (
            sample_signals(),
            vec!["gclient-smoke".into(), "local-node".into()],
        )
    };
    let read_model = service
        .upsert_read_model(subject.clone(), signals, evidence)
        .await?;
    assert_eq!(read_model.subject, subject);
    assert!(read_model.report.scores.trust_score > 0);

    if let Some(packet_v2) = preview_packet_v2 {
        println!(
            "gclient smoke: validate ScoreAgentV2 payload from GraphQL preview packet (no submission)"
        );
        let signals_v2 = ReputationSignalsV2::try_from(packet_v2.signals_v2)
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
        let encoded = ScoreAgentV2::encode_params(
            packet_v2.subject.clone(),
            signals_v2.clone(),
            packet_v2.evidence_v2.clone(),
        );
        let mut payload = &encoded[..];
        let route = String::decode(&mut payload)?;
        let (decoded_subject, decoded_signals, decoded_evidence) =
            <(String, ReputationSignalsV2, Vec<String>)>::decode(&mut payload)?;
        assert_eq!(route, "ScoreAgentV2");
        assert_eq!(decoded_subject, subject);
        assert_eq!(decoded_signals, signals_v2);
        assert_eq!(decoded_evidence, packet_v2.evidence_v2);
    }

    let upgrade_smoke = env::var("GCLIENT_SMOKE_UPGRADE").ok().as_deref() == Some("1");

    if env::var("GCLIENT_SMOKE_ECONOMIC").ok().as_deref() == Some("1") {
        println!("gclient smoke: usage prediction market");
        let participant_stake = 1_000_000_000_000_000u128;
        let now_ms = (gclient::now_micros() / 1_000) as u64;
        let three_hours_ms = 3 * 60 * 60 * 1_000u64;
        let first_window_end = now_ms.saturating_add(12_000);
        let first_window_start = first_window_end.saturating_sub(three_hours_ms);
        let second_window_start = first_window_end;
        let second_window_end = second_window_start.saturating_add(three_hours_ms);

        let first_prediction = service
            .open_usage_prediction(
                1,
                subject.clone(),
                first_window_start,
                first_window_end,
                100,
                "hash:gclient-usage-prediction-epoch-1".into(),
            )
            .with_value(participant_stake)
            .await?;
        println!("gclient smoke: waiting for first 3h window close on local node");
        tokio::time::sleep(Duration::from_secs(14)).await;

        let first_settlement = service
            .settle_usage_prediction(
                first_prediction.position_id.clone(),
                200,
                "hash:gclient-usage-snapshot-epoch-1".into(),
            )
            .await?;
        assert_eq!(first_settlement.status, PredictionStatus::Lost);
        assert_eq!(first_settlement.reward_pool_balance, participant_stake);

        let next_prediction = service
            .open_usage_prediction(
                2,
                subject.clone(),
                second_window_start,
                second_window_end,
                90,
                "hash:gclient-usage-prediction-epoch-2".into(),
            )
            .with_value(participant_stake)
            .await?;
        assert_ne!(first_prediction.position_id, next_prediction.position_id);
        assert_eq!(next_prediction.epoch_id, 2);
        assert_eq!(next_prediction.window_start_ms, second_window_start);
        assert_eq!(next_prediction.window_end_ms, second_window_end);

        let predictions = oracle
            .reputation_oracle()
            .export_usage_predictions_chunk(0, 10)
            .await?;
        assert_eq!(predictions.total, 2);
        assert!(matches!(predictions.items[0].status, PredictionStatus::Lost));
        assert!(matches!(predictions.items[1].status, PredictionStatus::Open));
        let early_next_settlement = service
            .settle_usage_prediction(
                next_prediction.position_id.clone(),
                92,
                "hash:gclient-usage-snapshot-epoch-2".into(),
            )
            .await;
        assert!(
            early_next_settlement.is_err(),
            "second 3h window must not settle before window_end_ms"
        );
        println!(
            "gclient smoke: rolled settled epoch {} into open epoch {} for subject {}",
            first_settlement.position_id,
            next_prediction.position_id,
            subject
        );
    } else {
        println!("gclient smoke: skipping optional economic prediction path; pass --economic to enable");
    }

    if upgrade_smoke {
        println!("gclient smoke: migration/export readiness before read-only cutover");
        let config = oracle.reputation_oracle().export_migration_config().await?;
        assert!(!config.read_only);
        assert_eq!(config.operator_count, 0);
        assert_eq!(config.read_model_count, 1);
        assert_eq!(config.attestation_count, 1);
        assert!(config.bounds.max_export_chunk >= 64);
        let stats = oracle.reputation_oracle().export_stats().await?;
        assert_eq!(stats.read_model_count, config.read_model_count);
        assert_eq!(stats.attestation_count, config.attestation_count);
        let operators = oracle.reputation_oracle().export_operators().await?;
        assert!(operators.is_empty());
        let read_models = oracle
            .reputation_oracle()
            .export_read_models_chunk(0, 10)
            .await?;
        assert_eq!(read_models.total, 1);
        assert_eq!(read_models.items[0].subject, subject);
        let attestations = oracle
            .reputation_oracle()
            .export_attestations_chunk(0, 10)
            .await?;
        assert_eq!(attestations.total, 1);
        assert_eq!(attestations.items[0].subject, subject);
        println!(
            "gclient smoke: migration export config read_models={} attestations={} max_chunk={}",
            config.read_model_count, config.attestation_count, config.bounds.max_export_chunk
        );
    }

    println!("gclient smoke: set_read_only");
    let read_only = service.set_read_only(true).await?;
    assert!(read_only.read_only);

    if upgrade_smoke {
        let status = oracle.reputation_oracle().oracle_status().await?;
        assert_eq!(status, reputation_oracle_client::OracleStatus::ReadOnly);
        let cutover_config = oracle.reputation_oracle().export_migration_config().await?;
        assert!(cutover_config.read_only);
        assert_eq!(cutover_config.status, reputation_oracle_client::OracleStatus::ReadOnly);
        println!("gclient smoke: read-only cutover status verified");
    }

    println!("gclient smoke: skipping expected read-only rejection follow-up calls; covered by gtest");

    let economic_note = if env::var("GCLIENT_SMOKE_ECONOMIC").ok().as_deref() == Some("1") {
        " + 3h prediction-window rollover"
    } else {
        ""
    };
    let upgrade_note = if upgrade_smoke {
        " + migration exports/read-only cutover"
    } else {
        ""
    };
    println!(
        "PASS gclient local-node smoke: deployed {:?} and verified read-model + V2 payload path{}{}",
        oracle.id(),
        economic_note,
        upgrade_note
    );
    Ok(())
}
