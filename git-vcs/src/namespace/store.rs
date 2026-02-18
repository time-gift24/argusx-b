use anyhow::Context;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use std::path::Path;
use std::sync::Mutex;

#[derive(Debug, Clone)]
pub struct OrgId(pub i64);
#[derive(Debug, Clone)]
pub struct ProjectId(pub i64);

#[derive(Debug, Clone)]
pub struct Organization {
    pub id: OrgId,
    pub name: String,
    pub gitolite_path: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct Project {
    pub id: ProjectId,
    pub org_id: OrgId,
    pub name: String,
    pub description: Option<String>,
    pub default_branch: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub trait NamespaceStore: Send + Sync {
    fn create_org(&self, name: &str, gitolite_path: &str) -> anyhow::Result<OrgId>;
    fn get_org(&self, id: OrgId) -> anyhow::Result<Organization>;
    fn list_orgs(&self) -> anyhow::Result<Vec<Organization>>;
    fn delete_org(&self, id: OrgId) -> anyhow::Result<()>;

    fn create_project(
        &self,
        org_id: OrgId,
        name: &str,
        description: Option<&str>,
    ) -> anyhow::Result<ProjectId>;
    fn get_project(&self, id: ProjectId) -> anyhow::Result<Project>;
    fn list_projects(&self, org_id: OrgId) -> anyhow::Result<Vec<Project>>;
    fn delete_project(&self, id: ProjectId) -> anyhow::Result<()>;
}

pub struct SqliteNamespaceStore {
    conn: Mutex<Connection>,
}

impl SqliteNamespaceStore {
    pub fn new(path: &Path) -> anyhow::Result<Self> {
        let conn = Connection::open(path).context("Failed to open database")?;
        let store = Self {
            conn: Mutex::new(conn),
        };
        store.init_schema()?;
        Ok(store)
    }

    fn init_schema(&self) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS organizations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                gitolite_path TEXT NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS projects (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                org_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                description TEXT,
                default_branch TEXT DEFAULT 'main',
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (org_id) REFERENCES organizations(id),
                UNIQUE(org_id, name)
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS git_configs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id INTEGER NOT NULL,
                key TEXT NOT NULL,
                value TEXT NOT NULL,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (project_id) REFERENCES projects(id),
                UNIQUE(project_id, key)
            )",
            [],
        )?;

        Ok(())
    }
}

impl NamespaceStore for SqliteNamespaceStore {
    fn create_org(&self, name: &str, gitolite_path: &str) -> anyhow::Result<OrgId> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO organizations (name, gitolite_path) VALUES (?1, ?2)",
            params![name, gitolite_path],
        )?;
        Ok(OrgId(conn.last_insert_rowid()))
    }

    fn get_org(&self, id: OrgId) -> anyhow::Result<Organization> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, gitolite_path, created_at, updated_at FROM organizations WHERE id = ?1"
        )?;

        let org = stmt.query_row(params![id.0], |row| {
            Ok(Organization {
                id: OrgId(row.get(0)?),
                name: row.get(1)?,
                gitolite_path: row.get(2)?,
                created_at: row.get::<_, String>(3)?.parse().unwrap_or_default(),
                updated_at: row.get::<_, String>(4)?.parse().unwrap_or_default(),
            })
        })?;

        Ok(org)
    }

    fn list_orgs(&self) -> anyhow::Result<Vec<Organization>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT id, name, gitolite_path, created_at, updated_at FROM organizations")?;

        let orgs = stmt
            .query_map([], |row| {
                Ok(Organization {
                    id: OrgId(row.get(0)?),
                    name: row.get(1)?,
                    gitolite_path: row.get(2)?,
                    created_at: row.get::<_, String>(3)?.parse().unwrap_or_default(),
                    updated_at: row.get::<_, String>(4)?.parse().unwrap_or_default(),
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(orgs)
    }

    fn delete_org(&self, id: OrgId) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM organizations WHERE id = ?1", params![id.0])?;
        Ok(())
    }

    fn create_project(
        &self,
        org_id: OrgId,
        name: &str,
        description: Option<&str>,
    ) -> anyhow::Result<ProjectId> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO projects (org_id, name, description) VALUES (?1, ?2, ?3)",
            params![org_id.0, name, description],
        )?;
        Ok(ProjectId(conn.last_insert_rowid()))
    }

    fn get_project(&self, id: ProjectId) -> anyhow::Result<Project> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, org_id, name, description, default_branch, created_at, updated_at
             FROM projects WHERE id = ?1",
        )?;

        let project = stmt.query_row(params![id.0], |row| {
            Ok(Project {
                id: ProjectId(row.get(0)?),
                org_id: OrgId(row.get(1)?),
                name: row.get(2)?,
                description: row.get(3)?,
                default_branch: row.get(4)?,
                created_at: row.get::<_, String>(5)?.parse().unwrap_or_default(),
                updated_at: row.get::<_, String>(6)?.parse().unwrap_or_default(),
            })
        })?;

        Ok(project)
    }

    fn list_projects(&self, org_id: OrgId) -> anyhow::Result<Vec<Project>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, org_id, name, description, default_branch, created_at, updated_at
             FROM projects WHERE org_id = ?1",
        )?;

        let projects = stmt
            .query_map(params![org_id.0], |row| {
                Ok(Project {
                    id: ProjectId(row.get(0)?),
                    org_id: OrgId(row.get(1)?),
                    name: row.get(2)?,
                    description: row.get(3)?,
                    default_branch: row.get(4)?,
                    created_at: row.get::<_, String>(5)?.parse().unwrap_or_default(),
                    updated_at: row.get::<_, String>(6)?.parse().unwrap_or_default(),
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(projects)
    }

    fn delete_project(&self, id: ProjectId) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM projects WHERE id = ?1", params![id.0])?;
        Ok(())
    }
}
