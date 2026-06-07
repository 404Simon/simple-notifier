use std::collections::HashSet;

use serde::Deserialize;

use crate::config::{BranchesConfig, RepoConfig};
use crate::notifier::{Notification, Notifier};
use crate::storage::Storage;

pub struct GitHub {
    repos: Vec<RepoConfig>,
    token: Option<String>,
}

impl GitHub {
    pub fn new(repos: Vec<RepoConfig>, token: Option<String>) -> Self {
        Self { repos, token }
    }
}

impl Notifier for GitHub {
    fn name(&self) -> &str {
        "github"
    }

    fn check(&self, storage: &mut Storage) -> Option<Notification> {
        let mut entries: Vec<String> = Vec::new();

        for repo in &self.repos {
            if repo.watch.commits
                && let Some(entry) = check_commits(repo, storage, self.token.as_deref())
            {
                entries.push(entry);
            }
            if repo.watch.prs
                && let Some(entry) = check_prs(repo, storage, self.token.as_deref())
            {
                entries.push(entry);
            }
            if repo.watch.releases
                && let Some(entry) = check_releases(repo, storage, self.token.as_deref())
            {
                entries.push(entry);
            }
        }

        if entries.is_empty() {
            return None;
        }

        Some(Notification {
            title: format!(
                "GitHub update ({} event{})",
                entries.len(),
                if entries.len() == 1 { "" } else { "s" }
            ),
            body: entries.join("\n\n----\n\n"),
        })
    }
}

// ── helpers ──────────────────────────────────────────────────────────────────

fn api_get(path: &str, token: Option<&str>) -> Result<serde_json::Value, String> {
    let url = format!("https://api.github.com{path}");
    let mut req = ureq::get(&url).header("User-Agent", "simple-notifier/0.1");
    if let Some(t) = token {
        req = req.header("Authorization", &format!("Bearer {t}"));
    }
    let resp = req
        .call()
        .map_err(|e| format!("HTTP error fetching {url}: {e}"))?;

    let body = resp
        .into_body()
        .read_to_string()
        .map_err(|e| format!("read error: {e}"))?;

    serde_json::from_str(&body).map_err(|e| format!("JSON parse error: {e}"))
}

fn resolve_branches(repo: &RepoConfig, storage: &mut Storage, token: Option<&str>) -> Vec<String> {
    match &repo.branches {
        BranchesConfig::Default => {
            let cache_key = format!("github_default_branch_{}_{}", repo.owner, repo.repo);
            if let Some(cached) = storage.get(&cache_key) {
                return vec![cached.to_string()];
            }
            let branch = fetch_default_branch(repo, token).unwrap_or_else(|| "main".to_string());
            storage.set(&cache_key, &branch);
            vec![branch]
        }
        BranchesConfig::All => fetch_all_branches(repo, token).unwrap_or_default(),
        BranchesConfig::List(list) => list.clone(),
    }
}

fn fetch_default_branch(repo: &RepoConfig, token: Option<&str>) -> Option<String> {
    let path = format!("/repos/{}/{}", repo.owner, repo.repo);
    let json = api_get(&path, token).ok()?;
    json.get("default_branch")?.as_str().map(|s| s.to_string())
}

fn fetch_all_branches(repo: &RepoConfig, token: Option<&str>) -> Option<Vec<String>> {
    let path = format!("/repos/{}/{}/branches?per_page=100", repo.owner, repo.repo);
    let json = api_get(&path, token).ok()?;
    let arr = json.as_array()?;
    Some(
        arr.iter()
            .filter_map(|v| v.get("name")?.as_str().map(|s| s.to_string()))
            .collect(),
    )
}

// ── commits ──────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct CommitDetail {
    sha: String,
    commit: CommitContent,
}

#[derive(Deserialize)]
struct CommitContent {
    #[serde(default)]
    author: Option<CommitAuthor>,
    message: String,
}

#[derive(Deserialize)]
struct CommitAuthor {
    name: String,
}

fn check_commits(repo: &RepoConfig, storage: &mut Storage, token: Option<&str>) -> Option<String> {
    let branches = resolve_branches(repo, storage, token);
    let mut branch_entries: Vec<String> = Vec::new();

    for branch in &branches {
        let key = format!("github_commits_{}_{}_{}", repo.owner, repo.repo, branch);
        let stored_sha = storage.get(&key).map(|s| s.to_string());

        let latest = fetch_latest_commit_sha(repo, branch, token)?;

        if stored_sha.as_deref() == Some(&latest) {
            continue;
        }

        let commits = match stored_sha {
            Some(ref old) if old != &latest => {
                fetch_commits_between(repo, old, &latest, branch, token).unwrap_or_default()
            }
            _ => vec![], // first time seeing this branch — record SHA, no notification
        };

        storage.set(&key, &latest);

        if commits.is_empty() {
            continue;
        }

        let lines: Vec<String> = commits
            .iter()
            .map(|c| {
                let sha_short = &c.sha[..7.min(c.sha.len())];
                let author = c
                    .commit
                    .author
                    .as_ref()
                    .map(|a| a.name.as_str())
                    .unwrap_or("unknown");
                let first_line = c.commit.message.lines().next().unwrap_or("");
                format!("    {sha_short} - {author} - \"{first_line}\"")
            })
            .collect();

        branch_entries.push(format!("  {branch}:\n{}", lines.join("\n")));
    }

    if branch_entries.is_empty() {
        return None;
    }

    Some(format!(
        "[{}/{}] New commits:\n{}",
        repo.owner,
        repo.repo,
        branch_entries.join("\n")
    ))
}

