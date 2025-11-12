use std::time::{Duration, SystemTime};

use crate::{
    config::save_language,
    model::{
        ActiveView, AppSettings, AppState, Language, RemoteTarget, SyncDirection, SyncSession,
        SyncStatus,
    },
};
use gpui::{
    div,
    prelude::FluentBuilder as _,
    Axis,
    Context,
    Div,
    Entity,
    IntoElement,
    ParentElement as _,
    Render,
    Styled as _,
    Window,
};
use gpui_component::{
    button::*,
    group_box::GroupBox,
    progress::Progress as ProgressBar,
    sidebar::{Sidebar, SidebarFooter, SidebarGroup, SidebarHeader, SidebarMenu, SidebarMenuItem},
    switch::Switch,
    tag::Tag,
    ActiveTheme,
    Disableable,
    Icon,
    IconName,
    Sizable as _,
    StyledExt,
};

pub struct AppView {
    state: Entity<AppState>,
}

impl AppView {
    pub fn new(state: Entity<AppState>) -> Self {
        Self { state }
    }
}

impl Render for AppView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.state.read(cx);
        let remote_targets = state.remote_targets.clone();
        let sessions = state.sessions.clone();
        let logs = state.logs.clone();
        let active_target_id = state.active_target;
        let active_view = state.active_view;
        let settings = state.settings.clone();
        let language = settings.language;

        let active_target = active_target_id
            .and_then(|id| remote_targets.iter().find(|target| target.id == id).cloned());

        let overview_handle = self.state.clone();
        let settings_handle = self.state.clone();
        let workspace_menu = SidebarMenu::new().children([
            SidebarMenuItem::new(tr(
                language,
                "Overview",
                "概览",
                "總覽",
            ))
                .icon(Icon::new(IconName::LayoutDashboard).small())
                .active(matches!(active_view, ActiveView::Dashboard))
                .on_click(move |_, _, cx| {
                    overview_handle.update(cx, |state, cx| {
                        state.active_view = ActiveView::Dashboard;
                        cx.notify();
                    });
                }),
            SidebarMenuItem::new(tr(
                language,
                "Settings",
                "设置",
                "設定",
            ))
                .icon(Icon::new(IconName::Settings).small())
                .active(matches!(active_view, ActiveView::Settings))
                .on_click(move |_, _, cx| {
                    settings_handle.update(cx, |state, cx| {
                        state.active_view = ActiveView::Settings;
                        cx.notify();
                    });
                }),
        ]);

        let sidebar_menu = SidebarMenu::new().children(remote_targets.iter().map(|target| {
            let target_id = target.id;
            let rule_count = target.rules.len();
            let pending = sessions
                .iter()
                .filter(|session| {
                    session.target_id == target_id
                        && matches!(
                            session.status,
                            SyncStatus::Planning | SyncStatus::AwaitingConfirmation
                        )
                })
                .count();
            let suffix_tag = if pending > 0 {
                Tag::warning()
                    .small()
                    .rounded_full()
                    .child(format!(
                        "{pending} {}",
                        tr(language, "pending", "待处理", "待處理")
                    ))
            } else {
                Tag::secondary()
                    .small()
                    .rounded_full()
                    .child(format!(
                        "{rule_count} {}",
                        tr(language, "rules", "规则", "規則")
                    ))
            };
            let handle = self.state.clone();

            SidebarMenuItem::new(target.name.clone())
                .icon(Icon::new(IconName::Globe).small())
                .suffix(suffix_tag)
                .active(active_view == ActiveView::Dashboard && active_target_id == Some(target_id))
                .on_click(move |_, _, cx| {
                    handle.update(cx, |state, cx| {
                        state.active_target = Some(target_id);
                        state.active_view = ActiveView::Dashboard;
                        cx.notify();
                    });
                })
        }));

        let sidebar = Sidebar::left()
            .header(
                SidebarHeader::new().child(
                    div()
                        .v_flex()
                        .gap_1()
                        .child(
                            div()
                                .text_lg()
                                .font_semibold()
                                .child(tr(language, "SFTP Sync", "SFTP 同步", "SFTP 同步")),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().sidebar_foreground.opacity(0.8))
                                .child(tr(
                                    language,
                                    "Manage remote mirrors",
                                    "管理远程镜像",
                                    "管理遠端鏡像",
                                )),
                        ),
                ),
            )
            .child(SidebarGroup::new(tr(language, "Workspace", "工作区", "工作區")).child(workspace_menu))
            .child(SidebarGroup::new(tr(language, "Targets", "目标", "目標")).child(sidebar_menu))
            .footer(
                SidebarFooter::new().child(
                    Button::new("add_target")
                        .ghost()
                        .small()
                        .icon(Icon::new(IconName::Plus).small())
                        .label(tr(language, "Add Target", "新增目标", "新增目標"))
                        .on_click(|_, _, _| println!("TODO: add new target form")),
                ),
            );

        let target_section = GroupBox::new()
            .title(tr(language, "Connection", "连接", "連線"))
            .fill()
            .child(match active_target {
                Some(target) => {
                    let rule_list = target.rules.iter().fold(
                        div().v_flex().gap_2(),
                        |builder, rule| {
                            builder.child(
                                div()
                                    .h_flex()
                                    .justify_between()
                                    .items_center()
                                    .gap_3()
                                    .p_3()
                                    .rounded(cx.theme().radius)
                                    .bg(cx.theme().muted.opacity(0.15))
                                    .child(
                                        div()
                                            .v_flex()
                                            .gap_1()
                                            .child(format!(
                                                "{} → {}",
                                                rule.local.display(),
                                                rule.remote.display()
                                            ))
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .text_color(
                                                        cx.theme().muted_foreground.opacity(0.9),
                                                    )
                                                    .child(tr(
                                                        language,
                                                        "Mapped path",
                                                        "映射路径",
                                                        "對應路徑",
                                                    )),
                                            ),
                                    )
                                    .child(
                                        Tag::info()
                                            .small()
                                            .rounded_full()
                                            .child(direction_label(rule.direction, language)),
                                    ),
                            )
                        },
                    );

                    div()
                        .v_flex()
                        .gap_4()
                        .child(
                            div()
                                .v_flex()
                                .gap_1()
                                .child(
                                    div()
                                        .h_flex()
                                        .gap_3()
                                        .items_center()
                                        .child(
                                            Icon::new(IconName::LayoutDashboard)
                                                .small()
                                                .text_color(cx.theme().primary),
                                        )
                                        .child(
                                            div()
                                                .text_xl()
                                                .font_semibold()
                                                .child(target.name.clone()),
                                        ),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(target.summary()),
                                ),
                        )
                        .child(
                            div()
                                .h_flex()
                                .gap_4()
                                .flex_wrap()
                                .child(
                                    div()
                                        .v_flex()
                                        .gap_1()
                                        .child(
                                            div()
                                                .text_sm()
                                                .text_color(cx.theme().muted_foreground)
                                                .child(tr(language, "Host", "主机", "主機")),
                                        )
                                        .child(div().font_medium().child(target.host.clone())),
                                )
                                .child(
                                    div()
                                        .v_flex()
                                        .gap_1()
                                        .child(
                                            div()
                                                .text_sm()
                                                .text_color(cx.theme().muted_foreground)
                                                .child(tr(
                                                    language,
                                                    "Base path",
                                                    "根路径",
                                                    "根路徑",
                                                )),
                                        )
                                        .child(
                                            div()
                                                .font_medium()
                                                .child(target.base_path.display().to_string()),
                                        ),
                                ),
                        )
                        .child(
                            div()
                                .v_flex()
                                .gap_2()
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(tr(language, "Sync rules", "同步规则", "同步規則")),
                                )
                                .child(rule_list),
                        )
                        .child(
                            div()
                                .h_flex()
                                .gap_3()
                                .child(
                                    Button::new("plan_sync")
                                        .primary()
                                        .label(tr(language, "Plan Dry Run", "生成试运行计划", "產生試運行計畫"))
                                        .icon(Icon::new(IconName::LayoutDashboard).small())
                                        .on_click(|_, _, _| println!("TODO: trigger dry run planning")),
                                )
                                .child(
                                    Button::new("sync_now")
                                        .success()
                                        .label(tr(language, "Execute Sync", "执行同步", "執行同步"))
                                        .icon(Icon::new(IconName::Check).small())
                                        .on_click(|_, _, _| println!("TODO: execute sync")),
                                ),
                        )
                }
                None => div()
                    .v_flex()
                    .gap_2()
                    .child(tr(
                        language,
                        "Select a target to begin planning a sync.",
                        "选择一个目标开始规划同步。",
                        "選擇一個目標開始規畫同步。",
                    ))
                    .child(
                        Button::new("create_target")
                            .primary()
                            .label(tr(language, "Create Target", "创建目标", "建立目標"))
                            .on_click(|_, _, _| println!("TODO: show create target modal")),
                    ),
            });

        let session_cards = if sessions.is_empty() {
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child(tr(
                    language,
                    "No sync sessions yet.",
                    "暂无同步任务。",
                    "尚無同步任務。",
                ))
        } else {
            sessions.iter().fold(div().v_flex().gap_3(), |builder, session| {
                builder.child(render_session_card(
                    session,
                    &remote_targets,
                    language,
                    cx,
                ))
            })
        };

        let session_section = GroupBox::new()
            .title(tr(language, "Sync Sessions", "同步任务", "同步任務"))
            .fill()
            .child(session_cards);

        let log_entries = if logs.is_empty() {
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child(tr(language, "No activity yet.", "暂无活动。", "尚無活動。"))
        } else {
            logs.iter()
                .rev()
                .take(6)
                .fold(div().v_flex().gap_2(), |builder, log| {
                    builder.child(
                        div()
                            .h_flex()
                            .justify_between()
                            .items_center()
                            .gap_3()
                            .child(
                                div()
                                    .h_flex()
                                    .gap_2()
                                    .items_center()
                                    .child(
                                        Icon::new(log_icon(&log.message))
                                            .small()
                                            .text_color(cx.theme().muted_foreground),
                                    )
                                    .child(log.message.clone()),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(format_timestamp(log.timestamp, language)),
                            ),
                    )
                })
        };

        let log_section = GroupBox::new()
            .title(tr(language, "Recent Activity", "最近活动", "最近活動"))
            .fill()
            .child(log_entries);

        let dashboard_stack = div()
            .v_flex()
            .gap_4()
            .p_6()
            .child(target_section)
            .child(session_section)
            .child(log_section);

        let settings_stack = render_settings_panel(&self.state, &settings, language, cx);

        let main_column = match active_view {
            ActiveView::Dashboard => dashboard_stack,
            ActiveView::Settings => settings_stack,
        };

        div()
            .h_flex()
            .size_full()
            .bg(cx.theme().background)
            .child(sidebar)
            .child(
                div()
                    .flex_1()
                    .h_full()
                    .scrollable(Axis::Vertical)
                    .bg(cx.theme().background)
                    .child(main_column),
            )
    }
}

