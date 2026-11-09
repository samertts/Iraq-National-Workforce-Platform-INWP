import test from 'node:test';
import assert from 'node:assert/strict';
import { enforceSameOrigin, HttpError, readJson } from '../src/security.js';
import { parseCookie } from '../src/auth.js';

function request(chunks, headers = {}) {
  return {
    headers,
    async *[Symbol.asyncIterator]() { for (const chunk of chunks) yield Buffer.from(chunk); }
  };
}

test('cookie parser selects exact cookie and safely decodes its value', () => {
  assert.equal(parseCookie('x=1; __Host-inwp_access=abc%2Edef; y=2', '__Host-inwp_access'), 'abc.def');
  assert.equal(parseCookie('not__Host-inwp_access=x', '__Host-inwp_access'), null);
  assert.equal(parseCookie('__Host-inwp_access=%ZZ', '__Host-inwp_access'), null);
});

test('unsafe request requires exact configured origin', () => {
  assert.doesNotThrow(() => enforceSameOrigin({ headers: { origin: 'https://inwp.example', 'sec-fetch-site': 'same-origin' } }, 'https://inwp.example'));
  assert.throws(() => enforceSameOrigin({ headers: { origin: 'https://evil.example' } }, 'https://inwp.example'), HttpError);
  assert.throws(() => enforceSameOrigin({ headers: {} }, 'https://inwp.example'), (e) => e.code === 'csrf_origin_required');
});

test('JSON body accepts object, rejects wrong content type and arrays', async () => {
  const good = await readJson(request(['{"employment_status":"active"}'], { 'content-type': 'application/json' }));
  assert.deepEqual(good, { employment_status: 'active' });
  await assert.rejects(readJson(request(['{}'], { 'content-type': 'text/plain' })), (e) => e.status === 415);
  await assert.rejects(readJson(request(['[]'], { 'content-type': 'application/json' })), (e) => e.status === 400);
});

test('JSON body enforces byte limit while streaming', async () => {
  await assert.rejects(readJson(request(['{"a":"12345"}'], { 'content-type': 'application/json' }), 4), (e) => e.status === 413);
});
