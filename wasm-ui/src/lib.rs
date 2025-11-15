// Copyright (c) 2025 Michael A Wright
// SPDX-License-Identifier: MIT

#[cfg(target_arch = "wasm32")]
use yew::prelude::*;

#[cfg(target_arch = "wasm32")]
#[derive(Clone, PartialEq)]
struct Repository {
    id: String,
    owner: String,
    name: String,
    language: String,
    last_push: String,
    branches: Vec<BranchInfo>,
    unmerged_count: u32,
    pr_count: u32,
}

#[cfg(target_arch = "wasm32")]
#[derive(Clone, PartialEq)]
struct BranchInfo {
    name: String,
    status: String,
    ahead: u32,
    behind: u32,
}

#[cfg(target_arch = "wasm32")]
#[derive(Clone, PartialEq)]
struct RepoGroup {
    name: String,
    repos: Vec<Repository>,
}

#[cfg(target_arch = "wasm32")]
#[derive(Clone, PartialEq)]
struct BuildInfo {
    version: String,
    build_date: String,
    build_host: String,
    git_commit_short: String,
    git_commit: String,
}

#[cfg(target_arch = "wasm32")]
#[function_component(App)]
fn app() -> Html {
    let groups = use_state(get_mock_groups);
    let active_tab = use_state(|| 0usize);
    let selected_repo = use_state(|| None::<Repository>);
    let build_info = use_state(|| BuildInfo {
        version: "0.1.0".to_string(),
        build_date: "Loading...".to_string(),
        build_host: "Unknown".to_string(),
        git_commit_short: "dev".to_string(),
        git_commit: "development".to_string(),
    });

    // Load build info on mount
    {
        let build_info = build_info.clone();
        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(info) = fetch_build_info().await {
                    build_info.set(info);
                }
            });
            || ()
        });
    }

    let on_tab_click = {
        let active_tab = active_tab.clone();
        Callback::from(move |idx: usize| {
            active_tab.set(idx);
        })
    };

    let on_repo_click = {
        let selected_repo = selected_repo.clone();
        Callback::from(move |repo: Repository| {
            selected_repo.set(Some(repo));
        })
    };

    let on_close_modal = {
        let selected_repo = selected_repo.clone();
        Callback::from(move |_| {
            selected_repo.set(None);
        })
    };

    html! {
        <>
            <div class="app-container">
                <header class="compact-header">
                    <h1>{ "Overall" }</h1>
                    <span class="tagline">{ "Repository Manager" }</span>
                </header>

                <nav class="tabs">
                    { for groups.iter().enumerate().map(|(idx, group)| {
                        let onclick = {
                            let on_tab_click = on_tab_click.clone();
                            Callback::from(move |_| on_tab_click.emit(idx))
                        };
                        html! {
                            <button
                                class={classes!("tab", (*active_tab == idx).then_some("active"))}
                                {onclick}
                            >
                                { &group.name }
                                <span class="repo-count">{ group.repos.len() }</span>
                            </button>
                        }
                    })}
                    <button class="tab tab-add" title="Add new group">{ "+" }</button>
                </nav>

                <main class="repo-list">
                    { if let Some(group) = groups.get(*active_tab) {
                        html! {
                            <>
                                { for group.repos.iter().map(|repo| {
                                    let onclick = {
                                        let on_repo_click = on_repo_click.clone();
                                        let repo = repo.clone();
                                        Callback::from(move |_| on_repo_click.emit(repo.clone()))
                                    };
                                    html! {
                                        <RepoRow repo={repo.clone()} {onclick} />
                                    }
                                })}
                            </>
                        }
                    } else {
                        html! { <div class="empty-state">{ "No repositories in this group" }</div> }
                    }}
                </main>

                <footer class="app-footer">
                    <div class="footer-left">
                        <span>{ "Copyright © 2025 Michael A Wright" }</span>
                        <span class="separator">{ "·" }</span>
                        <a href="https://github.com/softwarewrighter/overall/blob/main/LICENSE" target="_blank">{ "MIT License" }</a>
                        <span class="separator">{ "·" }</span>
                        <a href="https://github.com/softwarewrighter/overall" target="_blank">{ "GitHub Repository" }</a>
                    </div>
                    <div class="footer-right build-info">
                        <span title={format!("Full commit: {}", build_info.git_commit)}>
                            { format!("{}", build_info.git_commit_short) }
                        </span>
                        <span class="separator">{ "·" }</span>
                        <span>{ format!("{}", build_info.build_host) }</span>
                        <span class="separator">{ "·" }</span>
                        <span>{ &build_info.build_date }</span>
                    </div>
                </footer>
            </div>

            { if let Some(repo) = (*selected_repo).clone() {
                html! { <RepoDetailModal repo={repo} on_close={on_close_modal} /> }
            } else {
                html! {}
            }}
        </>
    }
}

