export const VARA_GRAPHQL_ENDPOINT = 'https://agents-api.vara.network/graphql';

export const OPERATOR_PREVIEW_QUERY = `query OperatorPreview($first: Int!) {
  allApplications(first: $first, orderBy: [REGISTERED_AT_DESC]) {
    nodes {
      id
      handle
      owner
      description
      githubUrl
      skillsUrl
      idlUrl
      xAccount
      status
      tags
      identityCardUpdatedAt
      registeredAt
      seasonId
      discordAccount
      telegramAccount
    }
  }
  allAppMetrics(first: $first, orderBy: [UPDATED_AT_DESC]) {
    nodes {
      applicationId
      seasonId
      uniqueSendersToMe
      mentionCount
      messagesSent
      postsActive
      integrationsOut
      integrationsIn
      uniquePartners
      callGraphDensity
      updatedAt
    }
  }
  allIdentityCards(first: $first, orderBy: [UPDATED_AT_DESC]) {
    nodes {
      id
      tags
      updatedAt
      updatedBy
      seasonId
    }
  }
  allHandleClaims(first: $first, orderBy: [CLAIMED_AT_DESC]) {
    nodes {
      handle
      ownerKind
      ownerId
      claimedAt
      seasonId
    }
  }
}`;

export function toNonNegativeInt(value) {
  const number = Number(value ?? 0);
  if (!Number.isFinite(number) || number <= 0) return 0;
  return Math.floor(number);
}

export function clampU32(value) {
  return Math.min(toNonNegativeInt(value), 0xffffffff);
}

export function truthyString(value) {
  return typeof value === 'string' && value.trim().length > 0;
}

function asArray(value) {
  return Array.isArray(value) ? value : [];
}

function byKey(rows, key) {
  const map = new Map();
  for (const row of asArray(rows)) {
    const id = row?.[key];
    if (truthyString(id) && !map.has(id.toLowerCase())) map.set(id.toLowerCase(), row);
  }
  return map;
}

function evidenceLabel(label) {
  return String(label).trim().slice(0, 96);
}

export function subjectForApplication(application) {
  if (truthyString(application?.handle)) return `@${application.handle.trim().replace(/^@+/, '')}`;
  if (truthyString(application?.id)) return `agent:${application.id.trim()}`;
  return null;
}

export function signalsForApplication(application, metric, identity, handleClaim) {
  const hasIdentityCard = Boolean(identity) || truthyString(application?.identityCardUpdatedAt);
  const socialFields = [
    application?.githubUrl,
    application?.skillsUrl,
    application?.idlUrl,
    application?.xAccount,
    application?.discordAccount,
    application?.telegramAccount,
  ];
  const hasVerifiedSocialProof = socialFields.some(truthyString);
  const missingRequiredMetadata = [
    application?.description,
    application?.githubUrl,
    application?.skillsUrl,
    application?.idlUrl,
    hasIdentityCard ? 'identity-card' : null,
  ].filter((value) => !truthyString(value)).length;

  const incomingUniqueCallers = Math.max(
    toNonNegativeInt(metric?.uniqueSendersToMe),
    toNonNegativeInt(metric?.integrationsIn) > 0 ? 1 : 0,
  );
  const outgoingMeaningfulCalls = Math.max(
    toNonNegativeInt(metric?.uniquePartners),
    toNonNegativeInt(metric?.integrationsOut),
  );
  const chatBoardUpdates =
    toNonNegativeInt(metric?.mentionCount) +
    toNonNegativeInt(metric?.messagesSent) +
    toNonNegativeInt(metric?.postsActive);
  const callGraphDensity = Number(metric?.callGraphDensity ?? 0);

  return {
    incoming_unique_callers: clampU32(incomingUniqueCallers),
    outgoing_meaningful_calls: clampU32(outgoingMeaningfulCalls),
    chat_board_updates: clampU32(chatBoardUpdates),
    has_identity_card: hasIdentityCard,
    has_verified_social_proof: hasVerifiedSocialProof,
    circular_call_signals: callGraphDensity >= 0.75 ? 1 : 0,
    missing_required_metadata: clampU32(missingRequiredMetadata),
    positive_attestations: clampU32(
      (truthyString(application?.status) ? 1 : 0) +
        (handleClaim ? 1 : 0) +
        (hasVerifiedSocialProof ? 1 : 0),
    ),
    negative_attestations: 0,
  };
}

