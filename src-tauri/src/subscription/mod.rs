use anyhow::Result;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::api::client::HiNotesClient;
use crate::api::types::SubscriptionStatus as ApiSubscriptionStatus;
use crate::db::types::{
    DbSubscription, InsertSubscription, InsertSubscriptionEvent, SubscriptionEvent,
    SubscriptionEventType, SubscriptionStatus as DbSubscriptionStatus, UpdateSubscription,
};
use crate::db::Database;

/// Convert API subscription status to database subscription status
fn api_status_to_db_status(status: &ApiSubscriptionStatus) -> DbSubscriptionStatus {
    match status {
        ApiSubscriptionStatus::Active => DbSubscriptionStatus::Active,
        ApiSubscriptionStatus::Expired => DbSubscriptionStatus::Expired,
        ApiSubscriptionStatus::Canceled => DbSubscriptionStatus::Canceled,
        ApiSubscriptionStatus::Trial => DbSubscriptionStatus::Trial,
    }
}

pub struct SubscriptionManager {
    db: Arc<RwLock<Database>>,
    api_client: Arc<HiNotesClient>,
}

impl SubscriptionManager {
    pub fn new(db: Arc<RwLock<Database>>, api_client: Arc<HiNotesClient>) -> Self {
        Self { db, api_client }
    }

    /// Get the current subscription from the database
    pub async fn get_current_subscription(&self) -> Result<Option<DbSubscription>> {
        let db = self.db.read().await;
        db.get_current_subscription()
    }

    /// Sync subscription status from API to database
    pub async fn sync_subscription_status(&self) -> Result<DbSubscription> {
        // Fetch from API
        let api_subscription = self.api_client.get_subscription_status().await?;

        // Get current subscription from DB
        let db = self.db.write().await;
        let current = db.get_current_subscription()?;

        // Determine if we need to update or insert
        let subscription = if let Some(current_sub) = current {
            // Update existing subscription
            let update = UpdateSubscription {
                status: Some(api_status_to_db_status(&api_subscription.status)),
                expires_at: api_subscription
                    .expires_at
                    .as_ref()
                    .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
                canceled_at: if api_subscription.status == ApiSubscriptionStatus::Canceled {
                    Some(Utc::now())
                } else {
                    None
                },
            };

            db.update_subscription(current_sub.id, &update)?
        } else {
            // Insert new subscription
            let insert = InsertSubscription {
                product_id: api_subscription.product_id.clone(),
                status: api_status_to_db_status(&api_subscription.status),
                expires_at: api_subscription
                    .expires_at
                    .as_ref()
                    .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
                purchased_at: Some(Utc::now()),
            };

            db.insert_subscription(&insert)?
        };

        Ok(subscription)
    }

    /// Check if user has an active subscription
    pub async fn has_active_subscription(&self) -> Result<bool> {
        let db = self.db.read().await;
        let subscription = db.get_current_subscription()?;

        match subscription {
            Some(sub) => Ok(sub.status == DbSubscriptionStatus::Active
                || sub.status == DbSubscriptionStatus::Trial),
            None => Ok(false),
        }
    }

