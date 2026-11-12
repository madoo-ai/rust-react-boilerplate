//! Dashboard aggregation use case.

use std::{collections::BTreeMap, sync::Arc};

use chrono::{Duration, NaiveDate, TimeZone, Utc};
use chrono_tz::Tz;

use crate::{
    app::{
        error::AppError,
        port::{Clock, DashboardRepository},
    },
    entity::{
        authorization::Actor,
        dashboard::{DashboardData, RoleDatum, SignupPoint},
        permission::Permission,
    },
};

/// The dashboard use case.
pub struct DashboardService {
    repository: Arc<dyn DashboardRepository>,
    clock: Arc<dyn Clock>,
    timezone: Tz,
}

impl std::fmt::Debug for DashboardService {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("DashboardService")
            .field("timezone", &self.timezone)
            .finish_non_exhaustive()
    }
}

impl DashboardService {
    #[must_use]
    pub fn new(
        repository: Arc<dyn DashboardRepository>,
        clock: Arc<dyn Clock>,
        timezone: Tz,
    ) -> Self {
        Self {
            repository,
            clock,
            timezone,
        }
    }

    /// Load the admin dashboard.
    ///
    /// # Errors
    ///
    /// Returns `Forbidden` when the actor lacks `view_admin`, or an
    /// infrastructure error.
    pub async fn load(&self, actor: &Actor) -> Result<DashboardData, AppError> {
        actor.require(Permission::ViewAdmin)?;

        let now = self.clock.now().with_timezone(&self.timezone);
        let today = now.date_naive();
        let first_day = today - Duration::days(29);

        let since = first_day
            .and_hms_opt(0, 0, 0)
            .and_then(|value| self.timezone.from_local_datetime(&value).single())
            .map_or_else(
                || Utc::now() - Duration::days(30),
                |value| value.with_timezone(&Utc),
            );

        let snapshot = self.repository.snapshot(since).await?;

        let mut buckets: BTreeMap<NaiveDate, i64> = BTreeMap::new();
        for offset in 0..30 {
            buckets.insert(first_day + Duration::days(offset), 0);
        }

        for signup in snapshot.signup_times {
            let date = signup.with_timezone(&self.timezone).date_naive();
            if let Some(bucket) = buckets.get_mut(&date) {
                *bucket += 1;
            }
        }

        let mut running = 0;
        let signup_data = buckets
            .into_iter()
            .map(|(date, users)| {
                running += users;
                SignupPoint {
                    date,
                    users: running,
                }
            })
            .collect();

        let role_data = snapshot
            .roles
            .iter()
            .map(|role| RoleDatum {
                name: role.role.name.clone(),
                label: role.role.label.clone(),
                users: role.user_count,
            })
            .collect::<Vec<_>>();

        Ok(DashboardData {
            total_users: snapshot.total_users,
            active_users: snapshot.active_users,
            pending_users: snapshot.pending_users,
            roles_count: i64::try_from(snapshot.roles.len()).unwrap_or(i64::MAX),
            role_data,
            signup_data,
            recent_users: snapshot.recent_users,
        })
    }
}