export function evidenceForApplication(application, metric, identity, handleClaim) {
  const evidence = [
    'source:vara-agent-network/graphql',
    application?.id ? `application:${application.id}` : null,
    application?.handle ? `handle:${application.handle}` : null,
    application?.status ? `status:${application.status}` : null,
    metric ? `metrics:in=${toNonNegativeInt(metric.integrationsIn)}:out=${toNonNegativeInt(metric.integrationsOut)}` : null,
    metric ? `chat:${toNonNegativeInt(metric.mentionCount) + toNonNegativeInt(metric.messagesSent)}` : null,
    identity ? 'identity-card:present' : null,
    handleClaim ? `handle-claim:${handleClaim.ownerKind ?? 'unknown'}` : null,
    application?.githubUrl ? 'metadata:github' : null,
    application?.skillsUrl ? 'metadata:skills' : null,
    application?.idlUrl ? 'metadata:idl' : null,
    application?.xAccount ? 'metadata:x' : null,
  ]
    .filter(Boolean)
    .map(evidenceLabel);

  return [...new Set(evidence)].slice(0, 16);
}


export function parseUnixMillis(value) {
  const number = Number(value ?? 0);
  if (!Number.isFinite(number) || number <= 0) return 0;
  return Math.floor(number);
}

export function applicationStatusV2(value) {
  const normalized = truthyString(value) ? value.trim().toLowerCase() : '';
  if (normalized === 'draft') return 'Draft';
  if (normalized === 'submitted') return 'Submitted';
  if (normalized === 'approved') return 'Approved';
  if (normalized === 'rejected') return 'Rejected';
  if (normalized === 'suspended') return 'Suspended';
  return 'Unknown';
}

export function hasClearBoardDescription(value) {
  if (!truthyString(value)) return false;
  const text = value.trim();
  const words = text.split(/\s+/).filter(Boolean);
  return text.length >= 24 && words.length >= 4;
}

export function hasActualGithubRepoCandidate(value) {
  if (!truthyString(value)) return false;
  try {
    const url = new URL(value.trim());
    const host = url.hostname.toLowerCase().replace(/^www\./, '');
    if (host !== 'github.com') return false;
    const parts = url.pathname.split('/').filter(Boolean);
    return parts.length >= 2 && parts[0].length > 0 && parts[1].length > 0;
  } catch {
    return false;
  }
}

function densityBps(value) {
  const number = Number(value ?? 0);
  if (!Number.isFinite(number) || number <= 0) return 0;
  return Math.min(Math.round(number * 10000), 10000);
}

function lowDiversityVolume(raw, unique) {
  return clampU32(toNonNegativeInt(raw) - toNonNegativeInt(unique) * 10);
}

export function signalsV2ForApplication(application, metric, identity, handleClaim) {
  const inboundUnique = toNonNegativeInt(metric?.uniqueSendersToMe);
  const inboundCount = toNonNegativeInt(metric?.integrationsIn);
  const outboundUnique = toNonNegativeInt(metric?.uniquePartners);
  const outboundCount = toNonNegativeInt(metric?.integrationsOut);
  const density = densityBps(metric?.callGraphDensity);
  const lowDiversityInbound = lowDiversityVolume(inboundCount, inboundUnique);
  const lowDiversityOutbound = lowDiversityVolume(outboundCount, outboundUnique);
  const participantLike = toNonNegativeInt(application?.seasonId) > 0 || toNonNegativeInt(metric?.seasonId) > 0;
  const identityUpdatedAt = parseUnixMillis(identity?.updatedAt ?? application?.identityCardUpdatedAt);

  return {
    inbound_unique_participants: clampU32(inboundUnique),
    inbound_call_count: clampU32(inboundCount),
    outbound_unique_participants: clampU32(outboundUnique),
    outbound_call_count: clampU32(outboundCount),
    valid_participant_inbound_count: 0,
    valid_participant_outbound_count: 0,
    non_participant_call_count: 0,
    self_loop_call_count: 0,
    same_owner_call_count: 0,
    mention_count: clampU32(metric?.mentionCount),
    messages_sent: clampU32(metric?.messagesSent),
    posts_active: clampU32(metric?.postsActive),
    has_clear_board_description: hasClearBoardDescription(application?.description),
    has_identity_card: Boolean(identity) || truthyString(application?.identityCardUpdatedAt),
    has_actual_github_repo: hasActualGithubRepoCandidate(application?.githubUrl),
    has_skills_url: truthyString(application?.skillsUrl),
    has_idl_url: truthyString(application?.idlUrl),
    has_frontend_url: false,
    handle_claimed: Boolean(handleClaim),
    application_status: applicationStatusV2(application?.status),
    participant_status: participantLike ? 'ParticipantLike' : 'Unknown',
    call_graph_density_bps: clampU32(density),
    low_diversity_volume_count: clampU32(lowDiversityInbound + lowDiversityOutbound),
    reciprocal_farming_signals: clampU32((density >= 7500 ? 1 : 0) + (lowDiversityInbound + lowDiversityOutbound > 0 ? 1 : 0)),
    positive_third_party_attestations: 0,
    negative_third_party_attestations: 0,
    metrics_updated_at: parseUnixMillis(metric?.updatedAt),
    identity_updated_at: identityUpdatedAt,
    registered_at: parseUnixMillis(application?.registeredAt),
  };
}

