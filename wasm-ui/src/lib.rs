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
    pull_requests: Vec<PullRequestInfo>,
    unmerged_count: u32,
    pr_count: u32,
}

#[cfg(target_arch = "wasm32")]
#[derive(Clone, PartialEq)]
struct BranchInfo {
    name: String,
    sha: String,
    status: String,
    ahead: u32,
    behind: u32,
    last_commit_date: String,
    commits: Vec<CommitInfo>,
}

#[cfg(target_arch = "wasm32")]
#[derive(Clone, PartialEq)]
struct CommitInfo {
    sha: String,
    message: String,
    author_name: String,
    author_email: String,
    authored_date: String,
}

#[cfg(target_arch = "wasm32")]
#[derive(Clone, PartialEq)]
struct PullRequestInfo {
    number: u32,
    title: String,
    state: String,
    created_at: String,
    updated_at: String,
}

#[cfg(target_arch = "wasm32")]
#[derive(Clone, PartialEq)]
struct RepoGroup {
    id: Option<i64>, // None for ungrouped
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
#[derive(Clone, PartialEq)]
struct LocalRepoRoot {
    id: i64,
    path: String,
    enabled: bool,
    created_at: String,
}

#[cfg(target_arch = "wasm32")]
#[derive(Clone, PartialEq)]
struct LocalRepoStatus {
    id: i64,
    repo_id: String,
    local_path: String,
    current_branch: Option<String>,
    uncommitted_files: u32,
    unpushed_commits: u32,
    behind_commits: u32,
    is_dirty: bool,
    last_checked: String,
}

#[cfg(target_arch = "wasm32")]
#[derive(Clone, PartialEq, Copy)]
enum SortColumn {
    Name,
    Language,
    LastUpdated,
    Status,
}

#[cfg(target_arch = "wasm32")]
#[derive(Clone, PartialEq)]
struct SortState {
    column: SortColumn,
    ascending: bool,
}

#[cfg(target_arch = "wasm32")]
#[function_component(App)]
fn app() -> Html {
    let groups = use_state(|| Vec::<RepoGroup>::new());
    let active_tab = use_state(|| 0usize);
    let selected_repo = use_state(|| None::<Repository>);
    let show_add_dialog = use_state(|| false);
    let show_settings = use_state(|| false);
    let dragged_repo_id = use_state(|| None::<String>);
    let loading_repo = use_state(|| None::<String>);
    let refreshing = use_state(|| false);
    let last_refresh = use_state(|| None::<f64>);
    let local_repo_statuses =
        use_state(|| std::collections::HashMap::<String, LocalRepoStatus>::new());
    let sort_state = use_state(|| SortState {
        column: SortColumn::Status,
        ascending: false,
    });
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

    // Load local repo statuses on mount
    {
        let local_repo_statuses = local_repo_statuses.clone();
        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                web_sys::console::log_1(&"[App] Fetching local repo statuses...".into());
                match fetch_local_repo_statuses().await {
                    Ok(statuses) => {
                        web_sys::console::log_1(
                            &format!("[App] Loaded {} local repo statuses", statuses.len()).into(),
                        );

                        // Debug log sw-install specifically
                        if let Some(sw_install_status) =
                            statuses.iter().find(|s| s.repo_id.contains("sw-install"))
                        {
                            web_sys::console::log_1(
                                &format!(
                                    "[App] sw-install status: uncommitted={}, is_dirty={}",
                                    sw_install_status.uncommitted_files, sw_install_status.is_dirty
                                )
                                .into(),
                            );
                        } else {
                            web_sys::console::log_1(
                                &"[App] sw-install NOT found in fetched statuses!".into(),
                            );
                        }

                        let status_map: std::collections::HashMap<String, LocalRepoStatus> =
                            statuses
                                .into_iter()
                                .map(|s| (s.repo_id.clone(), s))
                                .collect();

                        // Debug the HashMap key for sw-install
                        if status_map.contains_key("softwarewrighter/sw-install") {
                            web_sys::console::log_1(
                                &"[App] HashMap contains 'softwarewrighter/sw-install' key".into(),
                            );
                        } else {
                            web_sys::console::log_1(&"[App] HashMap does NOT contain 'softwarewrighter/sw-install' key!".into());
                        }

                        local_repo_statuses.set(status_map);
                    }
                    Err(e) => {
                        web_sys::console::error_1(
                            &format!("[App] ERROR fetching local repo statuses: {}", e).into(),
                        );
                    }
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
        let loading_repo = loading_repo.clone();
        let groups = groups.clone();
        let local_repo_statuses = local_repo_statuses.clone();
        Callback::from(move |repo: Repository| {
            let repo_id = repo.id.clone();

            // Show spinner overlay immediately
            loading_repo.set(Some(repo_id.clone()));

            // Clone for async block
            let selected_repo = selected_repo.clone();
            let loading_repo = loading_repo.clone();
            let groups = groups.clone();
            let local_repo_statuses = local_repo_statuses.clone();

            wasm_bindgen_futures::spawn_local(async move {
                // Step 1: Sync this specific repo from GitHub
                if let Err(e) = sync_single_repo(&repo_id).await {
                    web_sys::console::error_1(
                        &format!("Failed to sync repo {}: {}", repo_id, e).into(),
                    );
                    loading_repo.set(None);
                    return;
                }

                // Step 2: Fetch fresh /repos.json
                let fresh_groups = match fetch_repos().await {
                    Ok(g) => g,
                    Err(e) => {
                        web_sys::console::error_1(&format!("Failed to fetch repos: {}", e).into());
                        loading_repo.set(None);
                        return;
                    }
                };

                // Step 3: Fetch fresh local repo statuses
                if let Ok(statuses) = fetch_local_repo_statuses().await {
                    let status_map: std::collections::HashMap<String, LocalRepoStatus> = statuses
                        .into_iter()
                        .map(|s| (s.repo_id.clone(), s))
                        .collect();
                    local_repo_statuses.set(status_map);
                }

                // Step 4: Find the fresh repo data
                let fresh_repo = fresh_groups
                    .iter()
                    .flat_map(|g| &g.repos)
                    .find(|r| r.id == repo_id)
                    .cloned();

                // Step 5: Update state
                groups.set(fresh_groups);

                // Step 6: Hide spinner and show dialog
                loading_repo.set(None);
                if let Some(fresh_repo) = fresh_repo {
                    selected_repo.set(Some(fresh_repo));
                }
            });
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

    let on_open_settings = {
        let show_settings = show_settings.clone();
        Callback::from(move |_| {
            show_settings.set(true);
        })
    };

    let on_close_settings = {
        let show_settings = show_settings.clone();
        Callback::from(move |_| {
            show_settings.set(false);
        })
    };

    let on_refresh = {
        let groups = groups.clone();
        let local_repo_statuses = local_repo_statuses.clone();
        let refreshing = refreshing.clone();
        let last_refresh = last_refresh.clone();
        Callback::from(move |_| {
            // Set refreshing state
            refreshing.set(true);

            let groups = groups.clone();
            let local_repo_statuses = local_repo_statuses.clone();
            let refreshing = refreshing.clone();
            let last_refresh = last_refresh.clone();
            wasm_bindgen_futures::spawn_local(async move {
                // Trigger the scan
                if let Err(e) = trigger_local_repo_scan().await {
                    web_sys::console::error_1(&format!("Failed to trigger scan: {}", e).into());
                    refreshing.set(false);
                    return;
                }

                // Wait a bit for scan to complete
                gloo::timers::future::sleep(std::time::Duration::from_millis(1000)).await;

                // Reload repo data
                if let Ok(loaded_groups) = fetch_repos().await {
                    groups.set(loaded_groups);
                }

                // Reload local repo statuses
                if let Ok(statuses) = fetch_local_repo_statuses().await {
                    let status_map: std::collections::HashMap<String, LocalRepoStatus> = statuses
                        .into_iter()
                        .map(|s| (s.repo_id.clone(), s))
                        .collect();
                    local_repo_statuses.set(status_map);
                }

                // Update last refresh timestamp
                last_refresh.set(Some(js_sys::Date::now()));

                // Clear refreshing state
                refreshing.set(false);
            });
        })
    };

    let on_drag_start = {
        let dragged_repo_id = dragged_repo_id.clone();
        Callback::from(move |repo_id: String| {
            dragged_repo_id.set(Some(repo_id));
        })
    };

    let on_drop_to_group = {
        let dragged_repo_id = dragged_repo_id.clone();
        let groups_state = groups.clone();
        Callback::from(move |(_group_idx, target_group_id): (usize, Option<i64>)| {
            if let Some(repo_id) = (*dragged_repo_id).clone() {
                let groups_state = groups_state.clone();
                let dragged_repo_id = dragged_repo_id.clone();

                wasm_bindgen_futures::spawn_local(async move {
                    if move_repo_to_group(&repo_id, target_group_id).await.is_ok() {
                        // Reload data after successful move
                        if let Ok(loaded_groups) = fetch_repos().await {
                            groups_state.set(loaded_groups);
                        }
                    }
                    dragged_repo_id.set(None);
                });
            }
        })
    };

    let on_sort_column_click = {
        let sort_state = sort_state.clone();
        Callback::from(move |column: SortColumn| {
            let current = (*sort_state).clone();
            if current.column == column {
                // Toggle direction if clicking same column
                sort_state.set(SortState {
                    column,
                    ascending: !current.ascending,
                });
            } else {
                // New column - default to descending for Status, ascending for others
                let ascending = match column {
                    SortColumn::Status => false,
                    _ => true,
                };
                sort_state.set(SortState { column, ascending });
            }
        })
    };

    html! {
        <>
            <div class="app-container">
                <header class="compact-header">
                    <div class="header-left">
                        <h1>{ "Overall" }</h1>
                        <span class="tagline">{ "Repository Manager" }</span>
                    </div>
                    <div class="header-right">
                        { if let Some(timestamp) = *last_refresh {
                            html! {
                                <span class="last-refresh" title={format!("Last refreshed: {}", js_sys::Date::new(&timestamp.into()).to_iso_string())}>
                                    { format!("Last: {}", format_refresh_time(timestamp)) }
                                </span>
                            }
                        } else {
                            html! {}
                        }}
                        <button
                            class={if *refreshing { "btn-refresh refreshing" } else { "btn-refresh" }}
                            onclick={on_refresh}
                            disabled={*refreshing}
                            title="Refresh local repository status"
                        >
                            { "🔄" }
                        </button>
                        <button class="btn-settings" onclick={on_open_settings} title="Local Repository Settings">
                            { "⚙️" }
                        </button>
                    </div>
                </header>

                <nav class="tabs">
                    { for groups.iter().enumerate().map(|(idx, group)| {
                        let onclick = {
                            let on_tab_click = on_tab_click.clone();
                            Callback::from(move |_| on_tab_click.emit(idx))
                        };

                        let ondragover = {
                            Callback::from(move |e: DragEvent| {
                                e.prevent_default(); // Allow drop
                            })
                        };

                        let ondrop = {
                            let on_drop_to_group = on_drop_to_group.clone();
                            let group_id = group.id;
                            Callback::from(move |e: DragEvent| {
                                e.prevent_default();
                                on_drop_to_group.emit((idx, group_id));
                            })
                        };

                        // Calculate worst-case status priority across all repos in tab
                        // TRAFFIC LIGHT PRIORITY: 0=RED (STOP), 1=YELLOW (YIELD), 2=WHITE (cleanup), 3=GREEN (GO)
                        // CRITICAL: Use the SAME calculation as individual repos, then take minimum (worst)
                        let worst_priority = group.repos.iter()
                            .map(|repo| calculate_repo_status_priority(repo, local_repo_statuses.get(&repo.id)))
                            .min()
                            .unwrap_or(3); // Default to complete if no repos

                        // Use PNG icons from static/icons/ - show only worst-case status
                        let (status_icon, tab_class) = match worst_priority {
                            0 => (html! { <img class="tab-status-icon" src="/icons/needs-sync.png" alt="Needs sync" title="🛑 STOP: Has uncommitted, unpushed, or unfetched commits" /> }, "tab-needs-sync"),
                            1 => (html! { <img class="tab-status-icon" src="/icons/local-changes.png" alt="Local changes" title="⚠️ YIELD: Has local uncommitted changes" /> }, "tab-local-changes"),
                            2 => (html! { <img class="tab-status-icon" src="/icons/stale.png" alt="Stale" title="ℹ️ CLEAN UP: Has unmerged feature branches" /> }, "tab-stale"),
                            3 if !group.repos.is_empty() => (html! { <img class="tab-status-icon" src="/icons/complete.png" alt="Complete" title="✅ PROCEED: All repositories up to date" /> }, "tab-complete"),
                            _ => (html! {}, ""),
                        };

                        html! {
                            <button
                                class={classes!("tab", tab_class, (*active_tab == idx).then_some("active"))}
                                {onclick}
                                {ondragover}
                                {ondrop}
                            >
                                { &group.name }
                                <span class="repo-count">{ group.repos.len() }</span>
                                { status_icon }
                            </button>
                        }
                    })}
                    <button class="tab tab-add" title="Add repositories to groups" onclick={on_open_add_dialog.clone()}>{ "+" }</button>
                </nav>

                <main class="repo-list">
                    { if let Some(group) = groups.get(*active_tab) {
                        // Clone repos and sort them
                        let mut sorted_repos = group.repos.clone();
                        sort_repositories(&mut sorted_repos, &*sort_state, &*local_repo_statuses);

                        html! {
                            <>
                                <RepoListHeader
                                    sort_state={(*sort_state).clone()}
                                    on_column_click={on_sort_column_click.clone()}
                                />
                                { for sorted_repos.iter().map(|repo| {
                                    let onclick = {
                                        let on_repo_click = on_repo_click.clone();
                                        let repo = repo.clone();
                                        Callback::from(move |_| on_repo_click.emit(repo.clone()))
                                    };
                                    let on_drag_start = on_drag_start.clone();
                                    let local_status = local_repo_statuses.get(&repo.id).cloned();

                                    // Debug log for sw-install
                                    if repo.id.contains("sw-install") {
                                        if local_status.is_some() {
                                            web_sys::console::log_1(&format!("[App] Looked up local_status for '{}' - FOUND", repo.id).into());
                                        } else {
                                            web_sys::console::log_1(&format!("[App] Looked up local_status for '{}' - NOT FOUND", repo.id).into());
                                        }
                                    }

                                    html! {
                                        <RepoRow repo={repo.clone()} {onclick} {on_drag_start} {local_status} />
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
                html! { <RepoDetailModal repo={repo} groups={(*groups).clone()} on_close={on_close_modal} /> }
            } else {
                html! {}
            }}

            { if *show_add_dialog {
                html! { <AddRepoDialog groups={(*groups).clone()} on_close={on_close_add_dialog} /> }
            } else {
                html! {}
            }}

            { if *show_settings {
                html! { <SettingsDialog on_close={on_close_settings} /> }
            } else {
                html! {}
            }}

            { if let Some(repo_id) = (*loading_repo).clone() {
                html! {
                    <div class="spinner-overlay">
                        <div class="spinner-container">
                            <div class="spinner-large">{ "⟳" }</div>
                            <div class="spinner-text">{ format!("Refreshing {}...", repo_id) }</div>
                        </div>
                    </div>
                }
            } else {
                html! {}
            }}
        </>
    }
}

#[cfg(target_arch = "wasm32")]
#[derive(Properties, PartialEq)]
struct RepoListHeaderProps {
    sort_state: SortState,
    on_column_click: Callback<SortColumn>,
}

#[cfg(target_arch = "wasm32")]
#[function_component(RepoListHeader)]
fn repo_list_header(props: &RepoListHeaderProps) -> Html {
    let on_name_click = {
        let on_column_click = props.on_column_click.clone();
        Callback::from(move |_| on_column_click.emit(SortColumn::Name))
    };

    let on_language_click = {
        let on_column_click = props.on_column_click.clone();
        Callback::from(move |_| on_column_click.emit(SortColumn::Language))
    };

    let on_last_updated_click = {
        let on_column_click = props.on_column_click.clone();
        Callback::from(move |_| on_column_click.emit(SortColumn::LastUpdated))
    };

    let on_status_click = {
        let on_column_click = props.on_column_click.clone();
        Callback::from(move |_| on_column_click.emit(SortColumn::Status))
    };

    let sort_indicator = |column: SortColumn| -> &'static str {
        if props.sort_state.column == column {
            if props.sort_state.ascending {
                " ▲"
            } else {
                " ▼"
            }
        } else {
            ""
        }
    };

    html! {
        <div class="repo-list-header">
            <div class="header-column col-name" onclick={on_name_click}>
                { "Name" }{ sort_indicator(SortColumn::Name) }
            </div>
            <div class="header-column col-language" onclick={on_language_click}>
                { "Language" }{ sort_indicator(SortColumn::Language) }
            </div>
            <div class="header-column col-last-updated" onclick={on_last_updated_click}>
                { "Last Updated" }{ sort_indicator(SortColumn::LastUpdated) }
            </div>
            <div class="header-column col-status" onclick={on_status_click}>
                { "Status" }{ sort_indicator(SortColumn::Status) }
            </div>
        </div>
    }
}

#[cfg(target_arch = "wasm32")]
#[derive(Properties, PartialEq)]
struct RepoRowProps {
    repo: Repository,
    onclick: Callback<()>,
    on_drag_start: Callback<String>,
    local_status: Option<LocalRepoStatus>,
}

#[cfg(target_arch = "wasm32")]
#[function_component(RepoRow)]
fn repo_row(props: &RepoRowProps) -> Html {
    let repo = &props.repo;

    // Debug logging for sw-install
    if repo.id.contains("sw-install") {
        if let Some(ref status) = props.local_status {
            web_sys::console::log_1(
                &format!(
                    "[RepoRow] sw-install status: uncommitted={}, unpushed={}, is_dirty={}",
                    status.uncommitted_files, status.unpushed_commits, status.is_dirty
                )
                .into(),
            );
        } else {
            web_sys::console::log_1(&"[RepoRow] sw-install has NO local_status!".into());
        }
    }

    let onclick = {
        let onclick = props.onclick.clone();
        Callback::from(move |_| onclick.emit(()))
    };

    let ondragstart = {
        let repo_id = repo.id.clone();
        let on_drag_start = props.on_drag_start.clone();
        Callback::from(move |_e: DragEvent| {
            on_drag_start.emit(repo_id.clone());
            // Note: DataTransfer is handled by browser, we just track the repo_id in state
        })
    };

    html! {
        <div class="repo-row" draggable="true" {ondragstart} {onclick}>
            <div class="col-name">
                <span class="repo-name">{ &repo.id }</span>
            </div>
            <div class="col-language">
                <span class="language-badge">{ &repo.language }</span>
            </div>
            <div class="col-last-updated">
                <span class="last-push">{ &repo.last_push }</span>
            </div>
            <div class="col-status repo-status">
                { if let Some(status) = &props.local_status {
                    // Priority: local-changes (yellow) FIRST - commit before push!
                    // Then: needs-sync (red) for unpushed/behind
                    if status.uncommitted_files > 0 {
                        html! {
                            <span class="status-indicator local-changes" title={format!("{} uncommitted files", status.uncommitted_files)}>
                                <img class="status-icon" src="/icons/local-changes.png" alt="Local changes" />
                                <span class="count">{ status.uncommitted_files }</span>
                            </span>
                        }
                    } else if status.unpushed_commits > 0 || status.behind_commits > 0 {
                        html! {
                            <span class="status-indicator needs-sync" title={format!("{} unpushed commits", status.unpushed_commits)}>
                                <img class="status-icon" src="/icons/needs-sync.png" alt="Needs sync" />
                                <span class="count">{ status.unpushed_commits }</span>
                            </span>
                        }
                    } else {
                        html! {}
                    }
                } else {
                    html! {}
                }}
                // Check GitHub branch status (ahead/behind on remote) - only if no local status shown
                { if props.local_status.as_ref().map_or(true, |s| s.uncommitted_files == 0 && s.unpushed_commits == 0 && s.behind_commits == 0) {
                    // Check if any branch is ahead or behind on GitHub
                    let branches_needing_sync = repo.branches.iter()
                        .filter(|b| b.ahead > 0 || b.behind > 0)
                        .count();

                    if branches_needing_sync > 0 {
                        html! {
                            <span class="status-indicator needs-sync" title={format!("{} branches need sync", branches_needing_sync)}>
                                <img class="status-icon" src="/icons/needs-sync.png" alt="Needs sync" />
                                <span class="count">{ branches_needing_sync }</span>
                            </span>
                        }
                    } else {
                        html! {}
                    }
                } else {
                    html! {}
                }}
                { if repo.unmerged_count > 0 {
                    html! {
                        <span class="status-indicator stale" title="Unmerged branches">
                            <img class="status-icon" src="/icons/stale.png" alt="Stale" />
                            <span class="count">{ repo.unmerged_count }</span>
                        </span>
                    }
                } else {
                    html! {}
                }}
                { if repo.pr_count > 0 {
                    html! {
                        <span class="status-indicator info" title="Pending pull requests">
                            <img class="status-icon" src="/icons/stale.png" alt="Pull requests" />
                            <span class="count">{ repo.pr_count }</span>
                        </span>
                    }
                } else {
                    html! {}
                }}
                { if repo.unmerged_count == 0 && repo.pr_count == 0 && props.local_status.as_ref().map_or(true, |s| !s.is_dirty) {
                    html! {
                        <span class="status-indicator success" title="No pending work">
                            <img class="status-icon" src="/icons/complete.png" alt="Complete" />
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
    groups: Vec<RepoGroup>,
    on_close: Callback<()>,
}

#[cfg(target_arch = "wasm32")]
#[function_component(RepoDetailModal)]
fn repo_detail_modal(props: &RepoDetailModalProps) -> Html {
    let repo = &props.repo;

    // Find current group for this repo
    let current_group_id = props
        .groups
        .iter()
        .find(|g| g.repos.iter().any(|r| r.id == repo.id))
        .and_then(|g| g.id);

    let on_backdrop_click = {
        let on_close = props.on_close.clone();
        Callback::from(move |_| on_close.emit(()))
    };

    let on_modal_click = Callback::from(|e: MouseEvent| {
        e.stop_propagation();
    });

    let on_group_change = {
        let repo_id = repo.id.clone();
        Callback::from(move |e: Event| {
            let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
            let new_group_id = if select.value() == "ungrouped" {
                None
            } else {
                Some(select.value().parse::<i64>().unwrap())
            };

            let repo_id = repo_id.clone();
            wasm_bindgen_futures::spawn_local(async move {
                if let Err(e) = move_repo_to_group(&repo_id, new_group_id).await {
                    web_sys::console::error_1(&format!("Failed to move repo: {}", e).into());
                } else {
                    // Reload page to reflect changes
                    web_sys::window().unwrap().location().reload().ok();
                }
            });
        })
    };

    // Exclude main/master/develop branches - they should never have PRs created
    let ready_for_pr = repo
        .branches
        .iter()
        .filter(|b| {
            b.status == "ReadyForPR"
                && b.name != "main"
                && b.name != "master"
                && b.name != "develop"
        })
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

                    <div class="group-selector">
                        <label for="group-select">{ "Group: " }</label>
                        <select id="group-select" onchange={on_group_change}>
                            <option value="ungrouped" selected={current_group_id.is_none()}>
                                { "Ungrouped" }
                            </option>
                            { for props.groups.iter().map(|group| {
                                let is_selected = Some(group.id.unwrap_or(0)) == current_group_id;
                                html! {
                                    <option value={group.id.unwrap_or(0).to_string()} selected={is_selected}>
                                        { &group.name }
                                    </option>
                                }
                            })}
                        </select>
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

                        // Add "Create All PRs" button if there are branches ready for PR
                        if ready_for_pr > 0 {
                            {{
                                let repo_id_for_all_prs = repo.id.clone();
                                let on_create_all_prs = {
                                    Callback::from(move |_| {
                                        let repo_id = repo_id_for_all_prs.clone();

                                        wasm_bindgen_futures::spawn_local(async move {
                                            match create_all_pull_requests(&repo_id).await {
                                                Ok(message) => {
                                                    web_sys::console::log_1(&format!("Success: {}", message).into());
                                                }
                                                Err(e) => {
                                                    web_sys::console::error_1(&format!("Failed to create PRs: {}", e).into());
                                                }
                                            }
                                        });
                                    })
                                };

                                html! {
                                    <button onclick={on_create_all_prs} class="btn-create-all-prs" title="Create Pull Requests for all branches with unmerged work">
                                        { "Create All PRs" }
                                    </button>
                                }
                            }}
                        }
                    </div>

                    <h3>{ format!("Branches ({})", repo.branches.len()) }</h3>
                    <div class="branches-detail">
                        { for repo.branches.iter().map(|branch| {
                            let has_unmerged_work = branch.ahead > 0; // Show button if branch has commits ahead
                            let needs_sync = branch.behind > 0;
                            let repo_full_name = repo.id.clone();
                            let branch_name = branch.name.clone();

                            html! {
                            <div class={classes!("branch-detail", branch.status.to_lowercase())}>
                                <div class="branch-header">
                                    <div class="branch-info">
                                        <span class="branch-name">{ &branch.name }</span>
                                        <span class="branch-status-badge">{ &branch.status }</span>
                                    </div>
                                    <div class="branch-actions">
                                        {{
                                            let repo_id_for_pr = repo_full_name.clone();
                                            let branch_name_for_pr = branch_name.clone();

                                            let on_create_pr = {
                                                Callback::from(move |_| {
                                                    let repo_id = repo_id_for_pr.clone();
                                                    let branch_name = branch_name_for_pr.clone();

                                                    wasm_bindgen_futures::spawn_local(async move {
                                                        if let Err(e) = create_pull_request(&repo_id, &branch_name).await {
                                                            web_sys::console::error_1(&format!("Failed to create PR: {}", e).into());
                                                        }
                                                    });
                                                })
                                            };

                                            html! {
                                                <>
                                                { if has_unmerged_work {
                                                    html! {
                                                        <button onclick={on_create_pr} class="btn-create-pr" title="Create Pull Request">
                                                            { "Create PR" }
                                                        </button>
                                                    }
                                                } else {
                                                    html! {}
                                                }}
                                                </>
                                            }
                                        }}
                                    </div>
                                </div>
                                <div class="branch-meta">
                                    <div class="branch-commit-info">
                                        <span class="commit-sha" title={branch.sha.clone()}>
                                            { if branch.sha.len() > 7 { &branch.sha[..7] } else { &branch.sha } }
                                        </span>
                                        <span class="commit-timestamp">{ &branch.last_commit_date }</span>
                                    </div>
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
                                        { if needs_sync {
                                            html! { <span class="sync-warning">{ "⚠️ Needs sync with main" }</span> }
                                        } else {
                                            html! {}
                                        }}
                                    </div>
                                </div>
                                { if !branch.commits.is_empty() {
                                    html! {
                                        <div class="commits-list">
                                            <h4>{ format!("Commits ({})", branch.commits.len()) }</h4>
                                            { for branch.commits.iter().map(|commit| {
                                                let short_sha = if commit.sha.len() > 7 { &commit.sha[..7] } else { &commit.sha };
                                                let first_line = commit.message.lines().next().unwrap_or(&commit.message);
                                                html! {
                                                    <div class="commit-item">
                                                        <div class="commit-header">
                                                            <span class="commit-sha" title={commit.sha.clone()}>{ short_sha }</span>
                                                            <span class="commit-author">{ &commit.author_name }</span>
                                                            <span class="commit-date">{ &commit.authored_date }</span>
                                                        </div>
                                                        <div class="commit-message">{ first_line }</div>
                                                    </div>
                                                }
                                            })}
                                        </div>
                                    }
                                } else {
                                    html! {}
                                }}
                            </div>
                            }
                        })}
                    </div>

                    <h3>{ format!("Pull Requests ({})", repo.pull_requests.len()) }</h3>
                    <div class="pull-requests-detail">
                        { if repo.pull_requests.is_empty() {
                            html! {
                                <div class="no-prs">{ "No open pull requests" }</div>
                            }
                        } else {
                            html! {
                                { for repo.pull_requests.iter().map(|pr| {
                                    let repo_full_name = repo.id.clone();
                                    let pr_number = pr.number;

                                    html! {
                                        <div class={classes!("pr-detail", pr.state.to_lowercase())}>
                                            <div class="pr-header">
                                                <div class="pr-info">
                                                    <span class="pr-number">{ format!("#{}", pr.number) }</span>
                                                    <span class="pr-title">{ &pr.title }</span>
                                                    <span class={classes!("pr-state-badge", pr.state.to_lowercase())}>
                                                        { &pr.state }
                                                    </span>
                                                </div>
                                                <div class="pr-actions">
                                                    {{
                                                        let pr_url = format!("https://github.com/{}/pull/{}", repo_full_name, pr_number);
                                                        html! {
                                                            <a href={pr_url} target="_blank" class="btn-view-pr" title="View on GitHub">
                                                                { "View PR" }
                                                            </a>
                                                        }
                                                    }}
                                                </div>
                                            </div>
                                            <div class="pr-meta">
                                                <span class="pr-created">{ format!("Created: {}", &pr.created_at) }</span>
                                                <span class="pr-updated">{ format!("Updated: {}", &pr.updated_at) }</span>
                                            </div>
                                        </div>
                                    }
                                })}
                            }
                        }}
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

    let on_add_repos = {
        let selected_repos = selected_repos.clone();
        let target_group = target_group.clone();
        let new_group_name = new_group_name.clone();
        let create_new_group = create_new_group.clone();
        let all_groups = props.groups.clone();
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

            // Find the group ID if using existing group
            let target_group_id: Option<i64> = if !*create_new_group {
                all_groups
                    .iter()
                    .find(|g| g.name == group_name)
                    .map(|g| g.id)
            } else {
                None
            }
            .flatten();

            let repo_ids: Vec<String> = (*selected_repos).iter().cloned().collect();
            let on_close = on_close.clone();

            // Call the API
            wasm_bindgen_futures::spawn_local(async move {
                use gloo::net::http::Request;
                use serde::Serialize;

                #[derive(Serialize)]
                #[serde(rename_all = "camelCase")]
                struct AddReposRequest {
                    group_name: String,
                    repo_ids: Vec<String>,
                    target_group_id: Option<i64>,
                }

                let request_body = AddReposRequest {
                    group_name,
                    repo_ids,
                    target_group_id,
                };

                match Request::post("/api/groups/add-repos")
                    .header("Content-Type", "application/json")
                    .json(&request_body)
                {
                    Ok(req) => match req.send().await {
                        Ok(_) => {
                            // Close the modal
                            on_close.emit(());
                            // Reload the page to show updated groups
                            web_sys::window().unwrap().location().reload().ok();
                        }
                        Err(e) => {
                            web_sys::window()
                                .unwrap()
                                .alert_with_message(&format!("Failed to add repositories: {:?}", e))
                                .unwrap();
                        }
                    },
                    Err(e) => {
                        web_sys::window()
                            .unwrap()
                            .alert_with_message(&format!("Failed to create request: {:?}", e))
                            .unwrap();
                    }
                }
            });
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
                            onclick={on_add_repos}
                            disabled={selected_repos.is_empty()}
                        >
                            { format!("Add to Group ({} selected)", selected_repos.len()) }
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
#[derive(Properties, PartialEq)]
struct SettingsDialogProps {
    on_close: Callback<()>,
}

#[cfg(target_arch = "wasm32")]
#[function_component(SettingsDialog)]
fn settings_dialog(props: &SettingsDialogProps) -> Html {
    let local_repo_roots = use_state(|| Vec::<LocalRepoRoot>::new());
    let new_path = use_state(|| String::new());

    // Load local repo roots on mount
    {
        let local_repo_roots = local_repo_roots.clone();
        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                web_sys::console::log_1(&"[SettingsDialog] Fetching local repo roots...".into());
                match fetch_local_repo_roots().await {
                    Ok(roots) => {
                        web_sys::console::log_1(
                            &format!("[SettingsDialog] Fetched {} roots", roots.len()).into(),
                        );
                        local_repo_roots.set(roots);
                    }
                    Err(e) => {
                        web_sys::console::error_1(
                            &format!("[SettingsDialog] Error fetching roots: {}", e).into(),
                        );
                    }
                }
            });
            || ()
        });
    }

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

    let on_path_input = {
        let new_path = new_path.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            new_path.set(input.value());
        })
    };

    let on_add_path = {
        let new_path = new_path.clone();
        let local_repo_roots = local_repo_roots.clone();
        Callback::from(move |_| {
            let path = (*new_path).clone();
            web_sys::console::log_1(
                &format!("[SettingsDialog] Add path clicked: '{}'", path).into(),
            );
            if path.trim().is_empty() {
                web_sys::console::log_1(&"[SettingsDialog] Path is empty, ignoring".into());
                return;
            }
            let new_path = new_path.clone();
            let local_repo_roots = local_repo_roots.clone();
            wasm_bindgen_futures::spawn_local(async move {
                web_sys::console::log_1(&format!("[SettingsDialog] Adding path: {}", path).into());
                match add_local_repo_root(&path).await {
                    Ok(()) => {
                        web_sys::console::log_1(&"[SettingsDialog] Path added successfully".into());
                        new_path.set(String::new());
                        web_sys::console::log_1(
                            &"[SettingsDialog] Refreshing roots list...".into(),
                        );
                        if let Ok(roots) = fetch_local_repo_roots().await {
                            web_sys::console::log_1(
                                &format!("[SettingsDialog] Updated list has {} roots", roots.len())
                                    .into(),
                            );
                            local_repo_roots.set(roots);
                        }
                    }
                    Err(e) => {
                        web_sys::console::error_1(
                            &format!("[SettingsDialog] Error adding path: {}", e).into(),
                        );
                        // Show error to user
                        if let Some(window) = web_sys::window() {
                            let _ = window.alert_with_message(&format!("Error: {}", e));
                        }
                    }
                }
            });
        })
    };

    let on_scan = {
        Callback::from(move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                let _ = scan_local_repos().await;
                web_sys::window().and_then(|w| w.location().reload().ok());
            });
        })
    };

    html! {
        <div class="modal-backdrop" onclick={on_backdrop_click}>
            <div class="modal-content settings-modal" onclick={on_modal_click}>
                <div class="modal-header">
                    <h2>{ "Local Repository Settings" }</h2>
                    <button class="close-button" onclick={on_close_button_click}>{ "✕" }</button>
                </div>

                <div class="modal-body">
                    <div class="settings-section">
                        <h3>{ "Local Repository Root Paths" }</h3>
                        <p class="settings-description">
                            { "Configure directories to scan for local Git repositories. The tool will detect uncommitted changes and sync status." }
                        </p>

                        <div class="add-path-section">
                            <input
                                type="text"
                                class="path-input"
                                placeholder="e.g., ~/github/softwarewrighter"
                                value={(*new_path).clone()}
                                oninput={on_path_input}
                            />
                            <button class="btn btn-primary" onclick={on_add_path}>
                                { "Add Path" }
                            </button>
                        </div>

                        <div class="repo-roots-list">
                            { if local_repo_roots.is_empty() {
                                html! {
                                    <p class="empty-message">{ "No local repository roots configured" }</p>
                                }
                            } else {
                                html! {
                                    <>
                                        { for local_repo_roots.iter().map(|root| {
                                            let root_id = root.id;
                                            let local_repo_roots = local_repo_roots.clone();
                                            let on_delete = Callback::from(move |_| {
                                                let local_repo_roots = local_repo_roots.clone();
                                                wasm_bindgen_futures::spawn_local(async move {
                                                    web_sys::console::log_1(&format!("[SettingsDialog] Deleting root {}", root_id).into());
                                                    match remove_local_repo_root(root_id).await {
                                                        Ok(()) => {
                                                            web_sys::console::log_1(&"[SettingsDialog] Root deleted successfully".into());
                                                            if let Ok(roots) = fetch_local_repo_roots().await {
                                                                web_sys::console::log_1(&format!("[SettingsDialog] Updated list has {} roots", roots.len()).into());
                                                                local_repo_roots.set(roots);
                                                            }
                                                        }
                                                        Err(e) => {
                                                            web_sys::console::error_1(&format!("[SettingsDialog] Error deleting root: {}", e).into());
                                                            if let Some(window) = web_sys::window() {
                                                                let _ = window.alert_with_message(&format!("Error: {}", e));
                                                            }
                                                        }
                                                    }
                                                });
                                            });
                                            html! {
                                                <div class="repo-root-item">
                                                    <span class="root-path">{ &root.path }</span>
                                                    <span class="root-status">
                                                        { if root.enabled { "✓ Enabled" } else { "Disabled" } }
                                                    </span>
                                                    <button class="btn btn-danger btn-sm" onclick={on_delete}>{ "✕" }</button>
                                                </div>
                                            }
                                        })}
                                    </>
                                }
                            }}
                        </div>

                        <div class="scan-section">
                            <button class="btn btn-secondary" onclick={on_scan}>
                                { "Scan Now" }
                            </button>
                            <p class="scan-help">{ "This will scan all enabled root paths for repositories and update their status." }</p>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[cfg(target_arch = "wasm32")]
#[cfg(test)]
fn get_mock_groups() -> Vec<RepoGroup> {
    vec![
        RepoGroup {
            id: Some(1),
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
                    pull_requests: vec![],
                    branches: vec![
                        BranchInfo {
                            name: "main".to_string(),
                            sha: "a1b2c3d4e5f6".to_string(),
                            status: "InReview".to_string(),
                            ahead: 0,
                            behind: 0,
                            last_commit_date: "2 hours ago".to_string(),
                            commits: vec![],
                        },
                        BranchInfo {
                            name: "feature/yew-ui".to_string(),
                            sha: "f6e5d4c3b2a1".to_string(),
                            status: "ReadyForPR".to_string(),
                            ahead: 15,
                            behind: 0,
                            last_commit_date: "3 hours ago".to_string(),
                            commits: vec![],
                        },
                        BranchInfo {
                            name: "feature/ai-analysis".to_string(),
                            sha: "9876543210ab".to_string(),
                            status: "ReadyForPR".to_string(),
                            ahead: 8,
                            behind: 0,
                            last_commit_date: "5 hours ago".to_string(),
                            commits: vec![],
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
                    pull_requests: vec![],
                    branches: vec![
                        BranchInfo {
                            name: "main".to_string(),
                            sha: "abc123def456".to_string(),
                            status: "InReview".to_string(),
                            ahead: 0,
                            behind: 0,
                            last_commit_date: "5 hours ago".to_string(),
                            commits: vec![],
                        },
                        BranchInfo {
                            name: "fix/docs-update".to_string(),
                            sha: "789fedcba012".to_string(),
                            status: "InReview".to_string(),
                            ahead: 2,
                            behind: 0,
                            last_commit_date: "6 hours ago".to_string(),
                            commits: vec![],
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
                    pull_requests: vec![],
                    branches: vec![
                        BranchInfo {
                            name: "main".to_string(),
                            sha: "deadbeef1234".to_string(),
                            status: "InReview".to_string(),
                            ahead: 0,
                            behind: 0,
                            last_commit_date: "1 day ago".to_string(),
                            commits: vec![],
                        },
                        BranchInfo {
                            name: "feature/streaming".to_string(),
                            sha: "cafebabe5678".to_string(),
                            status: "NeedsUpdate".to_string(),
                            ahead: 5,
                            behind: 3,
                            last_commit_date: "2 days ago".to_string(),
                            commits: vec![],
                        },
                        BranchInfo {
                            name: "refactor/error-handling".to_string(),
                            sha: "1a2b3c4d5e6f".to_string(),
                            status: "ReadyForPR".to_string(),
                            ahead: 12,
                            behind: 0,
                            last_commit_date: "1 day ago".to_string(),
                            commits: vec![],
                        },
                    ],
                },
            ],
        },
        RepoGroup {
            id: Some(2),
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
                    pull_requests: vec![],
                    branches: vec![BranchInfo {
                        name: "main".to_string(),
                        sha: "fedcba987654".to_string(),
                        status: "InReview".to_string(),
                        ahead: 0,
                        behind: 0,
                        last_commit_date: "2 days ago".to_string(),
                        commits: vec![],
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
                    pull_requests: vec![],
                    branches: vec![BranchInfo {
                        name: "main".to_string(),
                        sha: "0123456789ab".to_string(),
                        status: "InReview".to_string(),
                        ahead: 0,
                        behind: 0,
                        last_commit_date: "1 week ago".to_string(),
                        commits: vec![],
                    }],
                },
            ],
        },
        RepoGroup {
            id: Some(3),
            name: "Experiments".to_string(),
            repos: vec![Repository {
                id: "softwarewrighter/test-repo".to_string(),
                owner: "softwarewrighter".to_string(),
                name: "test-repo".to_string(),
                language: "Python".to_string(),
                last_push: "3 weeks ago".to_string(),
                unmerged_count: 1,
                pr_count: 0,
                pull_requests: vec![],
                branches: vec![
                    BranchInfo {
                        name: "main".to_string(),
                        sha: "abcdef123456".to_string(),
                        status: "InReview".to_string(),
                        ahead: 0,
                        behind: 0,
                        last_commit_date: "3 weeks ago".to_string(),
                        commits: vec![],
                    },
                    BranchInfo {
                        name: "experimental".to_string(),
                        sha: "fedcba654321".to_string(),
                        status: "ReadyForPR".to_string(),
                        ahead: 3,
                        behind: 0,
                        last_commit_date: "4 weeks ago".to_string(),
                        commits: vec![],
                    },
                ],
            }],
        },
    ]
}

#[cfg(target_arch = "wasm32")]
fn format_refresh_time(timestamp_ms: f64) -> String {
    let now = js_sys::Date::now();
    let seconds = ((now - timestamp_ms) / 1000.0) as i64;

    if seconds < 10 {
        "just now".to_string()
    } else if seconds < 60 {
        format!("{} seconds ago", seconds)
    } else if seconds < 3600 {
        let minutes = seconds / 60;
        if minutes == 1 {
            "1 minute ago".to_string()
        } else {
            format!("{} minutes ago", minutes)
        }
    } else if seconds < 86400 {
        let hours = seconds / 3600;
        if hours == 1 {
            "1 hour ago".to_string()
        } else {
            format!("{} hours ago", hours)
        }
    } else {
        let days = seconds / 86400;
        if days == 1 {
            "1 day ago".to_string()
        } else {
            format!("{} days ago", days)
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn calculate_repo_status_priority(repo: &Repository, local_status: Option<&LocalRepoStatus>) -> u8 {
    // TRAFFIC LIGHT PRIORITY (lower number = more urgent):
    // Priority 0 = RED (needs-sync)    - 🛑 STOP - Red stop sign / red ! - MOST URGENT
    // Priority 1 = YELLOW (local-changes) - ⚠️ YIELD - Yellow yield / yellow ? - 2nd urgent
    // Priority 2 = WHITE (stale)       - ℹ️ CLEAN UP - Innocuous cleanup - 3rd
    // Priority 3 = GREEN (complete)    - ✅ PROCEED - Green light, all clear - LEAST urgent

    // PRIORITY 0 (RED): Check for sync issues FIRST - MOST URGENT
    // Unpushed/behind locally OR branches ahead/behind on GitHub
    if let Some(status) = local_status {
        if status.unpushed_commits > 0 || status.behind_commits > 0 {
            return 0; // needs-sync (RED - STOP!)
        }
    }

    // CRITICAL: MUST also check GitHub branch status for ahead/behind
    // A repo can have clean working directory but still have branches that need sync!
    for branch in &repo.branches {
        if branch.ahead > 0 || branch.behind > 0 {
            return 0; // needs-sync (RED - STOP!)
        }
    }

    // PRIORITY 1 (YELLOW): Check local uncommitted files - 2nd URGENT
    // Rationale: Yield to commit/push this work before starting new work
    if let Some(status) = local_status {
        if status.uncommitted_files > 0 {
            return 1; // local-changes (YELLOW - YIELD!)
        }
    }

    // PRIORITY 2 (WHITE): Check for stale unmerged branches - cleanup
    // Innocuous but should be cleaned up (merged branches should be deleted)
    if repo.unmerged_count > 0 {
        return 2; // stale (WHITE - clean up when convenient)
    }

    // PRIORITY 3 (GREEN): All clear - ready to proceed
    3 // complete (GREEN - GO! Ready for next feature)
}

#[cfg(target_arch = "wasm32")]
fn sort_repositories(
    repos: &mut [Repository],
    sort_state: &SortState,
    local_statuses: &std::collections::HashMap<String, LocalRepoStatus>,
) {
    repos.sort_by(|a, b| {
        let cmp = match sort_state.column {
            SortColumn::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            SortColumn::Language => a.language.to_lowercase().cmp(&b.language.to_lowercase()),
            SortColumn::LastUpdated => a.last_push.cmp(&b.last_push),
            SortColumn::Status => {
                let a_priority = calculate_repo_status_priority(a, local_statuses.get(&a.id));
                let b_priority = calculate_repo_status_priority(b, local_statuses.get(&b.id));
                a_priority.cmp(&b_priority)
            }
        };

        if sort_state.ascending {
            cmp
        } else {
            cmp.reverse()
        }
    });
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
        pull_requests: Vec<PullRequestJson>,
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
        commits: Vec<CommitJson>,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct CommitJson {
        sha: String,
        message: String,
        author_name: String,
        author_email: String,
        authored_date: String,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct PullRequestJson {
        number: u32,
        title: String,
        state: String,
        created_at: String,
        updated_at: String,
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

    // Add timestamp to bypass browser caching
    let timestamp = js_sys::Date::now() as u64;
    let url = format!("/repos.json?ts={}", timestamp);
    let response = Request::get(&url)
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
        let repos: Vec<Repository> = group
            .repos
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
                        sha: b.sha.clone(),
                        status: b.status,
                        ahead: b.ahead_by,
                        behind: b.behind_by,
                        last_commit_date: format_relative_time(&b.last_commit_date),
                        commits: b
                            .commits
                            .into_iter()
                            .map(|c| CommitInfo {
                                sha: c.sha,
                                message: c.message,
                                author_name: c.author_name,
                                author_email: c.author_email,
                                authored_date: format_relative_time(&c.authored_date),
                            })
                            .collect(),
                    })
                    .collect(),
                pull_requests: r
                    .pull_requests
                    .into_iter()
                    .map(|pr| PullRequestInfo {
                        number: pr.number,
                        title: pr.title,
                        state: pr.state,
                        created_at: format_relative_time(&pr.created_at),
                        updated_at: format_relative_time(&pr.updated_at),
                    })
                    .collect(),
                unmerged_count: r.unmerged_count,
                pr_count: r.pr_count,
            })
            .collect();

        result.push(RepoGroup {
            id: Some(group.id),
            name: group.name,
            repos,
        });
    }

    // Add ungrouped repositories as a separate tab if any exist
    if !data.ungrouped.is_empty() {
        let ungrouped_repos: Vec<Repository> = data
            .ungrouped
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
                        sha: b.sha.clone(),
                        status: b.status,
                        ahead: b.ahead_by,
                        behind: b.behind_by,
                        last_commit_date: format_relative_time(&b.last_commit_date),
                        commits: b
                            .commits
                            .into_iter()
                            .map(|c| CommitInfo {
                                sha: c.sha,
                                message: c.message,
                                author_name: c.author_name,
                                author_email: c.author_email,
                                authored_date: format_relative_time(&c.authored_date),
                            })
                            .collect(),
                    })
                    .collect(),
                pull_requests: r
                    .pull_requests
                    .into_iter()
                    .map(|pr| PullRequestInfo {
                        number: pr.number,
                        title: pr.title,
                        state: pr.state,
                        created_at: format_relative_time(&pr.created_at),
                        updated_at: format_relative_time(&pr.updated_at),
                    })
                    .collect(),
                unmerged_count: r.unmerged_count,
                pr_count: r.pr_count,
            })
            .collect();

        result.push(RepoGroup {
            id: None, // Ungrouped has no ID
            name: "Ungrouped".to_string(),
            repos: ungrouped_repos,
        });
    }

    Ok(result)
}

#[cfg(target_arch = "wasm32")]
async fn move_repo_to_group(repo_id: &str, target_group_id: Option<i64>) -> Result<(), String> {
    use gloo::net::http::Request;
    use serde::Serialize;

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct MoveRequest {
        repo_id: String,
        target_group_id: Option<i64>,
    }

    let request_body = MoveRequest {
        repo_id: repo_id.to_string(),
        target_group_id,
    };

    Request::post("/api/repos/move")
        .header("Content-Type", "application/json")
        .json(&request_body)
        .map_err(|e| format!("Failed to serialize request: {:?}", e))?
        .send()
        .await
        .map_err(|e| format!("Failed to move repository: {:?}", e))?;

    Ok(())
}

#[cfg(target_arch = "wasm32")]
async fn create_pull_request(repo_id: &str, branch_name: &str) -> Result<String, String> {
    use gloo::net::http::Request;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct CreatePRRequest {
        repo_id: String,
        branch_name: String,
        title: Option<String>,
        body: Option<String>,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct CreatePRResponse {
        success: bool,
        pr_url: Option<String>,
        message: String,
    }

    let request_body = CreatePRRequest {
        repo_id: repo_id.to_string(),
        branch_name: branch_name.to_string(),
        title: None, // Let the backend generate from branch name
        body: None,  // Use default
    };

    let response = Request::post("/api/pr/create")
        .header("Content-Type", "application/json")
        .json(&request_body)
        .map_err(|e| format!("Failed to serialize request: {:?}", e))?
        .send()
        .await
        .map_err(|e| format!("Failed to create PR: {:?}", e))?;

    let result: CreatePRResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {:?}", e))?;

    if result.success {
        if let Some(pr_url) = result.pr_url {
            // Open PR in new tab
            if let Some(window) = web_sys::window() {
                let _ = window.open_with_url_and_target(&pr_url, "_blank");
            }
            Ok(pr_url)
        } else {
            Err("PR created but no URL returned".to_string())
        }
    } else {
        Err(result.message)
    }
}

#[cfg(target_arch = "wasm32")]
async fn create_all_pull_requests(repo_id: &str) -> Result<String, String> {
    use gloo::net::http::Request;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct CreateAllPRsRequest {
        repo_id: String,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct PRResult {
        branch_name: String,
        success: bool,
        pr_url: Option<String>,
        error: Option<String>,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct CreateAllPRsResponse {
        success: bool,
        results: Vec<PRResult>,
        message: String,
    }

    let request_body = CreateAllPRsRequest {
        repo_id: repo_id.to_string(),
    };

    let response = Request::post("/api/pr/create-all")
        .header("Content-Type", "application/json")
        .json(&request_body)
        .map_err(|e| format!("Failed to serialize request: {:?}", e))?
        .send()
        .await
        .map_err(|e| format!("Failed to create PRs: {:?}", e))?;

    let result: CreateAllPRsResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {:?}", e))?;

    if result.success {
        // Open all successful PRs in new tabs
        if let Some(window) = web_sys::window() {
            for pr_result in &result.results {
                if pr_result.success {
                    if let Some(pr_url) = &pr_result.pr_url {
                        let _ = window.open_with_url_and_target(pr_url, "_blank");
                    }
                } else if let Some(error) = &pr_result.error {
                    // Log errors for failed PR creations
                    web_sys::console::warn_1(
                        &format!(
                            "Failed to create PR for branch {}: {}",
                            pr_result.branch_name, error
                        )
                        .into(),
                    );
                }
            }
        }
        Ok(result.message)
    } else {
        Err(result.message)
    }
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
async fn trigger_local_repo_scan() -> Result<(), String> {
    use gloo::net::http::Request;

    let response = Request::post("/api/local-repos/scan")
        .header("Content-Type", "application/json")
        .body("{}")
        .map_err(|e| format!("Failed to create request: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Failed to trigger scan: {}", e))?;

    if !response.ok() {
        return Err(format!("Scan failed with status: {}", response.status()));
    }

    Ok(())
}

#[cfg(target_arch = "wasm32")]
async fn sync_single_repo(repo_id: &str) -> Result<(), String> {
    use gloo::net::http::Request;
    use serde::Serialize;

    #[derive(Serialize)]
    struct SyncRepoRequest {
        repo_id: String,
    }

    let request_body = SyncRepoRequest {
        repo_id: repo_id.to_string(),
    };

    let response = Request::post("/api/repos/sync")
        .header("Content-Type", "application/json")
        .json(&request_body)
        .map_err(|e| format!("Failed to create sync request: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Failed to sync repo: {}", e))?;

    if !response.ok() {
        return Err(format!("Sync failed with status: {}", response.status()));
    }

    Ok(())
}

#[cfg(target_arch = "wasm32")]
async fn fetch_local_repo_statuses() -> Result<Vec<LocalRepoStatus>, String> {
    use gloo::net::http::Request;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct LocalRepoStatusJson {
        id: i64,
        repo_id: String,
        local_path: String,
        current_branch: Option<String>,
        uncommitted_files: u32,
        unpushed_commits: u32,
        behind_commits: u32,
        is_dirty: bool,
        last_checked: String,
    }

    // Add timestamp to bypass browser caching
    let timestamp = js_sys::Date::now() as u64;
    let url = format!("/api/local-repos/status?ts={}", timestamp);
    let response = Request::get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch local repo statuses: {:?}", e))?;

    let statuses_json: Vec<LocalRepoStatusJson> = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse local repo statuses: {:?}", e))?;

    Ok(statuses_json
        .into_iter()
        .map(|s| LocalRepoStatus {
            id: s.id,
            repo_id: s.repo_id,
            local_path: s.local_path,
            current_branch: s.current_branch,
            uncommitted_files: s.uncommitted_files,
            unpushed_commits: s.unpushed_commits,
            behind_commits: s.behind_commits,
            is_dirty: s.is_dirty,
            last_checked: s.last_checked,
        })
        .collect())
}

#[cfg(target_arch = "wasm32")]
async fn fetch_local_repo_roots() -> Result<Vec<LocalRepoRoot>, String> {
    use gloo::net::http::Request;
    use serde::Deserialize;

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct LocalRepoRootJson {
        id: i64,
        path: String,
        enabled: bool,
        created_at: String,
    }

    let response = Request::get("/api/local-repos/roots")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch local repo roots: {:?}", e))?;

    let roots_json: Vec<LocalRepoRootJson> = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse local repo roots: {:?}", e))?;

    Ok(roots_json
        .into_iter()
        .map(|r| LocalRepoRoot {
            id: r.id,
            path: r.path,
            enabled: r.enabled,
            created_at: r.created_at,
        })
        .collect())
}

#[cfg(target_arch = "wasm32")]
async fn add_local_repo_root(path: &str) -> Result<(), String> {
    use gloo::net::http::Request;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct AddLocalRepoRootRequest {
        path: String,
    }

    #[derive(Deserialize)]
    struct AddLocalRepoRootResponse {
        success: bool,
        message: String,
    }

    let request_body = AddLocalRepoRootRequest {
        path: path.to_string(),
    };

    let response = Request::post("/api/local-repos/roots")
        .header("Content-Type", "application/json")
        .json(&request_body)
        .map_err(|e| format!("Failed to serialize request: {:?}", e))?
        .send()
        .await
        .map_err(|e| format!("Failed to add local repo root: {:?}", e))?;

    let result: AddLocalRepoRootResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {:?}", e))?;

    if result.success {
        Ok(())
    } else {
        Err(result.message)
    }
}

#[cfg(target_arch = "wasm32")]
async fn remove_local_repo_root(id: i64) -> Result<(), String> {
    use gloo::net::http::Request;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct RemoveLocalRepoRootResponse {
        success: bool,
        message: String,
    }

    let response = Request::delete(&format!("/api/local-repos/roots/{}", id))
        .send()
        .await
        .map_err(|e| format!("Failed to remove local repo root: {:?}", e))?;

    let result: RemoveLocalRepoRootResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {:?}", e))?;

    if result.success {
        Ok(())
    } else {
        Err(result.message)
    }
}

#[cfg(target_arch = "wasm32")]
async fn scan_local_repos() -> Result<String, String> {
    use gloo::net::http::Request;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct ScanResponse {
        success: bool,
        message: String,
    }

    let response = Request::post("/api/local-repos/scan")
        .header("Content-Type", "application/json")
        .body("{}")
        .map_err(|e| format!("Failed to create request: {:?}", e))?
        .send()
        .await
        .map_err(|e| format!("Failed to scan local repos: {:?}", e))?;

    let result: ScanResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {:?}", e))?;

    if result.success {
        Ok(result.message)
    } else {
        Err(result.message)
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn run_app() {
    yew::Renderer::<App>::new().render();
}