#[cfg(target_arch = "wasm32")]
#[derive(Properties, PartialEq)]
struct RepoRowProps {
    repo: Repository,
    onclick: Callback<()>,
}

#[cfg(target_arch = "wasm32")]
#[function_component(RepoRow)]
fn repo_row(props: &RepoRowProps) -> Html {
    let repo = &props.repo;
    let onclick = {
        let onclick = props.onclick.clone();
        Callback::from(move |_| onclick.emit(()))
    };

    html! {
        <div class="repo-row" {onclick}>
            <div class="repo-info">
                <span class="repo-name">{ &repo.id }</span>
                <span class="repo-meta">
                    <span class="language-badge">{ &repo.language }</span>
                    <span class="last-push">{ &repo.last_push }</span>
                </span>
            </div>
            <div class="repo-status">
                { if repo.unmerged_count > 0 {
                    html! {
                        <span class="status-indicator warning" title="Unmerged branches">
                            <span class="icon">{ "⚠" }</span>
                            <span class="count">{ repo.unmerged_count }</span>
                        </span>
                    }
                } else {
                    html! {}
                }}
                { if repo.pr_count > 0 {
                    html! {
                        <span class="status-indicator info" title="Pending pull requests">
                            <span class="icon">{ "📋" }</span>
                            <span class="count">{ repo.pr_count }</span>
                        </span>
                    }
                } else {
                    html! {}
                }}
                { if repo.unmerged_count == 0 && repo.pr_count == 0 {
                    html! {
                        <span class="status-indicator success" title="No pending work">
                            <span class="icon">{ "✓" }</span>
                        </span>
                    }
                } else {
                    html! {}
                }}
            </div>
        </div>
    }
}

#[cfg(target_arch = "wasm32")]
#[derive(Properties, PartialEq)]
struct RepoDetailModalProps {
    repo: Repository,
    on_close: Callback<()>,
}

#[cfg(target_arch = "wasm32")]
#[function_component(RepoDetailModal)]
fn repo_detail_modal(props: &RepoDetailModalProps) -> Html {
    let repo = &props.repo;

    let on_backdrop_click = {
        let on_close = props.on_close.clone();
        Callback::from(move |_| on_close.emit(()))
    };

    let on_modal_click = Callback::from(|e: MouseEvent| {
        e.stop_propagation();
    });

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

    let on_close_button_click = {
        let on_close = props.on_close.clone();
        Callback::from(move |_| on_close.emit(()))
    };

    html! {
        <div class="modal-backdrop" onclick={on_backdrop_click}>
            <div class="modal-content" onclick={on_modal_click}>
                <div class="modal-header">
                    <h2>{ &repo.id }</h2>
                    <button class="close-button" onclick={on_close_button_click}>{ "✕" }</button>
                </div>

                <div class="modal-body">
                    <div class="repo-detail-meta">
                        <span class="language-badge">{ &repo.language }</span>
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

                    <h3>{ "Branches" }</h3>
                    <div class="branches-detail">
                        { for repo.branches.iter().map(|branch| html! {
                            <div class={classes!("branch-detail", branch.status.to_lowercase())}>
                                <div class="branch-header">
                                    <span class="branch-name">{ &branch.name }</span>
                                    <span class="branch-status-badge">{ &branch.status }</span>
                                </div>
                                if branch.ahead > 0 || branch.behind > 0 {
                                    <div class="branch-stats">
                                        { if branch.ahead > 0 {
                                            html! { <span class="ahead">{ format!("+{} ahead", branch.ahead) }</span> }
                                        } else {
                                            html! {}
                                        }}
                                        { if branch.behind > 0 {
                                            html! { <span class="behind">{ format!("-{} behind", branch.behind) }</span> }
                                        } else {
                                            html! {}
                                        }}
                                    </div>
                                }
                            </div>
                        })}
                    </div>
                </div>
            </div>
        </div>
    }
}

