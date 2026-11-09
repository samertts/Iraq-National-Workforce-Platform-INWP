import pg from 'pg';

const { Pool } = pg;

export function createPool(env = process.env) {
  if (!env.DATABASE_URL) throw new Error('DATABASE_URL is required');
  const production = env.NODE_ENV === 'production';
  let ssl = false;
  if (production) {
    if (!env.DB_CA_PEM) throw new Error('DB_CA_PEM is required in production');
    const databaseUrl = new URL(env.DATABASE_URL);
    if (databaseUrl.searchParams.has('sslmode')) {
      throw new Error('Remove sslmode from DATABASE_URL; TLS is enforced with DB_CA_PEM');
    }
    ssl = { rejectUnauthorized: true, ca: env.DB_CA_PEM };
  }
  return new Pool({
    connectionString: env.DATABASE_URL,
    ssl,
    max: Number(env.DB_POOL_MAX || 10),
    connectionTimeoutMillis: 5000,
    idleTimeoutMillis: 30000,
    allowExitOnIdle: false,
    options: '-c statement_timeout=5000 -c lock_timeout=3000 -c idle_in_transaction_session_timeout=10000'
  });
}
