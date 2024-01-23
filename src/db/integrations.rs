use sqlx::{prelude::{FromRow, }, Pool, Postgres};



#[derive(FromRow, Debug, Clone)]
pub struct Integration {
    pub id: i32,
    pub name: String,
    pub redirect: String
}

impl Integration {
    pub async fn find_by_slug(connection: &Pool<Postgres>, name: String) -> Integration {
        let integration = sqlx::query_as(
            "SELECT * FROM \"Integration\" WHERE name = $1",
        )
        .bind(name)
        .fetch_one(connection)
        .await
        .unwrap();

        integration
    }
}

#[derive(FromRow, Debug, Clone)]
pub struct LinkedIndegration {
    pub id: String,
    #[sqlx(rename = "displayName")]
    pub display_name: String,
    #[sqlx(rename = "platformId")]
    pub platform_id: String,
    #[sqlx(rename = "avatarUrl")]
    pub avatar_url: String,
    #[sqlx(rename = "userId")]
    pub user_id: i32,
    #[sqlx(rename = "platformType")]
    pub platform_type: i32,
    pub visible: bool
}

impl LinkedIndegration {
    pub async fn fetch_users_with_integration(connection: &Pool<Postgres>, integration: Integration) -> Vec<LinkedIndegration> {
        let users = sqlx::query_as(
            "SELECT * FROM \"LinkedIntegration\" WHERE \"platformType\" = $1",
        )
        .bind(integration.id)
        .fetch_all(connection)
        .await
        .unwrap();

        users
    }
}