fn render_session_card(
    session: &SyncSession,
    targets: &[RemoteTarget],
    language: Language,
    cx: &mut Context<AppView>,
) -> impl IntoElement {
    let target_name = targets
        .iter()
        .find(|target| target.id == session.target_id)
        .map(|target| target.name.clone())
        .unwrap_or_else(|| {
            format!(
                "{} {}",
                tr(language, "Target", "目标", "目標"),
                session.target_id
            )
        });

    let status_label = status_text(&session.status, language);
    let badge = status_tag(&session.status).child(status_label.clone());

    let progress_block = if let SyncStatus::Running { progress } = session.status {
        Some(
            div()
                .v_flex()
                .gap_1()
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child("Progress"),
                )
                .child(ProgressBar::new().value(progress.clamp(0.0, 1.0) * 100.0)),
        )
    } else {
        None
    };

    div()
        .v_flex()
        .gap_2()
        .p_4()
        .rounded(cx.theme().radius)
        .bg(cx.theme().list)
        .child(
            div()
                .h_flex()
                .justify_between()
                .items_center()
                .child(
                    div().font_semibold().child(format!(
                        "{} #{}",
                        tr(language, "Session", "会话", "會話"),
                        session.id
                    )),
                )
                .child(badge),
        )
        .child(
            div()
                .h_flex()
                .gap_3()
                .items_center()
                .flex_wrap()
                .child(
                    Tag::info()
                        .small()
                        .rounded_full()
                        .child(format!(
                            "{} {target_name}",
                            tr(language, "Target:", "目标：", "目標：")
                        )),
                )
                .child(
                    Tag::secondary()
                        .small()
                        .rounded_full()
                        .child(format!(
                            "{} {}",
                            tr(language, "Pending:", "待处理：", "待處理："),
                            session.pending_actions
                        )),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child(
                            session
                                .last_run
                                .map(|ts| {
                                    format!(
                                        "{} {}",
                                        tr(language, "Last run", "上次运行", "上次執行"),
                                        format_timestamp(ts, language)
                                    )
                                })
                                .unwrap_or_else(|| {
                                    tr(language, "Never executed", "尚未执行", "尚未執行").into()
                                }),
                        ),
                ),
        )
        .when_some(progress_block, |this, block| this.child(block))
        .when(
            matches!(session.status, SyncStatus::Failed { .. }),
            |this| {
                if let SyncStatus::Failed { reason } = &session.status {
                    this.child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().danger_foreground)
                            .child(reason.clone()),
                    )
                } else {
                    this
                }
            },
        )
}