export function evidenceV2ForApplication(application, metric, identity, handleClaim) {
  const evidence = [
    'source:vara-agent-network/graphql',
    application?.id ? `observed:application-id:${application.id}` : null,
    application?.handle ? `observed:handle:${application.handle}` : null,
    application?.owner ? 'observed:owner' : 'unknown:owner',
    application?.status ? `observed:application-status:${applicationStatusV2(application.status)}` : 'unknown:application-status',
    hasClearBoardDescription(application?.description) ? 'observed:description:clear' : 'observed:description:missing-or-thin',
    identity || truthyString(application?.identityCardUpdatedAt) ? 'observed:identity-card' : 'observed:identity-card:missing',
    handleClaim ? `observed:handle-claim:${handleClaim.ownerKind ?? 'unknown'}` : 'observed:handle-claim:missing',
    truthyString(application?.githubUrl) ? 'observed:github-url' : 'observed:github-url:missing',
    hasActualGithubRepoCandidate(application?.githubUrl) ? 'unverified:github-repo-quality' : null,
    truthyString(application?.skillsUrl) ? 'observed:skills-url' : 'observed:skills-url:missing',
    truthyString(application?.idlUrl) ? 'observed:idl-url' : 'observed:idl-url:missing',
    'unknown:frontend-url',
    metric ? `observed:integrations-in:${toNonNegativeInt(metric.integrationsIn)}` : 'observed:integrations-in:0',
    metric ? `observed:unique-inbound:${toNonNegativeInt(metric.uniqueSendersToMe)}` : 'observed:unique-inbound:0',
    metric ? `observed:integrations-out:${toNonNegativeInt(metric.integrationsOut)}` : 'observed:integrations-out:0',
    metric ? `observed:unique-partners:${toNonNegativeInt(metric.uniquePartners)}` : 'observed:unique-partners:0',
    'unknown:counterparty-identities',
    'unknown:self-loop-count',
    'unknown:same-owner-count',
    'unknown:non-participant-count',
    densityBps(metric?.callGraphDensity) >= 7500 ? 'risk:high-call-graph-density' : null,
  ]
    .filter(Boolean)
    .map(evidenceLabel);

  return [...new Set(evidence)].slice(0, 32);
}


function clampScore(value) {
  const number = Number(value ?? 0);
  if (!Number.isFinite(number)) return 0;
  return Math.max(0, Math.min(100, Math.round(number)));
}

function statusAllowsSubmitted(status) {
  return status === 'Submitted' || status === 'Approved';
}

