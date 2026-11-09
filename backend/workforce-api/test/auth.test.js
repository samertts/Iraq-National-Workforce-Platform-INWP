import test from 'node:test';
import assert from 'node:assert/strict';
import { createServer } from 'node:http';
import { exportJWK, generateKeyPair, SignJWT } from 'jose';
import { createAuthenticator } from '../src/auth.js';

const issuer = 'https://issuer.example.test';
const audience = 'inwp-workforce-api';

test('OIDC cookie JWT must verify issuer, audience, signature, and active account mapping', async (t) => {
  const { publicKey, privateKey } = await generateKeyPair('RS256');
  const jwk = await exportJWK(publicKey);
  jwk.kid = 'test-key-1';
  jwk.use = 'sig';
  jwk.alg = 'RS256';
  const jwksServer = createServer((req, res) => {
    res.writeHead(200, { 'content-type': 'application/json', 'cache-control': 'public, max-age=60' });
    res.end(JSON.stringify({ keys: [jwk] }));
  });
  await new Promise((resolve) => jwksServer.listen(0, '127.0.0.1', resolve));
  t.after(() => jwksServer.close());
  const jwksUrl = `http://127.0.0.1:${jwksServer.address().port}/jwks`;
  const rejects = [];
  const db = {
    async query(sql, params) {
      assert.match(sql, /iam\.external_identities/);
      assert.deepEqual(params, [issuer, 'subject-7']);
      return { rows: [{ id: 'account-uuid', status: 'active' }] };
    }
  };
  const authenticate = createAuthenticator({
    issuer, audience, jwksUrl, cookieName: '__Host-inwp_access', db,
    onReject: async (_req, reason) => rejects.push(reason)
  });
  const token = await new SignJWT({})
    .setProtectedHeader({ alg: 'RS256', kid: 'test-key-1', typ: 'JWT' })
    .setIssuer(issuer).setAudience(audience).setSubject('subject-7')
    .setIssuedAt().setExpirationTime('2m').sign(privateKey);
  const result = await authenticate({ headers: { cookie: `__Host-inwp_access=${token}` } });
  assert.deepEqual(result, { accountId: 'account-uuid', issuer, subject: 'subject-7' });
  assert.deepEqual(rejects, []);
});