fn fetch_latest_commit_sha(repo: &RepoConfig, branch: &str, token: Option<&str>) -> Option<String> {
    let path = format!(
        "/repos/{}/{}/commits?sha={}&per_page=1",
        repo.owner, repo.repo, branch
    );
    let json = api_get(&path, token).ok()?;
    let arr = json.as_array()?;
    let first = arr.first()?;
    first.get("sha")?.as_str().map(|s| s.to_string())
}

fn fetch_commits_between(
    repo: &RepoConfig,
    base: &str,
    head: &str,
    branch: &str,
    token: Option<&str>,
) -> Option<Vec<CommitDetail>> {
    let path = format!(
        "/repos/{}/{}/compare/{}...{}",
        repo.owner, repo.repo, base, head
    );
    let json = api_get(&path, token).ok()?;
    // If compare fails (e.g. force-push), fall back to listing commits on the branch
    if json.get("commits").is_none() {
        return fetch_recent_commits(repo, branch, token, 30);
    }
    serde_json::from_value(json["commits"].clone()).ok()
}

fn fetch_recent_commits(
    repo: &RepoConfig,
    branch: &str,
    token: Option<&str>,
    count: usize,
) -> Option<Vec<CommitDetail>> {
    let path = format!(
        "/repos/{}/{}/commits?sha={}&per_page={}",
        repo.owner, repo.repo, branch, count
    );
    let json = api_get(&path, token).ok()?;
    serde_json::from_value(json).ok()
}

// ── pull requests ────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct PRInfo {
    number: i64,
    title: String,
    user: PRUser,
}

#[derive(Deserialize)]
struct PRUser {
    login: String,
}

fn check_prs(repo: &RepoConfig, storage: &mut Storage, token: Option<&str>) -> Option<String> {
    let key = format!("github_prs_max_{}_{}", repo.owner, repo.repo);
    let stored_max: Option<i64> = storage.get(&key).and_then(|s| s.parse::<i64>().ok());

    let prs = fetch_open_prs(repo, token)?;

    let current_max = prs.iter().map(|p| p.number).max();

    // First run: record max PR number, no notification
    let stored_max = match stored_max {
        Some(m) => m,
        None => {
            if let Some(max) = current_max {
                storage.set(&key, &max.to_string());
            }
            return None;
        }
    };

    let new_prs: Vec<&PRInfo> = prs.iter().filter(|p| p.number > stored_max).collect();

    if new_prs.is_empty() {
        return None;
    }

    if let Some(max) = current_max {
        storage.set(&key, &max.to_string());
    }

    let lines: Vec<String> = new_prs
        .iter()
        .map(|p| format!("  PR #{}: \"{}\" by {}", p.number, p.title, p.user.login))
        .collect();

    Some(format!(
        "[{}/{}] New PRs:\n{}",
        repo.owner,
        repo.repo,
        lines.join("\n")
    ))
}

fn fetch_open_prs(repo: &RepoConfig, token: Option<&str>) -> Option<Vec<PRInfo>> {
    let path = format!(
        "/repos/{}/{}/pulls?state=open&per_page=100&sort=created&direction=desc",
        repo.owner, repo.repo
    );
    let json = api_get(&path, token).ok()?;
    serde_json::from_value(json).ok()
}

// ── releases ─────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct ReleaseInfo {
    tag_name: String,
    name: String,
}

fn check_releases(repo: &RepoConfig, storage: &mut Storage, token: Option<&str>) -> Option<String> {
    let key = format!("github_releases_seen_{}_{}", repo.owner, repo.repo);
    let seen: HashSet<String> = storage
        .get(&key)
        .map(|s| {
            s.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default();

    let releases = fetch_releases(repo, token)?;

    let new_releases: Vec<&ReleaseInfo> = releases
        .iter()
        .filter(|r| !seen.contains(&r.tag_name))
        .collect();

    // First run: record seen tags, no notification
    if seen.is_empty() {
        let tags: Vec<String> = releases.iter().map(|r| r.tag_name.clone()).collect();
        storage.set(&key, &tags.join(","));
        return None;
    }

    if new_releases.is_empty() {
        return None;
    }

    let mut updated_seen: HashSet<String> = seen;
    let lines: Vec<String> = new_releases
        .iter()
        .map(|r| {
            updated_seen.insert(r.tag_name.clone());
            let name = if r.name.is_empty() || r.name == r.tag_name {
                String::new()
            } else {
                format!(" - \"{}\"", r.name)
            };
            format!("  Release {}{}", r.tag_name, name)
        })
        .collect();

    storage.set(
        &key,
        &updated_seen.into_iter().collect::<Vec<_>>().join(","),
    );

    Some(format!(
        "[{}/{}] New releases:\n{}",
        repo.owner,
        repo.repo,
        lines.join("\n")
    ))
}

fn fetch_releases(repo: &RepoConfig, token: Option<&str>) -> Option<Vec<ReleaseInfo>> {
    let path = format!(
        "/repos/{}/{}/releases?per_page=10&sort=created&direction=desc",
        repo.owner, repo.repo
    );
    let json = api_get(&path, token).ok()?;
    serde_json::from_value(json).ok()
}
