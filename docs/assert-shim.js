export default function assert(condition, message) { if (!condition) throw new Error(message || 'Assertion failed'); }
export function ok(condition, message) { return assert(condition, message); }
