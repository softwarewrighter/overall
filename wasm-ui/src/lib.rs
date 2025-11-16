// Copyright (c) 2025 Michael A Wright
// SPDX-License-Identifier: MIT

#[cfg(target_arch = "wasm32")]
use yew::prelude::*;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

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
    let groups = use_state(|| Vec::<RepoGroup>::new());
    let active_tab = use_state(|| 0usize);
    let selected_repo = use_state(|| None::<Repository>);
    let show_add_dialog = use_state(|| false);
    let build_info = use_state(|| BuildInfo {
        version: "0.1.0".to_string(),
        build_date: "Loading...".to_string(),
        build_host: "Unknown".to_string(),
        git_commit_short: "dev".to_string(),
        git_commit: "development".to_string(),
    });

    // Load repository data on mount
    {
        let groups = groups.clone();
        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(loaded_groups) = fetch_repos().await {
                    groups.set(loaded_groups);
                }
            });
            || ()
        });
    }

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

    let on_open_add_dialog = {
        let show_add_dialog = show_add_dialog.clone();
        Callback::from(move |_| {
            show_add_dialog.set(true);
        })
    };

    let on_close_add_dialog = {
        let show_add_dialog = show_add_dialog.clone();
        Callback::from(move |_| {
            show_add_dialog.set(false);
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
                    <button class="tab tab-add" title="Add repositories to groups" onclick={on_open_add_dialog.clone()}>{ "+" }</button>
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

            { if *show_add_dialog {
                html! { <AddRepoDialog groups={(*groups).clone()} on_close={on_close_add_dialog} /> }
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
#[derive(Properties, PartialEq)]
struct AddRepoDialogProps {
    groups: Vec<RepoGroup>,
    on_close: Callback<()>,
}

#[cfg(target_arch = "wasm32")]
#[function_component(AddRepoDialog)]
fn add_repo_dialog(props: &AddRepoDialogProps) -> Html {
    let selected_repos = use_state(|| std::collections::HashSet::<String>::new());
    let target_group = use_state(|| None::<String>);
    let new_group_name = use_state(|| String::new());
    let create_new_group = use_state(|| false);

    let on_backdrop_click = {
        let on_close = props.on_close.clone();
        Callback::from(move |_| on_close.emit(()))
    };

    let on_modal_click = Callback::from(|e: MouseEvent| {
        e.stop_propagation();
    });

    let on_close_button_click = {
        let on_close = props.on_close.clone();
        Callback::from(move |_| on_close.emit(()))
    };

    // Find ungrouped repos
    let ungrouped_repos = props
        .groups
        .iter()
        .find(|g| g.name == "Ungrouped")
        .map(|g| g.repos.clone())
        .unwrap_or_default();

    // Get existing group names (excluding Ungrouped)
    let existing_groups: Vec<String> = props
        .groups
        .iter()
        .filter(|g| g.name != "Ungrouped")
        .map(|g| g.name.clone())
        .collect();

    let on_repo_toggle = {
        let selected_repos = selected_repos.clone();
        Callback::from(move |repo_id: String| {
            let mut repos = (*selected_repos).clone();
            if repos.contains(&repo_id) {
                repos.remove(&repo_id);
            } else {
                repos.insert(repo_id);
            }
            selected_repos.set(repos);
        })
    };

    let on_group_change = {
        let target_group = target_group.clone();
        Callback::from(move |e: Event| {
            let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
            let value = select.value();
            if value.is_empty() {
                target_group.set(None);
            } else {
                target_group.set(Some(value));
            }
        })
    };

    let on_new_group_toggle = {
        let create_new_group = create_new_group.clone();
        let target_group = target_group.clone();
        Callback::from(move |_| {
            let new_value = !*create_new_group;
            create_new_group.set(new_value);
            if new_value {
                target_group.set(None);
            }
        })
    };

    let on_new_group_name_change = {
        let new_group_name = new_group_name.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            new_group_name.set(input.value());
        })
    };

    let on_generate_sql = {
        let selected_repos = selected_repos.clone();
        let target_group = target_group.clone();
        let new_group_name = new_group_name.clone();
        let create_new_group = create_new_group.clone();
        let on_close = props.on_close.clone();

        Callback::from(move |_| {
            if selected_repos.is_empty() {
                web_sys::window()
                    .unwrap()
                    .alert_with_message("Please select at least one repository")
                    .unwrap();
                return;
            }

            let group_name = if *create_new_group {
                if new_group_name.trim().is_empty() {
                    web_sys::window()
                        .unwrap()
                        .alert_with_message("Please enter a group name")
                        .unwrap();
                    return;
                }
                (*new_group_name).clone()
            } else {
                match (*target_group).clone() {
                    Some(name) => name,
                    None => {
                        web_sys::window()
                            .unwrap()
                            .alert_with_message("Please select a target group")
                            .unwrap();
                        return;
                    }
                }
            };

            // Generate SQL script
            let mut sql = String::new();

            if *create_new_group {
                sql.push_str("-- Create new group\n");
                sql.push_str(&format!(
                    "INSERT OR IGNORE INTO groups (name, display_order, created_at) \n\
                     SELECT '{}', COALESCE(MAX(display_order) + 1, 0), datetime('now', 'utc') || 'Z' \n\
                     FROM groups;\n\n",
                    group_name.replace('\'', "''")
                ));
            }

            sql.push_str(&format!("-- Add repositories to group '{}'\n", group_name));
            for repo_id in (*selected_repos).iter() {
                sql.push_str(&format!(
                    "INSERT OR IGNORE INTO repo_groups (repo_id, group_id, added_at)\n\
                     SELECT '{}', id, datetime('now', 'utc') || 'Z' FROM groups WHERE name = '{}';\n",
                    repo_id.replace('\'', "''"),
                    group_name.replace('\'', "''")
                ));
            }

            // Download the SQL file
            let blob_options = web_sys::BlobPropertyBag::new();
            blob_options.set_type("text/plain");
            let blob = web_sys::Blob::new_with_str_sequence_and_options(
                &js_sys::Array::of1(&wasm_bindgen::JsValue::from_str(&sql)),
                &blob_options,
            )
            .unwrap();

            let url = web_sys::Url::create_object_url_with_blob(&blob).unwrap();
            let document = web_sys::window().unwrap().document().unwrap();
            let a: web_sys::HtmlAnchorElement = document
                .create_element("a")
                .unwrap()
                .dyn_into()
                .unwrap();

            a.set_href(&url);
            a.set_download("add-repos.sql");
            a.click();

            web_sys::Url::revoke_object_url(&url).unwrap();

            on_close.emit(());
        })
    };

    html! {
        <div class="modal-backdrop" onclick={on_backdrop_click}>
            <div class="modal-content add-repo-modal" onclick={on_modal_click}>
                <div class="modal-header">
                    <h2>{ "Add Repositories to Group" }</h2>
                    <button class="close-button" onclick={on_close_button_click.clone()}>{ "✕" }</button>
                </div>

                <div class="modal-body">
                    <div class="add-repo-section">
                        <h3>{ format!("Select Repositories ({} ungrouped)", ungrouped_repos.len()) }</h3>
                        <div class="repo-selection-list">
                            { if ungrouped_repos.is_empty() {
                                html! { <p class="empty-message">{ "No ungrouped repositories available" }</p> }
                            } else {
                                html! {
                                    <>
                                        { for ungrouped_repos.iter().map(|repo| {
                                            let is_selected = selected_repos.contains(&repo.id);
                                            let repo_id = repo.id.clone();
                                            let onclick = {
                                                let on_repo_toggle = on_repo_toggle.clone();
                                                let repo_id = repo_id.clone();
                                                Callback::from(move |_| on_repo_toggle.emit(repo_id.clone()))
                                            };

                                            html! {
                                                <div class={classes!("repo-checkbox-item", is_selected.then_some("selected"))} {onclick}>
                                                    <input
                                                        type="checkbox"
                                                        checked={is_selected}
                                                        readonly=true
                                                    />
                                                    <div class="repo-checkbox-info">
                                                        <span class="repo-name">{ &repo.id }</span>
                                                        <span class="repo-meta-small">
                                                            <span class="language-badge-small">{ &repo.language }</span>
                                                            <span>{ &repo.last_push }</span>
                                                        </span>
                                                    </div>
                                                </div>
                                            }
                                        })}
                                    </>
                                }
                            }}
                        </div>
                    </div>

                    <div class="add-repo-section">
                        <h3>{ "Target Group" }</h3>

                        <div class="group-selection">
                            <label class="group-option">
                                <input
                                    type="radio"
                                    name="group-type"
                                    checked={!*create_new_group}
                                    onclick={on_new_group_toggle.clone()}
                                />
                                <span>{ "Existing Group:" }</span>
                            </label>

                            <select
                                class="group-select"
                                disabled={*create_new_group}
                                onchange={on_group_change}
                            >
                                <option value="" selected={target_group.is_none()}>
                                    { "-- Select a group --" }
                                </option>
                                { for existing_groups.iter().map(|group| {
                                    let is_selected = target_group.as_ref() == Some(group);
                                    html! {
                                        <option value={group.clone()} selected={is_selected}>
                                            { group }
                                        </option>
                                    }
                                })}
                            </select>

                            <label class="group-option">
                                <input
                                    type="radio"
                                    name="group-type"
                                    checked={*create_new_group}
                                    onclick={on_new_group_toggle.clone()}
                                />
                                <span>{ "New Group:" }</span>
                            </label>

                            <input
                                type="text"
                                class="group-name-input"
                                placeholder="Enter new group name"
                                disabled={!*create_new_group}
                                value={(*new_group_name).clone()}
                                oninput={on_new_group_name_change}
                            />
                        </div>
                    </div>

                    <div class="add-repo-actions">
                        <button
                            class="btn btn-primary"
                            onclick={on_generate_sql}
                            disabled={selected_repos.is_empty()}
                        >
                            { format!("Generate SQL ({} selected)", selected_repos.len()) }
                        </button>
                        <button class="btn btn-secondary" onclick={on_close_button_click.clone()}>
                            { "Cancel" }
                        </button>
                    </div>

                    <div class="add-repo-help">
                        <p>{ "This will generate a SQL script to add the selected repositories to the chosen group." }</p>
                        <p>{ "Run the script with: " }<code>{ "sqlite3 ~/.overall/overall.db < add-repos.sql" }</code></p>
                        <p>{ "Then re-export and reload: " }<code>{ "./target/release/overall export" }</code></p>
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
async fn fetch_repos() -> Result<Vec<RepoGroup>, String> {
    use gloo::net::http::Request;
    use serde::Deserialize;

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct RepoJson {
        id: String,
        owner: String,
        name: String,
        language: String,
        last_push: String,
        branches: Vec<BranchJson>,
        unmerged_count: u32,
        pr_count: u32,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct BranchJson {
        name: String,
        sha: String,
        ahead_by: u32,
        behind_by: u32,
        status: String,
        last_commit_date: String,
    }

    #[derive(Deserialize)]
    struct GroupJson {
        id: i64,
        name: String,
        repos: Vec<RepoJson>,
    }

    #[derive(Deserialize)]
    struct DataJson {
        groups: Vec<GroupJson>,
        ungrouped: Vec<RepoJson>,
    }

    let response = Request::get("/repos.json")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch repos: {:?}", e))?;

    let data: DataJson = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse repos: {:?}", e))?;

    let mut result = Vec::new();

    // Convert grouped repositories
    for group in data.groups {
        let repos: Vec<Repository> = group.repos
            .into_iter()
            .map(|r| Repository {
                id: r.id,
                owner: r.owner,
                name: r.name,
                language: r.language,
                last_push: format_relative_time(&r.last_push),
                branches: r
                    .branches
                    .into_iter()
                    .map(|b| BranchInfo {
                        name: b.name,
                        status: b.status,
                        ahead: b.ahead_by,
                        behind: b.behind_by,
                    })
                    .collect(),
                unmerged_count: r.unmerged_count,
                pr_count: r.pr_count,
            })
            .collect();

        result.push(RepoGroup {
            name: group.name,
            repos,
        });
    }

    // Add ungrouped repositories as a separate tab if any exist
    if !data.ungrouped.is_empty() {
        let ungrouped_repos: Vec<Repository> = data.ungrouped
            .into_iter()
            .map(|r| Repository {
                id: r.id,
                owner: r.owner,
                name: r.name,
                language: r.language,
                last_push: format_relative_time(&r.last_push),
                branches: r
                    .branches
                    .into_iter()
                    .map(|b| BranchInfo {
                        name: b.name,
                        status: b.status,
                        ahead: b.ahead_by,
                        behind: b.behind_by,
                    })
                    .collect(),
                unmerged_count: r.unmerged_count,
                pr_count: r.pr_count,
            })
            .collect();

        result.push(RepoGroup {
            name: "Ungrouped".to_string(),
            repos: ungrouped_repos,
        });
    }

    Ok(result)
}

#[cfg(target_arch = "wasm32")]
fn format_relative_time(iso_date: &str) -> String {
    // Simple relative time formatting
    // In production, use a proper date library
    use chrono::{DateTime, Utc};

    let parsed = iso_date.parse::<DateTime<Utc>>();
    if let Ok(date) = parsed {
        let now = Utc::now();
        let duration = now.signed_duration_since(date);

        if duration.num_hours() < 1 {
            let mins = duration.num_minutes();
            if mins < 1 {
                return "just now".to_string();
            }
            return format!("{} min{} ago", mins, if mins == 1 { "" } else { "s" });
        } else if duration.num_days() < 1 {
            let hours = duration.num_hours();
            return format!("{} hour{} ago", hours, if hours == 1 { "" } else { "s" });
        } else if duration.num_days() < 7 {
            let days = duration.num_days();
            return format!("{} day{} ago", days, if days == 1 { "" } else { "s" });
        } else if duration.num_weeks() < 4 {
            let weeks = duration.num_weeks();
            return format!("{} week{} ago", weeks, if weeks == 1 { "" } else { "s" });
        } else {
            let months = duration.num_days() / 30;
            if months < 12 {
                return format!("{} month{} ago", months, if months == 1 { "" } else { "s" });
            } else {
                let years = months / 12;
                return format!("{} year{} ago", years, if years == 1 { "" } else { "s" });
            }
        }
    }

    iso_date.to_string()
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