fn status_tag(status: &SyncStatus) -> Tag {
    match status {
        SyncStatus::Idle => Tag::secondary(),
        SyncStatus::Planning => Tag::info(),
        SyncStatus::AwaitingConfirmation => Tag::warning(),
        SyncStatus::Running { .. } => Tag::primary(),
        SyncStatus::Failed { .. } => Tag::danger(),
        SyncStatus::Completed => Tag::success(),
    }
    .small()
    .rounded_full()
}

fn log_icon(message: &str) -> IconName {
    let lowercase = message.to_ascii_lowercase();
    if lowercase.contains("fail") || lowercase.contains("error") {
        IconName::TriangleAlert
    } else if lowercase.contains("stage") || lowercase.contains("detect") {
        IconName::LayoutDashboard
    } else {
        IconName::CircleCheck
    }
}

fn render_settings_panel(
    state: &Entity<AppState>,
    settings: &AppSettings,
    language: Language,
    cx: &mut Context<AppView>,
) -> Div {
    let auto_handle = state.clone();
    let auto_connect = Switch::new("auto_connect")
        .checked(settings.auto_connect)
        .on_click(move |next, _, cx| {
            auto_handle.update(cx, |state, cx| {
                state.settings.auto_connect = *next;
                cx.notify();
            });
        });

    let watch_handle = state.clone();
    let watch_changes = Switch::new("watch_changes")
        .checked(settings.watch_local_changes)
        .on_click(move |next, _, cx| {
            watch_handle.update(cx, |state, cx| {
                state.settings.watch_local_changes = *next;
                cx.notify();
            });
        });

    let confirm_handle = state.clone();
    let confirm_switch = Switch::new("confirm_destructive")
        .checked(settings.confirm_destructive)
        .on_click(move |next, _, cx| {
            confirm_handle.update(cx, |state, cx| {
                state.settings.confirm_destructive = *next;
                cx.notify();
            });
        });

    let limit_handle = state.clone();
    let limit_switch = Switch::new("limit_bandwidth")
        .checked(settings.limit_bandwidth)
        .on_click(move |next, _, cx| {
            limit_handle.update(cx, |state, cx| {
                state.settings.limit_bandwidth = *next;
                cx.notify();
            });
        });

    let decrease_handle = state.clone();
    let increase_handle = state.clone();
    let bandwidth_controls = div()
        .h_flex()
        .gap_2()
        .items_center()
        .child(
            Button::new("bw_decrease")
                .ghost()
                .icon(Icon::new(IconName::Minus).small())
                .disabled(!settings.limit_bandwidth || settings.bandwidth_mbps <= 10)
                .on_click(move |_, _, cx| {
                    decrease_handle.update(cx, |state, cx| {
                        if state.settings.bandwidth_mbps > 10 {
                            state.settings.bandwidth_mbps -= 10;
                            cx.notify();
                        }
                    });
                }),
        )
        .child(
            Tag::info()
                .small()
                .rounded_full()
                .child(format!("{} Mbps", settings.bandwidth_mbps)),
        )
        .child(
            Button::new("bw_increase")
                .ghost()
                .icon(Icon::new(IconName::Plus).small())
                .disabled(!settings.limit_bandwidth)
                .on_click(move |_, _, cx| {
                    increase_handle.update(cx, |state, cx| {
                        state.settings.bandwidth_mbps += 10;
                        cx.notify();
                    });
                }),
        );

    let language_handle = state.clone();
    let language_selector = LANGUAGE_CHOICES.iter().fold(
        div().h_flex().gap_2(),
        |builder, (choice, label)| {
            let mut button = Button::new(language_button_id(*choice)).label(*label);
            if *choice == settings.language {
                button = button.primary();
            } else {
                button = button.ghost();
            }
            builder.child(
                button.on_click({
                    let handle = language_handle.clone();
                    let selected = *choice;
                    move |_, _, cx| {
                        handle.update(cx, |state, cx| {
                            state.settings.language = selected;
                            cx.notify();
                        });
                        save_language(selected);
                    }
                }),
            )
        },
    );

    let general_box = GroupBox::new()
        .title(tr(language, "General", "常规", "一般"))
        .fill()
        .child(
            div()
                .v_flex()
                .gap_3()
                .child(settings_row(
                    tr(language, "Auto-connect", "自动连接", "自動連線"),
                    tr(
                        language,
                        "Attach to the last used remote as soon as the app launches.",
                        "启动应用后自动连接到上次使用的远程。",
                        "啟動應用後自動連線到上次使用的遠端。",
                    ),
                    auto_connect,
                    cx,
                ))
                .child(settings_row(
                    tr(language, "Watch local changes", "监视本地更改", "監視本地變更"),
                    tr(
                        language,
                        "Monitor the local workspace and enqueue diffs automatically.",
                        "监控本地工作区并自动加入差异。",
                        "監控本地工作區並自動加入差異。",
                    ),
                    watch_changes,
                    cx,
                ))
                .child(settings_row(
                    tr(language, "Interface language", "界面语言", "介面語言"),
                    tr(
                        language,
                        "Choose the display language.",
                        "选择界面显示语言。",
                        "選擇介面顯示語言。",
                    ),
                    language_selector,
                    cx,
                )),
        );

    let safety_box = GroupBox::new()
        .title(tr(language, "Safety & Limits", "安全与限制", "安全與限制"))
        .fill()
        .child(
            div()
                .v_flex()
                .gap_3()
                .child(settings_row(
                    tr(
                        language,
                        "Confirm destructive actions",
                        "破坏性操作需确认",
                        "破壞性操作需確認",
                    ),
                    tr(
                        language,
                        "Require explicit approval before deleting or overwriting remote files.",
                        "删除或覆盖远程文件前需要确认。",
                        "刪除或覆寫遠端檔案前需要確認。",
                    ),
                    confirm_switch,
                    cx,
                ))
                .child(settings_row(
                    tr(
                        language,
                        "Limit outbound bandwidth",
                        "限制上传带宽",
                        "限制上傳頻寬",
                    ),
                    tr(
                        language,
                        "Throttle transfer speed to keep headroom for other workloads.",
                        "限制传输速度，为其他任务保留带宽。",
                        "限制傳輸速度，為其他任務保留頻寬。",
                    ),
                    limit_switch,
                    cx,
                ))
                .child(
                    settings_row(
                        tr(language, "Bandwidth cap", "带宽上限", "頻寬上限"),
                        tr(
                            language,
                            "Applies when throttling is enabled.",
                            "仅在启用限速时生效。",
                            "僅在啟用限速時生效。",
                        ),
                        bandwidth_controls,
                        cx,
                    )
                    .when(!settings.limit_bandwidth, |row| row.opacity(0.5)),
                ),
        );

    div()
        .v_flex()
        .gap_4()
        .p_6()
        .child(
            div()
                .v_flex()
                .gap_1()
                .child(
                    div()
                        .text_2xl()
                        .font_semibold()
                        .child(tr(language, "Settings", "设置", "設定")),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child(tr(
                            language,
                            "Tune global behavior for every sync session.",
                            "调整所有同步任务的全局行为。",
                            "調整所有同步任務的全域行為。",
                        )),
                ),
        )
        .child(general_box)
        .child(safety_box)
}

