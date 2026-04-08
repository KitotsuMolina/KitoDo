use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct ExternalCandidate {
    pub external_key: String,
    pub kind: String,
    pub url: String,
    pub title: String,
    pub state: String,
    pub repo_full: String,
    pub number: Option<i64>,
    pub author: Option<String>,
    pub assignee: Option<String>,
    pub updated_at_ext: Option<String>,
    pub payload_json: Option<String>,
    pub is_review_requested: bool,
}

#[derive(Debug)]
pub struct EtagFetch<T> {
    pub not_modified: bool,
    pub etag: Option<String>,
    pub data: T,
}

#[derive(Debug, Deserialize)]
pub struct GithubUser {
    pub login: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SearchResponse {
    pub items: Vec<SearchItem>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SearchItem {
    pub number: i64,
    pub title: String,
    pub html_url: String,
    pub state: String,
    pub updated_at: Option<String>,
    pub user: Option<SearchUser>,
    pub assignee: Option<SearchUser>,
    pub pull_request: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SearchUser {
    pub login: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RepoItem {
    pub name: String,
    pub full_name: String,
    pub private: bool,
    pub owner: RepoOwner,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RepoOwner {
    pub login: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NotificationItem {
    pub id: String,
    pub updated_at: Option<String>,
    pub repository: NotificationRepo,
    pub subject: NotificationSubject,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NotificationRepo {
    pub full_name: String,
    pub html_url: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NotificationSubject {
    pub title: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GithubSyncError {
    pub source: String,
    pub message: String,
}