#[cfg(target_arch = "wasm32")]
fn get_mock_groups() -> Vec<RepoGroup> {
    vec![
        RepoGroup {
            name: "Active Projects".to_string(),
            repos: vec![
                Repository {
                    id: "softwarewrighter/overall".to_string(),
                    owner: "softwarewrighter".to_string(),
                    name: "overall".to_string(),
                    language: "Rust".to_string(),
                    last_push: "2 hours ago".to_string(),
                    unmerged_count: 2,
                    pr_count: 1,
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
                    unmerged_count: 0,
                    pr_count: 2,
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
                    unmerged_count: 2,
                    pr_count: 0,
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
            ],
        },
        RepoGroup {
            name: "Utilities".to_string(),
            repos: vec![
                Repository {
                    id: "softwarewrighter/markdown-checker".to_string(),
                    owner: "softwarewrighter".to_string(),
                    name: "markdown-checker".to_string(),
                    language: "Rust".to_string(),
                    last_push: "2 days ago".to_string(),
                    unmerged_count: 0,
                    pr_count: 0,
                    branches: vec![BranchInfo {
                        name: "main".to_string(),
                        status: "InReview".to_string(),
                        ahead: 0,
                        behind: 0,
                    }],
                },
                Repository {
                    id: "softwarewrighter/dotfiles".to_string(),
                    owner: "softwarewrighter".to_string(),
                    name: "dotfiles".to_string(),
                    language: "Shell".to_string(),
                    last_push: "1 week ago".to_string(),
                    unmerged_count: 0,
                    pr_count: 0,
                    branches: vec![BranchInfo {
                        name: "main".to_string(),
                        status: "InReview".to_string(),
                        ahead: 0,
                        behind: 0,
                    }],
                },
            ],
        },
        RepoGroup {
            name: "Experiments".to_string(),
            repos: vec![
                Repository {
                    id: "softwarewrighter/test-repo".to_string(),
                    owner: "softwarewrighter".to_string(),
                    name: "test-repo".to_string(),
                    language: "Python".to_string(),
                    last_push: "3 weeks ago".to_string(),
                    unmerged_count: 1,
                    pr_count: 0,
                    branches: vec![
                        BranchInfo {
                            name: "main".to_string(),
                            status: "InReview".to_string(),
                            ahead: 0,
                            behind: 0,
                        },
                        BranchInfo {
                            name: "experimental".to_string(),
                            status: "ReadyForPR".to_string(),
                            ahead: 3,
                            behind: 0,
                        },
                    ],
                },
            ],
        },
    ]
}

#[cfg(target_arch = "wasm32")]
async fn fetch_build_info() -> Result<BuildInfo, String> {
    use gloo::net::http::Request;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct BuildInfoJson {
        version: String,
        build_date: String,
        build_host: String,
        git_commit: String,
        git_commit_short: String,
    }

    let response = Request::get("/build-info.json")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch build info: {:?}", e))?;

    let info: BuildInfoJson = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse build info: {:?}", e))?;

    Ok(BuildInfo {
        version: info.version,
        build_date: info.build_date,
        build_host: info.build_host,
        git_commit: info.git_commit,
        git_commit_short: info.git_commit_short,
    })
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn run_app() {
    yew::Renderer::<App>::new().render();
}