fn settings_row(
    title: &str,
    description: &str,
    control: impl IntoElement,
    cx: &mut Context<AppView>,
) -> Div {
    let title_text = title.to_owned();
    let description_text = description.to_owned();
    let control = control.into_any_element();
    div()
        .h_flex()
        .gap_4()
        .items_center()
        .justify_between()
        .child(
            div()
                .v_flex()
                .gap_1()
                .child(div().font_semibold().child(title_text))
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child(description_text),
                ),
        )
        .child(control)
}

fn format_timestamp(ts: SystemTime, language: Language) -> String {
    match SystemTime::now().duration_since(ts) {
        Ok(elapsed) if elapsed < Duration::from_secs(60) => {
            tr(language, "just now", "刚刚", "剛剛").into()
        }
        Ok(elapsed) if elapsed < Duration::from_secs(3600) => {
            let minutes = elapsed.as_secs() / 60;
            match language {
                Language::English => format!("{minutes}m ago"),
                Language::SimplifiedChinese => format!("{minutes} 分钟前"),
                Language::TraditionalChinese => format!("{minutes} 分鐘前"),
            }
        }
        Ok(elapsed) => {
            let hours = elapsed.as_secs() / 3600;
            match language {
                Language::English => format!("{hours}h ago"),
                Language::SimplifiedChinese => format!("{hours} 小时前"),
                Language::TraditionalChinese => format!("{hours} 小時前"),
            }
        }
        Err(_) => tr(language, "in the future", "未来", "未來").into(),
    }
}

