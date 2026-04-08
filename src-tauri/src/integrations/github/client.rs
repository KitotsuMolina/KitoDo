use anyhow::{anyhow, Context, Result};
use reqwest::blocking::{Client, RequestBuilder};

use super::models::{EtagFetch, GithubUser, NotificationItem, RepoItem, SearchResponse};

const API_BASE: &str = "https://api.github.com";

pub struct GithubClient {
    http: Client,
    token: String,
}

impl GithubClient {
    pub fn new(token: String) -> Result<Self> {
        let http = Client::builder().build()?;
        Ok(Self { http, token })
    }

    pub fn get_user(&self) -> Result<GithubUser> {
        self.get_json(format!("{API_BASE}/user"), None)
    }

    pub fn detect_token_kind(&self) -> Result<String> {
        let req = self
            .base(self.http.get(format!("{API_BASE}/notifications")))
            .query(&[("per_page", "1")]);
        let response = req.send().context("No se pudo consultar notifications")?;

        if response.status().is_success() {
            return Ok("classic".to_string());
        }

        let status = response.status().as_u16();
        if status == 401 || status == 403 {
            return Ok("unknown".to_string());
        }

        Ok("unknown".to_string())
    }

    pub fn list_repos(&self) -> Result<Vec<RepoItem>> {
        self.get_json(
            format!("{API_BASE}/user/repos?per_page=100&sort=updated&affiliation=owner,collaborator,organization_member"),
            None,
        )
    }

    pub fn search_review_requested_prs(
        &self,
        owner: &str,
        repo: &str,
        etag: Option<&str>,
    ) -> Result<EtagFetch<Vec<super::models::SearchItem>>> {
        let q = format!("is:pr is:open review-requested:@me repo:{owner}/{repo}");
        self.search_issues(&q, etag)
    }

    pub fn search_assigned_issues(
        &self,
        owner: &str,
        repo: &str,
        etag: Option<&str>,
    ) -> Result<EtagFetch<Vec<super::models::SearchItem>>> {
        let q = format!("is:issue is:open assignee:@me repo:{owner}/{repo}");
        self.search_issues(&q, etag)
    }

    fn search_issues(
        &self,
        query: &str,
        etag: Option<&str>,
    ) -> Result<EtagFetch<Vec<super::models::SearchItem>>> {
        let mut req = self
            .base(self.http.get(format!("{API_BASE}/search/issues")))
            .query(&[
                ("q", query),
                ("sort", "updated"),
                ("order", "desc"),
                ("per_page", "50"),
            ]);

        if let Some(tag) = etag {
            req = req.header("If-None-Match", tag);
        }

        let response = req.send().context("No se pudo consultar search/issues")?;
        if response.status().as_u16() == 304 {
            return Ok(EtagFetch {
                not_modified: true,
                etag: None,
                data: Vec::new(),
            });
        }

        if !response.status().is_success() {
            return Err(anyhow!(
                "GitHub search/issues falló con status {}",
                response.status()
            ));
        }

        let etag = response
            .headers()
            .get("etag")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let payload: SearchResponse = response
            .json()
            .context("Respuesta inválida de search/issues")?;

        Ok(EtagFetch {
            not_modified: false,
            etag,
            data: payload.items,
        })
    }

    pub fn list_repo_notifications(
        &self,
        owner: &str,
        repo: &str,
        etag: Option<&str>,
    ) -> Result<EtagFetch<Vec<NotificationItem>>> {
        let url = format!("{API_BASE}/repos/{owner}/{repo}/notifications");
        let mut req = self.base(self.http.get(url)).query(&[
            ("all", "false"),
            ("participating", "true"),
            ("per_page", "50"),
        ]);

        if let Some(tag) = etag {
            req = req.header("If-None-Match", tag);
        }

        let response = req
            .send()
            .context("No se pudo consultar notifications del repo")?;
        if response.status().as_u16() == 304 {
            return Ok(EtagFetch {
                not_modified: true,
                etag: None,
                data: Vec::new(),
            });
        }

        if !response.status().is_success() {
            return Err(anyhow!(
                "GitHub notifications falló con status {}",
                response.status()
            ));
        }

        let etag = response
            .headers()
            .get("etag")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let payload: Vec<NotificationItem> = response
            .json()
            .context("Respuesta inválida de notifications")?;

        Ok(EtagFetch {
            not_modified: false,
            etag,
            data: payload,
        })
    }

    fn get_json<T: serde::de::DeserializeOwned>(
        &self,
        url: String,
        if_none_match: Option<&str>,
    ) -> Result<T> {
        let mut req = self.base(self.http.get(url));
        if let Some(tag) = if_none_match {
            req = req.header("If-None-Match", tag);
        }

        let response = req.send().context("Request GitHub falló")?;
        if !response.status().is_success() {
            return Err(anyhow!("GitHub API status {}", response.status()));
        }

        response.json().context("No se pudo parsear JSON de GitHub")
    }

    fn base(&self, req: RequestBuilder) -> RequestBuilder {
        req.header("Accept", "application/vnd.github+json")
            .header("Authorization", format!("Bearer {}", self.token))
            .header("User-Agent", "KitoDo/0.4")
            .header("X-GitHub-Api-Version", "2022-11-28")
    }
}
