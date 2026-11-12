//! Dashboard read model.

use chrono::{DateTime, NaiveDate, Utc};

use crate::entity::{role::RoleWithCount, user::UserWithRole};

/// Raw dashboard facts loaded by the adapter.
#[derive(Debug, Clone)]
pub struct DashboardSnapshot {
    pub total_users: i64,
    pub active_users: i64,
    pub pending_users: i64,
    pub roles: Vec<RoleWithCount>,
    pub recent_users: Vec<UserWithRole>,
    pub signup_times: Vec<DateTime<Utc>>,
}

/// One point in the cumulative 30-day signup chart.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignupPoint {
    pub date: NaiveDate,
    pub users: i64,
}

/// The dashboard read model returned by the use case.
#[derive(Debug, Clone)]
pub struct DashboardData {
    pub total_users: i64,
    pub active_users: i64,
    pub pending_users: i64,
    pub roles_count: i64,
    pub role_data: Vec<RoleDatum>,
    pub signup_data: Vec<SignupPoint>,
    pub recent_users: Vec<UserWithRole>,
}

/// One bar in the role distribution chart.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoleDatum {
    pub name: String,
    pub label: String,
    pub users: i64,
}