fn status_text(status: &SyncStatus, language: Language) -> String {
    match status {
        SyncStatus::Idle => tr(language, "Idle", "空闲", "閒置").into(),
        SyncStatus::Planning => tr(language, "Planning sync plan", "规划同步计划", "規畫同步計畫").into(),
        SyncStatus::AwaitingConfirmation => tr(
            language,
            "Awaiting user confirmation",
            "等待用户确认",
            "等待使用者確認",
        )
        .into(),
        SyncStatus::Running { progress } => match language {
            Language::English => format!("Running ({:.0}% complete)", progress.clamp(0.0, 1.0) * 100.0),
            Language::SimplifiedChinese => format!(
                "运行中（完成 {:.0}%）",
                progress.clamp(0.0, 1.0) * 100.0
            ),
            Language::TraditionalChinese => format!(
                "執行中（完成 {:.0}%）",
                progress.clamp(0.0, 1.0) * 100.0
            ),
        },
        SyncStatus::Failed { reason } => match language {
            Language::English => format!("Failed: {reason}"),
            Language::SimplifiedChinese => format!("失败：{reason}"),
            Language::TraditionalChinese => format!("失敗：{reason}"),
        },
        SyncStatus::Completed => tr(language, "Completed", "已完成", "已完成").into(),
    }
}

fn direction_label(direction: SyncDirection, language: Language) -> &'static str {
    match direction {
        SyncDirection::Push => tr(language, "local → remote", "本地 → 远程", "本地 → 遠端"),
        SyncDirection::Pull => tr(language, "remote → local", "远程 → 本地", "遠端 → 本地"),
        SyncDirection::Bidirectional => tr(language, "two-way", "双向", "雙向"),
    }
}

const LANGUAGE_CHOICES: &[(Language, &str)] = &[
    (Language::English, "English"),
    (Language::SimplifiedChinese, "简体中文"),
    (Language::TraditionalChinese, "繁體中文"),
];

fn language_button_id(language: Language) -> &'static str {
    match language {
        Language::English => "lang_en",
        Language::SimplifiedChinese => "lang_zh_hans",
        Language::TraditionalChinese => "lang_zh_hant",
    }
}

fn tr(language: Language, en: &'static str, zh_hans: &'static str, zh_hant: &'static str) -> &'static str {
    match language {
        Language::English => en,
        Language::SimplifiedChinese => zh_hans,
        Language::TraditionalChinese => zh_hant,
    }
}