    /// Handle subscription event (activated, expired, renewed, canceled)
    pub async fn handle_subscription_event(
        &self,
        event_type: SubscriptionEventType,
        product_id: String,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<()> {
        let db = self.db.write().await;
        let current_sub = db.get_current_subscription()?;

        // Record the event
        let event = InsertSubscriptionEvent {
            subscription_id: current_sub.as_ref().map(|s| s.id),
            event_type: event_type.clone(),
            product_id: product_id.clone(),
            expires_at,
            occurred_at: Utc::now(),
        };

        db.insert_subscription_event(&event)?;

        // Update subscription status based on event
        if let Some(sub) = current_sub {
            let new_status = match event_type {
                SubscriptionEventType::Activated | SubscriptionEventType::Renewed => {
                    DbSubscriptionStatus::Active
                }
                SubscriptionEventType::Expired => DbSubscriptionStatus::Expired,
                SubscriptionEventType::Canceled => DbSubscriptionStatus::Canceled,
            };

            let update = UpdateSubscription {
                status: Some(new_status),
                expires_at,
                canceled_at: if event_type == SubscriptionEventType::Canceled {
                    Some(Utc::now())
                } else {
                    None
                },
            };

            db.update_subscription(sub.id, &update)?;
        } else {
            // Create new subscription if none exists
            let insert = InsertSubscription {
                product_id,
                status: match event_type {
                    SubscriptionEventType::Activated => DbSubscriptionStatus::Active,
                    SubscriptionEventType::Expired => DbSubscriptionStatus::Expired,
                    SubscriptionEventType::Canceled => DbSubscriptionStatus::Canceled,
                    SubscriptionEventType::Renewed => DbSubscriptionStatus::Active,
                },
                expires_at,
                purchased_at: Some(Utc::now()),
            };

            db.insert_subscription(&insert)?;
        }

        Ok(())
    }

    /// Get subscription event history
    pub async fn get_subscription_events(&self, limit: i64) -> Result<Vec<SubscriptionEvent>> {
        let db = self.db.read().await;
        db.list_subscription_events(limit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::types::SubscriptionStatus as ApiSubscriptionStatus;
    use std::path::PathBuf;

    fn setup() -> (Arc<RwLock<Database>>, Arc<HiNotesClient>) {
        let db = Database::new_in_memory().expect("Failed to create in-memory database");
        let api_client = HiNotesClient::new("http://localhost:3001/v1");

        (Arc::new(RwLock::new(db)), Arc::new(api_client))
    }

    #[tokio::test]
    async fn test_get_current_subscription_when_none_exists() {
        let (db, api_client) = setup();
        let manager = SubscriptionManager::new(db.clone(), api_client);

        let result = manager.get_current_subscription().await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_has_active_subscription_returns_false_when_none() {
        let (db, api_client) = setup();
        let manager = SubscriptionManager::new(db.clone(), api_client);

        let result = manager.has_active_subscription().await;

        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[tokio::test]
    async fn test_handle_subscription_event_activated() {
        let (db, api_client) = setup();
        let manager = SubscriptionManager::new(db.clone(), api_client);

        let expires_at = Utc::now() + chrono::Duration::days(30);
        let result = manager
            .handle_subscription_event(
                SubscriptionEventType::Activated,
                "premium_monthly".to_string(),
                Some(expires_at),
            )
            .await;

        assert!(result.is_ok());

        // Verify subscription was created
        let subscription = manager.get_current_subscription().await.unwrap();
        assert!(subscription.is_some());
        let sub = subscription.unwrap();
        assert_eq!(sub.status, DbSubscriptionStatus::Active);
        assert_eq!(sub.product_id, "premium_monthly");
    }

    #[tokio::test]
    async fn test_handle_subscription_event_expired() {
        let (db, api_client) = setup();
        let manager = SubscriptionManager::new(db.clone(), api_client);

        // First activate a subscription
        let expires_at = Utc::now() + chrono::Duration::days(30);
        manager
            .handle_subscription_event(
                SubscriptionEventType::Activated,
                "premium_monthly".to_string(),
                Some(expires_at),
            )
            .await
            .unwrap();

        // Then expire it
        let result = manager
            .handle_subscription_event(
                SubscriptionEventType::Expired,
                "premium_monthly".to_string(),
                None,
            )
            .await;

        assert!(result.is_ok());

        // Verify subscription is now expired
        let subscription = manager.get_current_subscription().await.unwrap();
        assert!(subscription.is_some());
        assert_eq!(subscription.unwrap().status, DbSubscriptionStatus::Expired);
    }

    #[tokio::test]
    async fn test_handle_subscription_event_renewed() {
        let (db, api_client) = setup();
        let manager = SubscriptionManager::new(db.clone(), api_client);

        // First activate a subscription
        let initial_expires = Utc::now() + chrono::Duration::days(30);
        manager
            .handle_subscription_event(
                SubscriptionEventType::Activated,
                "premium_monthly".to_string(),
                Some(initial_expires),
            )
            .await
            .unwrap();

        // Then renew it
        let new_expires = Utc::now() + chrono::Duration::days(60);
        let result = manager
            .handle_subscription_event(
                SubscriptionEventType::Renewed,
                "premium_monthly".to_string(),
                Some(new_expires),
            )
            .await;

        assert!(result.is_ok());

        // Verify subscription is still active with new expiry
        let subscription = manager.get_current_subscription().await.unwrap();
        assert!(subscription.is_some());
        let sub = subscription.unwrap();
        assert_eq!(sub.status, DbSubscriptionStatus::Active);
        assert!(sub.expires_at.is_some());
    }

    #[tokio::test]
    async fn test_handle_subscription_event_canceled() {
        let (db, api_client) = setup();
        let manager = SubscriptionManager::new(db.clone(), api_client);

        // First activate a subscription
        let expires_at = Utc::now() + chrono::Duration::days(30);
        manager
            .handle_subscription_event(
                SubscriptionEventType::Activated,
                "premium_monthly".to_string(),
                Some(expires_at),
            )
            .await
            .unwrap();

        // Then cancel it
        let result = manager
            .handle_subscription_event(
                SubscriptionEventType::Canceled,
                "premium_monthly".to_string(),
                Some(expires_at),
            )
            .await;

        assert!(result.is_ok());

        // Verify subscription is canceled
        let subscription = manager.get_current_subscription().await.unwrap();
        assert!(subscription.is_some());
        let sub = subscription.unwrap();
        assert_eq!(sub.status, DbSubscriptionStatus::Canceled);
        assert!(sub.canceled_at.is_some());
    }

    #[tokio::test]
    async fn test_get_subscription_events() {
        let (db, api_client) = setup();
        let manager = SubscriptionManager::new(db.clone(), api_client);

        // Create multiple events
        manager
            .handle_subscription_event(
                SubscriptionEventType::Activated,
                "premium_monthly".to_string(),
                Some(Utc::now() + chrono::Duration::days(30)),
            )
            .await
            .unwrap();

        manager
            .handle_subscription_event(
                SubscriptionEventType::Renewed,
                "premium_monthly".to_string(),
                Some(Utc::now() + chrono::Duration::days(60)),
            )
            .await
            .unwrap();

        // Get event history
        let events = manager.get_subscription_events(10).await.unwrap();

        assert_eq!(events.len(), 2);
        assert_eq!(events[0].event_type, SubscriptionEventType::Renewed);
        assert_eq!(events[1].event_type, SubscriptionEventType::Activated);
    }

    #[tokio::test]
    async fn test_has_active_subscription_returns_true_for_active() {
        let (db, api_client) = setup();
        let manager = SubscriptionManager::new(db.clone(), api_client);

        // Activate subscription
        manager
            .handle_subscription_event(
                SubscriptionEventType::Activated,
                "premium_monthly".to_string(),
                Some(Utc::now() + chrono::Duration::days(30)),
            )
            .await
            .unwrap();

        let result = manager.has_active_subscription().await;

        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn test_has_active_subscription_returns_false_for_expired() {
        let (db, api_client) = setup();
        let manager = SubscriptionManager::new(db.clone(), api_client);

        // Activate then expire subscription
        manager
            .handle_subscription_event(
                SubscriptionEventType::Activated,
                "premium_monthly".to_string(),
                Some(Utc::now() + chrono::Duration::days(30)),
            )
            .await
            .unwrap();

        manager
            .handle_subscription_event(
                SubscriptionEventType::Expired,
                "premium_monthly".to_string(),
                None,
            )
            .await
            .unwrap();

        let result = manager.has_active_subscription().await;

        assert!(result.is_ok());
        assert!(!result.unwrap());
    }
}
