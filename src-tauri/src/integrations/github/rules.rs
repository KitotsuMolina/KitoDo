use super::models::ExternalCandidate;

pub fn build_task_title(item: &ExternalCandidate) -> String {
    let kind = match item.kind.as_str() {
        "pr" => "PR",
        "issue" => "Issue",
        "notification" => "Notif",
        _ => "Item",
    };

    match item.number {
        Some(number) => format!("{kind} #{number} ({}): {}", item.repo_full, item.title),
        None => format!("{kind} ({}) : {}", item.repo_full, item.title),
    }
}

pub fn infer_priority(item: &ExternalCandidate) -> i64 {
    let title = item.title.to_lowercase();
    if title.contains("urgent") || title.contains("security") || title.contains("hotfix") {
        return 1;
    }

    if item.kind == "notification" || item.is_review_requested {
        return 2;
    }

    3
}

pub fn system_labels(item: &ExternalCandidate) -> Vec<String> {
    let mut labels = vec!["github".to_string(), item.kind.clone()];

    if item.is_review_requested {
        labels.push("review".to_string());
    }

    let repo_tag = item
        .repo_full
        .replace('/', "-")
        .replace('_', "-")
        .to_lowercase();
    labels.push(format!("repo-{repo_tag}"));

    labels
}
