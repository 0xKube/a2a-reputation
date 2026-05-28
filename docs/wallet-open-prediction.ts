import { GearApi } from '@gear-js/api';
import { u8aToHex } from '@polkadot/util';
import { decodeAddress } from '@polkadot/util-crypto';
import { web3Accounts, web3Enable, web3FromAddress } from '@polkadot/extension-dapp';
import { Sails } from 'sails-js';
import { SailsIdlParser } from 'sails-js-parser';

declare global {
  interface Window {
    varaMarketsWallet?: VaraMarketsWallet;
  }
}

type Account = { address: string; meta?: { name?: string; source?: string } };

type TxResult = {
  txHash: string;
  blockHash: string;
  msgId?: string;
  response?: unknown;
};

type ExportedPosition = {
  position_id: string;
  predictor: string;
  epoch_id: number;
  subject: string;
  window_start_ms: string;
  window_end_ms: string;
  predicted_delta_calls: number;
  evidence_hash: string;
  stake: string;
  effective_stake: string;
  status: { kind?: string } | string;
  payout: string;
};

type VaraMarketsWallet = {
  connect(): Promise<{ address: string; name: string; source: string }>;
  openPrediction(input: OpenPredictionInput): Promise<TxResult>;
  exportPositions(): Promise<ExportedPosition[]>;
  disconnect(): Promise<void>;
  state(): { connected: boolean; address: string | null; addressHex: string | null; status: string };
};

type OpenPredictionInput = {
  subject: string;
  predictedDeltaCalls: number;
  stakeVara: number;
  evidence: string;
};

const PROGRAM_ID = '0x580b6bae88499c2595985acf7d8d320e3f0eaf5187f3dc47fd773c9c97b8f62a';
const IDL_URL = './a2a-reputation-v2.idl';
const VARA_RPC = 'wss://rpc.vara.network';
const VARA_UNIT = 1_000_000_000_000n;

let apiPromise: Promise<GearApi> | null = null;
let sailsPromise: Promise<Sails> | null = null;
let selectedAccount: Account | null = null;
let selectedAccountHex: string | null = null;
let statusText = 'wallet not connected';

function setStatus(text: string) {
  statusText = text;
  window.dispatchEvent(new CustomEvent('vara-markets-status', { detail: { status: text } }));
}

function sanitizeSubject(value: string) {
  const subject = value.trim();
  if (!subject.startsWith('@') || subject.length < 4) throw new Error('Subject must look like @agent-handle');
  return subject;
}

function sanitizePrediction(value: number) {
  if (!Number.isInteger(value) || value < 0) throw new Error('Prediction must be a non-negative integer');
  return value;
}

function stakeToPlancks(value: number) {
  if (!Number.isFinite(value) || value < 10 || value > 10_000) {
    throw new Error('Stake must be between 10 and 10,000 VARA');
  }
  const fixed = value.toFixed(6);
  const [whole, frac = ''] = fixed.split('.');
  return BigInt(whole) * VARA_UNIT + BigInt(frac.padEnd(6, '0')) * 1_000_000n;
}

function nextEpochWindow() {
  const now = Date.now();
  const threeHours = 3 * 60 * 60 * 1000;
  // Keep epoch_id compact and deterministic from wall-clock 3h buckets.
  const epochId = Math.floor(now / threeHours);
  const startMs = now;
  const endMs = now + threeHours;
  return { epochId, startMs, endMs };
}

async function getApi() {
  if (!apiPromise) {
    setStatus('connecting to Vara RPC…');
    apiPromise = GearApi.create({ providerAddress: VARA_RPC });
  }
  return apiPromise;
}

async function getSails() {
  if (!sailsPromise) {
    sailsPromise = (async () => {
      const [api, parser, idl] = await Promise.all([
        getApi(),
        SailsIdlParser.new(),
        fetch(IDL_URL).then((response) => {
          if (!response.ok) throw new Error(`Failed to load IDL: HTTP ${response.status}`);
          return response.text();
        }),
      ]);
      const sails = new Sails(parser);
      sails.parseIdl(idl);
      sails.setApi(api);
      sails.setProgramId(PROGRAM_ID);
      return sails;
    })();
  }
  return sailsPromise;
}

