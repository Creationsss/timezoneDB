use crate::config::DatabaseConfig;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::fs;
use std::path::Path;
use std::time::Duration;
use tracing::{error, info, warn};

pub async fn connect(config: &DatabaseConfig) -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .acquire_timeout(Duration::from_secs(config.connect_timeout_seconds))
        .idle_timeout(Some(Duration::from_secs(600)))
        .max_lifetime(Some(Duration::from_secs(1800)))
        .connect(&config.url)
        .await?;

    create_migrations_table(&pool).await?;

    run_migrations(&pool).await?;

    Ok(pool)
}

async fn create_migrations_table(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS schema_migrations (
            version TEXT PRIMARY KEY,
            applied_at TIMESTAMPTZ DEFAULT NOW()
        )
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::Error> {
    let migrations_dir = Path::new("migrations");

    if !migrations_dir.exists() {
        warn!("Migrations directory not found, skipping migrations");
        return Ok(());
    }

    let mut migration_files = Vec::new();

    match fs::read_dir(migrations_dir) {
        Ok(entries) => {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("sql") {
                        if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
                            migration_files.push(file_name.to_string());
                        }
                    }
                }
            }
        }
        Err(e) => {
            error!("Failed to read migrations directory: {}", e);
            return Err(sqlx::Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to read migrations directory: {}", e),
            )));
        }
    }

    migration_files.sort();

    for migration_file in migration_files {
        let applied = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = $1)",
        )
        .bind(&migration_file)
        .fetch_one(pool)
        .await?;

        if applied {
            info!("Migration {} already applied, skipping", migration_file);
            continue;
        }

        let migration_path = migrations_dir.join(&migration_file);
        let migration_sql = match fs::read_to_string(&migration_path) {
            Ok(content) => content,
            Err(e) => {
                error!("Failed to read migration file {}: {}", migration_file, e);
                return Err(sqlx::Error::Io(e));
            }
        };

        info!("Running migration: {}", migration_file);

        let mut tx = pool.begin().await?;

        let statements: Vec<&str> = migration_sql
            .split(';')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        for statement in statements {
            if let Err(e) = sqlx::query(statement).execute(&mut *tx).await {
                error!("Failed to execute migration {}: {}", migration_file, e);
                return Err(e);
            }
        }

        sqlx::query("INSERT INTO schema_migrations (version) VALUES ($1)")
            .bind(&migration_file)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;
        info!("Successfully applied migration: {}", migration_file);
    }

    Ok(())
}
