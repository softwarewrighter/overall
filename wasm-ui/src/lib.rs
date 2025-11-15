use yew::prelude::*;

#[derive(Clone, PartialEq)]
struct Repository {
    id: String,
    owner: String,
    name: String,
    language: String,
    last_push: String,
    branches: Vec<BranchInfo>,
}

#[derive(Clone, PartialEq)]
struct BranchInfo {
    name: String,
    status: String,
    ahead: u32,
    behind: u32,
}

#[function_component(App)]
fn app() -> Html {
    let repos = use_state(get_mock_data);

    html! {
        <div class="container">
            <header>
                <h1>{ "GitHub Repository Manager" }</h1>
                <p class="subtitle">{ "Track branches ready for pull requests and merges" }</p>
            </header>

            <main>
                { for repos.iter().map(|repo| html! {
                    <RepositoryCard repo={repo.clone()} />
                })}
            </main>
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct RepositoryCardProps {
    repo: Repository,
}

#[function_component(RepositoryCard)]
fn repository_card(props: &RepositoryCardProps) -> Html {
    let repo = &props.repo;
    let ready_for_pr = repo
        .branches
        .iter()
        .filter(|b| b.status == "ReadyForPR")
        .count();
    let in_review = repo
        .branches
        .iter()
        .filter(|b| b.status == "InReview")
        .count();
    let needs_update = repo
        .branches
        .iter()
        .filter(|b| b.status == "NeedsUpdate")
        .count();

    html! {
        <div class="repo-card">
            <div class="repo-header">
                <h2>{ &repo.id }</h2>
                <span class="language">{ &repo.language }</span>
            </div>
            <div class="repo-meta">
                <span class="last-push">{ format!("Last push: {}", &repo.last_push) }</span>
            </div>

            <div class="branch-summary">
                if ready_for_pr > 0 {
                    <span class="badge ready">{ format!("{} Ready for PR", ready_for_pr) }</span>
                }
                if in_review > 0 {
                    <span class="badge review">{ format!("{} In Review", in_review) }</span>
                }
                if needs_update > 0 {
                    <span class="badge update">{ format!("{} Needs Update", needs_update) }</span>
                }
            </div>

            <div class="branches">
                { for repo.branches.iter().map(|branch| html! {
                    <div class={classes!("branch", branch.status.to_lowercase())}>
                        <span class="branch-name">{ &branch.name }</span>
                        <span class="branch-status">{ &branch.status }</span>
                        if branch.ahead > 0 || branch.behind > 0 {
                            <span class="branch-diff">
                                if branch.ahead > 0 {
                                    { format!("+{} ", branch.ahead) }
                                }
                                if branch.behind > 0 {
                                    { format!("-{}", branch.behind) }
                                }
                            </span>
                        }
                    </div>
                })}
            </div>
        </div>
    }
}

#[allow(dead_code)]
fn get_mock_data() -> Vec<Repository> {
    vec![
        Repository {
            id: "softwarewrighter/overall".to_string(),
            owner: "softwarewrighter".to_string(),
            name: "overall".to_string(),
            language: "Rust".to_string(),
            last_push: "2 hours ago".to_string(),
            branches: vec![
                BranchInfo {
                    name: "main".to_string(),
                    status: "InReview".to_string(),
                    ahead: 0,
                    behind: 0,
                },
                BranchInfo {
                    name: "feature/yew-ui".to_string(),
                    status: "ReadyForPR".to_string(),
                    ahead: 15,
                    behind: 0,
                },
                BranchInfo {
                    name: "feature/ai-analysis".to_string(),
                    status: "ReadyForPR".to_string(),
                    ahead: 8,
                    behind: 0,
                },
            ],
        },
        Repository {
            id: "softwarewrighter/proact".to_string(),
            owner: "softwarewrighter".to_string(),
            name: "proact".to_string(),
            language: "Rust".to_string(),
            last_push: "5 hours ago".to_string(),
            branches: vec![
                BranchInfo {
                    name: "main".to_string(),
                    status: "InReview".to_string(),
                    ahead: 0,
                    behind: 0,
                },
                BranchInfo {
                    name: "fix/docs-update".to_string(),
                    status: "InReview".to_string(),
                    ahead: 2,
                    behind: 0,
                },
            ],
        },
        Repository {
            id: "softwarewrighter/ask".to_string(),
            owner: "softwarewrighter".to_string(),
            name: "ask".to_string(),
            language: "Rust".to_string(),
            last_push: "1 day ago".to_string(),
            branches: vec![
                BranchInfo {
                    name: "main".to_string(),
                    status: "InReview".to_string(),
                    ahead: 0,
                    behind: 0,
                },
                BranchInfo {
                    name: "feature/streaming".to_string(),
                    status: "NeedsUpdate".to_string(),
                    ahead: 5,
                    behind: 3,
                },
                BranchInfo {
                    name: "refactor/error-handling".to_string(),
                    status: "ReadyForPR".to_string(),
                    ahead: 12,
                    behind: 0,
                },
            ],
        },
        Repository {
            id: "softwarewrighter/markdown-checker".to_string(),
            owner: "softwarewrighter".to_string(),
            name: "markdown-checker".to_string(),
            language: "Rust".to_string(),
            last_push: "2 days ago".to_string(),
            branches: vec![BranchInfo {
                name: "main".to_string(),
                status: "InReview".to_string(),
                ahead: 0,
                behind: 0,
            }],
        },
    ]
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn run_app() {
    yew::Renderer::<App>::new().render();
}