function statusKind(status: ExportedPosition['status']) {
  if (!status) return 'Unknown';
  if (typeof status === 'string') return status;
  return status.kind || Object.keys(status)[0] || 'Unknown';
}

async function exportPositions(): Promise<ExportedPosition[]> {
  const sails = await getSails();
  setStatus('refreshing contract positions…');
  const result = await sails.services.ReputationOracle.queries.ExportUsagePredictionsChunk(0, 100).call() as { items?: ExportedPosition[] };
  const items = Array.isArray(result?.items) ? result.items : [];
  setStatus(`loaded ${items.length} contract positions`);
  return items.map((item) => ({ ...item, status: typeof item.status === 'string' ? item.status : { kind: statusKind(item.status) } }));
}

async function connect() {
  setStatus('requesting wallet access…');
  const extensions = await web3Enable('Vara Agent Markets');
  if (!extensions.length) throw new Error('No Polkadot/SubWallet/Talisman extension approved access');
  const accounts = await web3Accounts({ ss58Format: 137 });
  if (!accounts.length) throw new Error('No Vara-compatible account found in wallet extension');
  selectedAccount = accounts[0];
  selectedAccountHex = u8aToHex(decodeAddress(selectedAccount.address));
  setStatus(`connected: ${selectedAccount.meta?.name || 'wallet account'}`);
  window.dispatchEvent(new CustomEvent('vara-markets-wallet', { detail: { address: selectedAccount.address, addressHex: selectedAccountHex } }));
  return {
    address: selectedAccount.address,
    name: selectedAccount.meta?.name || 'wallet account',
    source: selectedAccount.meta?.source || 'extension',
  };
}

async function openPrediction(input: OpenPredictionInput): Promise<TxResult> {
  if (!selectedAccount) await connect();
  if (!selectedAccount) throw new Error('Connect wallet first');

  const subject = sanitizeSubject(input.subject);
  const predicted = sanitizePrediction(Number(input.predictedDeltaCalls));
  const stake = stakeToPlancks(Number(input.stakeVara));
  const evidence = input.evidence.trim() || `web:vara-agent-markets:${subject}:${predicted}`;
  if (evidence.length > 160) throw new Error('Evidence note/hash is too long');

  const { epochId, startMs, endMs } = nextEpochWindow();
  const injector = await web3FromAddress(selectedAccount.address);
  const sails = await getSails();
  const tx = sails.services.ReputationOracle.functions
    .OpenUsagePrediction(epochId, subject, startMs, endMs, predicted, evidence)
    .withAccount(selectedAccount.address, { signer: injector.signer })
    .withValue(stake);

  setStatus('estimating gas…');
  await tx.calculateGas(false, 20);
  setStatus('waiting for wallet signature…');
  const result = await tx.signAndSend();
  setStatus('transaction included; waiting for program reply…');
  let response: unknown = null;
  try {
    response = await result.response();
    setStatus('prediction opened; refreshing positions…');
    const positions = await exportPositions();
    window.dispatchEvent(new CustomEvent('vara-markets-positions', { detail: { positions } }));
  } catch (error) {
    setStatus('transaction sent, but reply decode failed');
    throw error;
  }
  return {
    txHash: result.txHash,
    blockHash: result.blockHash,
    msgId: result.msgId,
    response,
  };
}

async function disconnect() {
  selectedAccount = null;
  selectedAccountHex = null;
  window.dispatchEvent(new CustomEvent('vara-markets-wallet', { detail: { address: null, addressHex: null } }));
  setStatus('wallet not connected');
}

window.varaMarketsWallet = {
  connect,
  openPrediction,
  exportPositions,
  disconnect,
  state: () => ({ connected: !!selectedAccount, address: selectedAccount?.address || null, addressHex: selectedAccountHex, status: statusText }),
};

setStatus('wallet module ready');