export function scoreSignalsV2(signals = {}) {
  const ecosystemValueScore = clampScore(
    (signals.has_clear_board_description ? 35 : 0) +
      (signals.has_idl_url ? 20 : 0) +
      (signals.has_skills_url ? 15 : 0) +
      (signals.has_identity_card ? 15 : 0) +
      Math.min(15, toNonNegativeInt(signals.posts_active) * 5 + Math.min(toNonNegativeInt(signals.mention_count) + toNonNegativeInt(signals.messages_sent), 5)),
  );

  const realIntegrationScore = clampScore(
    Math.min(toNonNegativeInt(signals.inbound_unique_participants), 10) * 4 +
      Math.min(toNonNegativeInt(signals.outbound_unique_participants), 10) * 3 +
      Math.min(toNonNegativeInt(signals.inbound_call_count), 50) / 5 +
      Math.min(toNonNegativeInt(signals.outbound_call_count), 50) / 5 +
      (toNonNegativeInt(signals.inbound_unique_participants) >= 2 && toNonNegativeInt(signals.outbound_unique_participants) >= 2 ? 10 : 0),
  );

  const uniqueTotal = Math.max(toNonNegativeInt(signals.inbound_unique_participants), toNonNegativeInt(signals.outbound_unique_participants));
  const counterpartyDiversityScore = clampScore(
    Math.min(uniqueTotal, 10) * 7 +
      (toNonNegativeInt(signals.inbound_unique_participants) >= 2 && toNonNegativeInt(signals.outbound_unique_participants) >= 2 ? 20 : uniqueTotal >= 2 ? 10 : 0) +
      (toNonNegativeInt(signals.call_graph_density_bps) < 5000 ? 10 : 0),
  );

  const identityProvenanceScore = clampScore(
    (signals.handle_claimed ? 25 : 0) +
      (signals.has_identity_card ? 25 : 0) +
      (statusAllowsSubmitted(signals.application_status) ? 20 : 0) +
      (signals.participant_status !== 'Unknown' ? 15 : 0) +
      (toNonNegativeInt(signals.identity_updated_at) > 0 ? 15 : 0),
  );

  const demoReadinessScore = clampScore(
    (signals.has_clear_board_description ? 30 : 0) +
      (signals.has_actual_github_repo ? 25 : 0) +
      (signals.has_frontend_url ? 25 : 0) +
      (signals.has_idl_url ? 10 : 0) +
      (signals.has_skills_url ? 10 : 0),
  );

  const validParticipantCalls = toNonNegativeInt(signals.valid_participant_inbound_count) + toNonNegativeInt(signals.valid_participant_outbound_count);
  let riskPoints = 0;
  if (toNonNegativeInt(signals.self_loop_call_count) > 0) riskPoints += 35;
  if (toNonNegativeInt(signals.same_owner_call_count) > 0) riskPoints += 25;
  if (validParticipantCalls > 0 && toNonNegativeInt(signals.non_participant_call_count) > validParticipantCalls) riskPoints += 25;
  if (toNonNegativeInt(signals.call_graph_density_bps) >= 7500) riskPoints += 20;
  if (toNonNegativeInt(signals.low_diversity_volume_count) >= 25) riskPoints += 15;
  if (toNonNegativeInt(signals.outbound_call_count) >= 30 && toNonNegativeInt(signals.outbound_unique_participants) < 3) riskPoints += 15;
  if (toNonNegativeInt(signals.inbound_call_count) >= 30 && toNonNegativeInt(signals.inbound_unique_participants) < 3) riskPoints += 10;
  if (!signals.has_clear_board_description && !signals.has_idl_url && !signals.has_skills_url && toNonNegativeInt(signals.inbound_call_count) + toNonNegativeInt(signals.outbound_call_count) >= 30) riskPoints += 10;
  riskPoints += Math.min(toNonNegativeInt(signals.negative_third_party_attestations) * 25, 50);
  riskPoints -= Math.min(toNonNegativeInt(signals.positive_third_party_attestations) * 5, 15);

  const spamRisk = clampScore(riskPoints);
  const safetyScore = 100 - spamRisk;
  const metadataPresent = [signals.has_clear_board_description, signals.has_actual_github_repo, signals.has_skills_url, signals.has_idl_url].filter(Boolean).length;
  const confidenceScore = clampScore(
    25 +
      (toNonNegativeInt(signals.metrics_updated_at) > 0 ? 20 : 0) +
      (signals.has_identity_card ? 15 : 0) +
      (signals.handle_claimed ? 10 : 0) +
      Math.round((metadataPresent / 4) * 15) +
      (signals.participant_status !== 'Unknown' ? 10 : 0),
  );

  const overallScore = clampScore(
    ecosystemValueScore * 0.2 +
      realIntegrationScore * 0.25 +
      counterpartyDiversityScore * 0.15 +
      identityProvenanceScore * 0.1 +
      demoReadinessScore * 0.15 +
      safetyScore * 0.15,
  );

  let verdict = 'avoid_or_wait';
  if (signals.application_status === 'Rejected' || signals.application_status === 'Suspended' || spamRisk >= 70) {
    verdict = 'avoid_or_wait';
  } else if (overallScore >= 75 && spamRisk < 30 && confidenceScore >= 65 && realIntegrationScore >= 55) {
    verdict = 'recommended';
  } else if (overallScore >= 45 && spamRisk < 70) {
    verdict = 'review';
  }

  return {
    ecosystem_value_score: ecosystemValueScore,
    real_integration_score: realIntegrationScore,
    counterparty_diversity_score: counterpartyDiversityScore,
    identity_provenance_score: identityProvenanceScore,
    demo_readiness_score: demoReadinessScore,
    safety_score: safetyScore,
    spam_risk: spamRisk,
    confidence_score: confidenceScore,
    overall_score: overallScore,
    verdict,
  };
}

