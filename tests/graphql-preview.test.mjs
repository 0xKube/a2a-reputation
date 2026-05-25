import test from 'node:test';
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { readFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';
import {
  fetchOperatorPreview,
  hasActualGithubRepoCandidate,
  normalizeOperatorPreview,
  normalizeOperatorPreviewV2,
  scoreSignalsV2,
  selectPreviewPacket,
  signalsForApplication,
  signalsV2ForApplication,
} from '../src/graphql-preview.mjs';

const __dirname = dirname(fileURLToPath(import.meta.url));

const fixture = {
  allApplications: {
    nodes: [
      {
        id: '0xabc',
        handle: 'alpha-agent',
        description: 'Useful agent',
        githubUrl: 'https://github.com/example/alpha',
        skillsUrl: 'https://example.com/skills.md',
        idlUrl: 'https://example.com/client.idl',
        xAccount: '@alpha',
        status: 'Submitted',
        identityCardUpdatedAt: '1779210381001',
      },
      {
        id: '0xdef',
        handle: 'beta-agent',
        description: '',
        githubUrl: null,
        skillsUrl: null,
        idlUrl: null,
        xAccount: null,
        status: 'Submitted',
        identityCardUpdatedAt: null,
      },
    ],
  },
  allAppMetrics: {
    nodes: [
      {
        applicationId: '0xabc',
        uniqueSendersToMe: 4,
        mentionCount: 3,
        messagesSent: 2,
        postsActive: 1,
        integrationsOut: 7,
        integrationsIn: 11,
        uniquePartners: 5,
        callGraphDensity: 0.1,
      },
    ],
  },
  allIdentityCards: { nodes: [{ id: '0xabc' }] },
  allHandleClaims: { nodes: [{ ownerId: '0xabc', ownerKind: 'Application' }] },
};

test('normalizes GraphQL data into deterministic preview packets', () => {
  const packets = normalizeOperatorPreview(fixture);
  assert.deepEqual(packets.map((packet) => packet.subject), ['@alpha-agent', '@beta-agent']);
  assert.deepEqual(packets[0].signals, {
    incoming_unique_callers: 4,
    outgoing_meaningful_calls: 7,
    chat_board_updates: 6,
    has_identity_card: true,
    has_verified_social_proof: true,
    circular_call_signals: 0,
    missing_required_metadata: 0,
    positive_attestations: 3,
    negative_attestations: 0,
  });
  assert.ok(packets[0].evidence.includes('source:vara-agent-network/graphql'));
  assert.ok(packets[0].evidence.includes('identity-card:present'));
  assert.equal(packets[1].signals.missing_required_metadata, 5);
});

test('flags dense call graph as circular signal without inventing negative attestations', () => {
  const signals = signalsForApplication(
    { description: 'x', githubUrl: 'g', skillsUrl: 's', idlUrl: 'i', identityCardUpdatedAt: '1', status: 'Submitted' },
    { callGraphDensity: 0.9 },
    null,
    null,
  );
  assert.equal(signals.circular_call_signals, 1);
  assert.equal(signals.negative_attestations, 0);
});

test('fetchOperatorPreview uses read-only GraphQL POST and injected fetch', async () => {
  const calls = [];
  const packets = await fetchOperatorPreview({
    first: 2,
    fetchImpl: async (url, init) => {
      calls.push({ url, init });
      return {
        ok: true,
        async text() {
          return JSON.stringify({ data: fixture });
        },
      };
    },
  });

  assert.equal(calls[0].init.method, 'POST');
  assert.match(calls[0].init.body, /OperatorPreview/);
  assert.match(calls[0].init.body, /"first":2/);
  assert.equal(packets.length, 2);
});

test('selectPreviewPacket supports explicit subject selection', () => {
  const packets = normalizeOperatorPreview(fixture);
  assert.equal(selectPreviewPacket(packets, { subject: '@beta-agent' }).subject, '@beta-agent');
  assert.throws(() => selectPreviewPacket(packets, { subject: '@missing' }), /subject not found/);
});


test('normalizes fixture data into V2 signal packets with evidence labels', async () => {
  const recorded = JSON.parse(await readFile(join(__dirname, '../fixtures/graphql-preview-minimal.json'), 'utf8'));
  const packets = normalizeOperatorPreviewV2(recorded.data);

  assert.equal(packets.length, 1);
  assert.equal(packets[0].subject, '@fixture-agent');
  assert.deepEqual(packets[0].signalsV2, {
    inbound_unique_participants: 3,
    inbound_call_count: 9,
    outbound_unique_participants: 2,
    outbound_call_count: 4,
    valid_participant_inbound_count: 0,
    valid_participant_outbound_count: 0,
    non_participant_call_count: 0,
    self_loop_call_count: 0,
    same_owner_call_count: 0,
    mention_count: 2,
    messages_sent: 1,
    posts_active: 1,
    has_clear_board_description: true,
    has_identity_card: true,
    has_actual_github_repo: true,
    has_skills_url: true,
    has_idl_url: true,
    has_frontend_url: false,
    handle_claimed: true,
    application_status: 'Submitted',
    participant_status: 'ParticipantLike',
    call_graph_density_bps: 500,
    low_diversity_volume_count: 0,
    reciprocal_farming_signals: 0,
    positive_third_party_attestations: 0,
    negative_third_party_attestations: 0,
    metrics_updated_at: 1779408000000,
    identity_updated_at: 1779210381001,
    registered_at: 1779210312000,
  });
  assert.ok(packets[0].evidenceV2.includes('observed:unique-inbound:3'));
  assert.ok(packets[0].evidenceV2.includes('observed:integrations-in:9'));
  assert.ok(packets[0].evidenceV2.includes('unverified:github-repo-quality'));
  assert.ok(packets[0].evidenceV2.includes('unknown:frontend-url'));
  assert.ok(packets[0].evidenceV2.includes('unknown:counterparty-identities'));
  assert.deepEqual(packets[0].scoresV2, {
    ecosystem_value_score: 93,
    real_integration_score: 31,
    counterparty_diversity_score: 51,
    identity_provenance_score: 100,
    demo_readiness_score: 75,
    safety_score: 100,
    spam_risk: 0,
    confidence_score: 95,
    overall_score: 70,
    verdict: 'review',
  });
});

test('V2 normalizer marks dense, low-diversity aggregate activity as risk without fabricating counters', () => {
  const signals = signalsV2ForApplication(
    {
      id: '0xrisky',
      handle: 'risky-agent',
      description: 'Thin',
      githubUrl: 'https://github.com/example',
      status: 'Submitted',
    },
    {
      uniqueSendersToMe: 1,
      integrationsIn: 42,
      uniquePartners: 1,
      integrationsOut: 35,
      callGraphDensity: 0.82,
    },
    null,
    null,
  );

  assert.equal(signals.call_graph_density_bps, 8200);
  assert.equal(signals.low_diversity_volume_count, 57);
  assert.equal(signals.reciprocal_farming_signals, 2);
  assert.equal(signals.self_loop_call_count, 0);
  assert.equal(signals.same_owner_call_count, 0);
  assert.equal(signals.non_participant_call_count, 0);
  assert.equal(signals.positive_third_party_attestations, 0);
  assert.equal(hasActualGithubRepoCandidate('https://github.com/example'), false);
});


test('scores V2 components from useful integration and inflated loop examples', () => {
  const useful = scoreSignalsV2({
    inbound_unique_participants: 10,
    inbound_call_count: 50,
    outbound_unique_participants: 6,
    outbound_call_count: 30,
    valid_participant_inbound_count: 0,
    valid_participant_outbound_count: 0,
    non_participant_call_count: 0,
    self_loop_call_count: 0,
    same_owner_call_count: 0,
    mention_count: 3,
    messages_sent: 2,
    posts_active: 2,
    has_clear_board_description: true,
    has_identity_card: true,
    has_actual_github_repo: true,
    has_skills_url: true,
    has_idl_url: true,
    has_frontend_url: true,
    handle_claimed: true,
    application_status: 'Submitted',
    participant_status: 'ParticipantLike',
    call_graph_density_bps: 1200,
    low_diversity_volume_count: 0,
    reciprocal_farming_signals: 0,
    positive_third_party_attestations: 0,
    negative_third_party_attestations: 0,
    metrics_updated_at: 1779408000000,
    identity_updated_at: 1779210381001,
    registered_at: 1779210312000,
  });

  assert.equal(useful.real_integration_score, 84);
  assert.equal(useful.counterparty_diversity_score, 100);
  assert.equal(useful.demo_readiness_score, 100);
  assert.equal(useful.verdict, 'recommended');

  const inflatedLoop = scoreSignalsV2({
    inbound_unique_participants: 1,
    inbound_call_count: 80,
    outbound_unique_participants: 1,
    outbound_call_count: 80,
    self_loop_call_count: 1,
    same_owner_call_count: 1,
    mention_count: 0,
    messages_sent: 0,
    posts_active: 0,
    has_clear_board_description: false,
    has_identity_card: false,
    has_actual_github_repo: false,
    has_skills_url: false,
    has_idl_url: false,
    has_frontend_url: false,
    handle_claimed: false,
    application_status: 'Submitted',
    participant_status: 'Unknown',
    call_graph_density_bps: 9000,
    low_diversity_volume_count: 140,
    reciprocal_farming_signals: 2,
    positive_third_party_attestations: 0,
    negative_third_party_attestations: 1,
    metrics_updated_at: 1779408000000,
    identity_updated_at: 0,
    registered_at: 1779210312000,
  });

  assert.equal(inflatedLoop.spam_risk, 100);
  assert.equal(inflatedLoop.safety_score, 0);
  assert.equal(inflatedLoop.verdict, 'avoid_or_wait');
});

test('preview CLI can emit a selected V2 packet with scores', () => {
  const output = execFileSync(process.execPath, [
    'scripts/preview-graphql.mjs',
    '--fixture',
    'fixtures/graphql-preview-minimal.json',
    '--v2',
    '--one',
    '--subject',
    '@fixture-agent',
  ], { cwd: join(__dirname, '..'), encoding: 'utf8' });
  const packet = JSON.parse(output);
  assert.equal(packet.subject, '@fixture-agent');
  assert.equal(packet.signalsV2.inbound_call_count, 9);
  assert.equal(packet.scoresV2.verdict, 'review');
  assert.equal(packet.evidenceV2.includes('unknown:frontend-url'), true);
});
