import { createRemoteJWKSet, jwtVerify } from 'jose';

export function parseCookie(header, name) {
  if (!header) return null;
  for (const part of header.split(';')) {
    const index = part.indexOf('=');
    if (index < 0) continue;
    const key = part.slice(0, index).trim();
    if (key !== name) continue;
    try { return decodeURIComponent(part.slice(index + 1).trim()); }
    catch { return null; }
  }
  return null;
}

export function createAuthenticator({ issuer, audience, jwksUrl, cookieName, db, onReject }) {
  if (!issuer || !audience || !jwksUrl) throw new Error('OIDC issuer, audience, and JWKS URL are required');
  const jwks = createRemoteJWKSet(new URL(jwksUrl), {
    timeoutDuration: 5000,
    cooldownDuration: 30000,
    cacheMaxAge: 600000
  });

  return async function authenticate(req) {
    const token = parseCookie(req.headers.cookie, cookieName);
    if (!token) {
      await onReject(req, 'missing_session_cookie');
      return null;
    }

    let claims;
    try {
      ({ payload: claims } = await jwtVerify(token, jwks, {
        issuer,
        audience,
        algorithms: ['RS256', 'ES256'],
        clockTolerance: 5,
        maxTokenAge: '5 minutes',
        requiredClaims: ['iss', 'sub', 'aud', 'exp', 'iat']
      }));
    } catch {
      await onReject(req, 'invalid_or_expired_session');
      return null;
    }

    if (typeof claims.sub !== 'string' || claims.sub.length < 1 || claims.sub.length > 512) {
      await onReject(req, 'invalid_session_subject');
      return null;
    }

    // Roles and organization scope are never trusted from token claims; query the database each request.
    const result = await db.query(
      `SELECT a.id, a.status
         FROM iam.external_identities i
         JOIN iam.accounts a ON a.id = i.account_id
        WHERE i.issuer = $1 AND i.subject = $2
        LIMIT 1`,
      [issuer, claims.sub]
    );
    const account = result.rows[0];
    if (!account || account.status !== 'active') {
      await onReject(req, 'account_unlinked_or_inactive');
      return null;
    }
    return { accountId: account.id, issuer, subject: claims.sub };
  };
}