export function normalizeOperatorPreviewV2(data) {
  const applications = asArray(data?.allApplications?.nodes);
  const metrics = byKey(data?.allAppMetrics?.nodes, 'applicationId');
  const identities = byKey(data?.allIdentityCards?.nodes, 'id');
  const claims = byKey(data?.allHandleClaims?.nodes, 'ownerId');

  return applications
    .map((application) => {
      const subject = subjectForApplication(application);
      if (!subject || !truthyString(application?.id)) return null;
      const key = application.id.toLowerCase();
      const metric = metrics.get(key) ?? null;
      const identity = identities.get(key) ?? null;
      const handleClaim = claims.get(key) ?? null;
      const signalsV2 = signalsV2ForApplication(application, metric, identity, handleClaim);
      return {
        subject,
        signalsV2,
        scoresV2: scoreSignalsV2(signalsV2),
        evidenceV2: evidenceV2ForApplication(application, metric, identity, handleClaim),
      };
    })
    .filter(Boolean)
    .sort((left, right) => left.subject.localeCompare(right.subject));
}

export function normalizeOperatorPreview(data) {
  const applications = asArray(data?.allApplications?.nodes);
  const metrics = byKey(data?.allAppMetrics?.nodes, 'applicationId');
  const identities = byKey(data?.allIdentityCards?.nodes, 'id');
  const claims = byKey(data?.allHandleClaims?.nodes, 'ownerId');

  return applications
    .map((application) => {
      const subject = subjectForApplication(application);
      if (!subject || !truthyString(application?.id)) return null;
      const key = application.id.toLowerCase();
      const metric = metrics.get(key) ?? null;
      const identity = identities.get(key) ?? null;
      const handleClaim = claims.get(key) ?? null;
      return {
        subject,
        signals: signalsForApplication(application, metric, identity, handleClaim),
        evidence: evidenceForApplication(application, metric, identity, handleClaim),
      };
    })
    .filter(Boolean)
    .sort((left, right) => left.subject.localeCompare(right.subject));
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

export async function graphqlRequest({
  endpoint = VARA_GRAPHQL_ENDPOINT,
  query = OPERATOR_PREVIEW_QUERY,
  variables = {},
  fetchImpl = globalThis.fetch,
  retries = 2,
} = {}) {
  if (typeof fetchImpl !== 'function') {
    throw new Error('fetch is unavailable; use Node.js 18+ or pass fetchImpl');
  }
  let lastError;
  for (let attempt = 0; attempt <= retries; attempt += 1) {
    try {
      const response = await fetchImpl(endpoint, {
        method: 'POST',
        headers: {
          'content-type': 'application/json',
          accept: 'application/json',
          'user-agent': 'a2a-reputation-oracle/graphql-preview',
        },
        body: JSON.stringify({ query, variables }),
      });
      const text = await response.text();
      if (!response.ok) throw new Error(`GraphQL HTTP ${response.status}: ${text.slice(0, 500)}`);
      const payload = JSON.parse(text);
      if (payload.errors?.length) {
        throw new Error(`GraphQL errors: ${payload.errors.map((error) => error.message).join('; ')}`);
      }
      return payload.data;
    } catch (error) {
      lastError = error;
      if (attempt < retries) await sleep(250 * 2 ** attempt);
    }
  }
  throw lastError;
}

export async function fetchOperatorPreview({ endpoint, first = 25, fetchImpl } = {}) {
  const data = await graphqlRequest({
    endpoint,
    variables: { first },
    fetchImpl,
  });
  return normalizeOperatorPreview(data);
}

export function selectPreviewPacket(packets, { subject } = {}) {
  if (!packets?.length) throw new Error('no preview packets available');
  if (!subject) return packets[0];
  const normalized = subject.trim().toLowerCase();
  const packet = packets.find((candidate) => candidate.subject.toLowerCase() === normalized);
  if (!packet) throw new Error(`subject not found in preview packets: ${subject}`);
  return packet;
}
