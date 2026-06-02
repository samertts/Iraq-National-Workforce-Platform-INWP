use crate::core::node::{NodeIdentity, NodeType, NodeStatus, Capabilities};
use crate::error::{SyncEngineError, SyncResult};
use sqlx::PgPool;

#[derive(Clone)]
pub struct NodeRepo {
    pool: PgPool,
}

impl NodeRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert(&self, node: &NodeIdentity) -> SyncResult<()> {
        sqlx::query(
            r#"
            INSERT INTO sync.node_registry
                (node_id, node_type, node_name, ministry_id, site_id, region,
                 certificate_serial, public_key, address, port, capabilities,
                 status, last_heartbeat, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            ON CONFLICT (node_id) DO UPDATE SET
                node_type = EXCLUDED.node_type,
                node_name = EXCLUDED.node_name,
                address = EXCLUDED.address,
                port = EXCLUDED.port,
                capabilities = EXCLUDED.capabilities,
                status = EXCLUDED.status,
                last_heartbeat = EXCLUDED.last_heartbeat,
                metadata = EXCLUDED.metadata
            "#,
        )
        .bind(node.node_id)
        .bind(node.node_type.as_str())
        .bind(&node.node_name)
        .bind(node.ministry_id)
        .bind(node.site_id)
        .bind(&node.region)
        .bind(&node.certificate_serial)
        .bind(&node.public_key)
        .bind(&node.address)
        .bind(node.port as i32)
        .bind(serde_json::to_value(&node.capabilities)?)
        .bind(node.status.as_str())
        .bind(node.last_heartbeat)
        .bind(serde_json::Value::Null)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn find_by_id(&self, node_id: uuid::Uuid) -> SyncResult<Option<NodeIdentity>> {
        let row = sqlx::query_as::<_, NodeRow>(
            "SELECT * FROM sync.node_registry WHERE node_id = $1",
        )
        .bind(node_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(TryInto::try_into).transpose()
    }

    pub async fn find_peers(
        &self,
        region: &str,
        node_type: Option<NodeType>,
    ) -> SyncResult<Vec<NodeIdentity>> {
        let mut query = "SELECT * FROM sync.node_registry WHERE region = $1".to_string();
        if let Some(_nt) = node_type {
            query.push_str(" AND node_type = $2");
        }

        let rows = if let Some(nt) = node_type {
            sqlx::query_as::<_, NodeRow>(&query)
                .bind(region)
                .bind(nt.as_str())
                .fetch_all(&self.pool)
                .await?
        } else {
            sqlx::query_as::<_, NodeRow>(&query)
                .bind(region)
                .fetch_all(&self.pool)
                .await?
        };

        rows.into_iter().map(|r| r.try_into()).collect()
    }

    pub async fn update_heartbeat(&self, node_id: uuid::Uuid) -> SyncResult<()> {
        sqlx::query(
            "UPDATE sync.node_registry SET last_heartbeat = $1, status = 'online' WHERE node_id = $2",
        )
        .bind(chrono::Utc::now())
        .bind(node_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn mark_offline(&self, node_id: uuid::Uuid) -> SyncResult<()> {
        sqlx::query(
            "UPDATE sync.node_registry SET status = 'offline' WHERE node_id = $1",
        )
        .bind(node_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn list_all(&self) -> SyncResult<Vec<NodeIdentity>> {
        let rows = sqlx::query_as::<_, NodeRow>("SELECT * FROM sync.node_registry")
            .fetch_all(&self.pool)
            .await?;
        rows.into_iter().map(|r| r.try_into()).collect()
    }
}

#[derive(sqlx::FromRow)]
struct NodeRow {
    node_id: uuid::Uuid,
    node_type: String,
    node_name: String,
    ministry_id: uuid::Uuid,
    site_id: uuid::Uuid,
    region: String,
    certificate_serial: String,
    public_key: Vec<u8>,
    address: String,
    port: i32,
    capabilities: serde_json::Value,
    status: String,
    last_heartbeat: chrono::DateTime<chrono::Utc>,
}

impl TryFrom<NodeRow> for NodeIdentity {
    type Error = SyncEngineError;

    fn try_from(row: NodeRow) -> Result<Self, Self::Error> {
        let node_type = match row.node_type.as_str() {
            "national_hub" => NodeType::NationalHub,
            "regional_relay" => NodeType::RegionalRelay,
            "edge" => NodeType::Edge,
            "mobile" => NodeType::Mobile,
            "dr_replica" => NodeType::DrReplica,
            other => return Err(SyncEngineError::Internal(format!("Unknown node type: {}", other))),
        };

        let status = match row.status.as_str() {
            "online" => NodeStatus::Online,
            "offline" => NodeStatus::Offline,
            "suspected" => NodeStatus::Suspected,
            "recovering" => NodeStatus::Recovering,
            "quarantined" => NodeStatus::Quarantined,
            _ => NodeStatus::Offline,
        };

        let capabilities: Capabilities = serde_json::from_value(row.capabilities)?;

        Ok(NodeIdentity {
            node_id: row.node_id,
            node_type,
            node_name: row.node_name,
            ministry_id: row.ministry_id,
            site_id: row.site_id,
            region: row.region,
            certificate_serial: row.certificate_serial,
            public_key: row.public_key,
            address: row.address,
            port: row.port as u16,
            capabilities,
            status,
            last_heartbeat: row.last_heartbeat,
        })
    }
}
