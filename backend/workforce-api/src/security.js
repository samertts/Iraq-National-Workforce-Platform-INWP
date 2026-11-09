import { randomUUID } from 'node:crypto';

export class HttpError extends Error {
  constructor(status, code, message = code) {
    super(message);
    this.status = status;
    this.code = code;
  }
}

export function requestId(req) {
  // Do not trust caller-provided correlation IDs as the primary security trace identifier.
  return randomUUID();
}

export function applySecurityHeaders(res, id) {
  res.setHeader('content-type', 'application/json; charset=utf-8');
  res.setHeader('cache-control', 'no-store');
  res.setHeader('x-content-type-options', 'nosniff');
  res.setHeader('x-frame-options', 'DENY');
  res.setHeader('referrer-policy', 'no-referrer');
  res.setHeader('content-security-policy', "default-src 'none'; frame-ancestors 'none'");
  res.setHeader('x-request-id', id);
}

export function enforceSameOrigin(req, configuredOrigin) {
  const origin = req.headers.origin;
  if (!origin || !configuredOrigin) throw new HttpError(403, 'csrf_origin_required');
  try {
    if (new URL(origin).origin !== new URL(configuredOrigin).origin) {
      throw new HttpError(403, 'csrf_origin_mismatch');
    }
  } catch (error) {
    if (error instanceof HttpError) throw error;
    throw new HttpError(403, 'csrf_origin_invalid');
  }
  const fetchSite = req.headers['sec-fetch-site'];
  if (fetchSite && !['same-origin', 'same-site', 'none'].includes(fetchSite)) {
    throw new HttpError(403, 'csrf_fetch_site_rejected');
  }
}

export async function readJson(req, maxBytes = 8192) {
  const contentType = (req.headers['content-type'] || '').split(';', 1)[0].trim().toLowerCase();
  if (contentType !== 'application/json') throw new HttpError(415, 'application_json_required');
  const declared = Number(req.headers['content-length'] || 0);
  if (declared > maxBytes) throw new HttpError(413, 'request_too_large');
  const chunks = [];
  let size = 0;
  for await (const chunk of req) {
    size += chunk.length;
    if (size > maxBytes) throw new HttpError(413, 'request_too_large');
    chunks.push(chunk);
  }
  let value;
  try { value = JSON.parse(Buffer.concat(chunks).toString('utf8')); }
  catch { throw new HttpError(400, 'invalid_json'); }
  if (!value || typeof value !== 'object' || Array.isArray(value)) throw new HttpError(400, 'json_object_required');
  return value;
}

export function sendJson(res, status, body) {
  const payload = JSON.stringify(body);
  res.statusCode = status;
  res.setHeader('content-length', Buffer.byteLength(payload));
  res.end(payload);
}
