use std::{
    path::PathBuf,
    time::{Duration, SystemTime},
};

use crate::{
    config::save_state,
    connection,
    model::{
        ActiveView, AppSettings, AppState, ConnectionTestState, Language, RemoteTarget,
        SyncDirection, SyncRule, SyncSession, SyncStatus, TargetFormMode, TargetId,
    },
};
use anyhow::Error;
use gpui::{
    AppContext, Axis, Context, Div, Entity, IntoElement, ParentElement as _, Render, Styled as _,
    Window, div, prelude::FluentBuilder as _,
};
use gpui_component::{
    ActiveTheme, ContextModal, Disableable, Icon, IconName, Root, Sizable as _, StyledExt,
    button::*,
    group_box::GroupBox,
    input::{InputState, TextInput},
    modal::ModalButtonProps,
    progress::Progress as ProgressBar,
    sidebar::{Sidebar, SidebarFooter, SidebarGroup, SidebarHeader, SidebarMenu, SidebarMenuItem},
    switch::Switch,
    tag::Tag,
};

pub struct AppView {
    state: Entity<AppState>,
    target_form_view: Option<Entity<TargetFormView>>,
    current_form_mode: Option<TargetFormMode>,
}

impl AppView {
    pub fn new(state: Entity<AppState>) -> Self {
        Self {
            state,
            target_form_view: None,
            current_form_mode: None,
        }
    }
}

impl Render for AppView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (
            remote_targets,
            sessions,
            logs,
            active_target_id,
            active_view,
            settings,
            target_form_mode,
            connection_tests,
        ) = {
            let state = self.state.read(cx);
            (
                state.remote_targets.clone(),
                state.sessions.clone(),
                state.logs.clone(),
                state.active_target,
                state.active_view,
                state.settings.clone(),
                state.target_form,
                state.connection_tests.clone(),
            )
        };
        let language = settings.language;

        match target_form_mode {
            Some(mode) => {
                if self.current_form_mode != Some(mode) {
                    self.target_form_view = None;
                    self.current_form_mode = Some(mode);
                }
            }
            None => {
                self.target_form_view = None;
                self.current_form_mode = None;
            }
        }

        let active_target = active_target_id.and_then(|id| {
            remote_targets
                .iter()
                .find(|target| target.id == id)
                .cloned()
        });

        let overview_handle = self.state.clone();
        let settings_handle = self.state.clone();
        let target_settings_handle = self.state.clone();
        let mut workspace_items = vec![
            SidebarMenuItem::new(tr(language, "Overview", "概览", "總覽"))
                .icon(Icon::new(IconName::LayoutDashboard).small())
                .active(matches!(active_view, ActiveView::Dashboard))
                .on_click(move |_, _, cx| {
                    overview_handle.update(cx, |state, cx| {
                        state.active_view = ActiveView::Dashboard;
                        cx.notify();
                    });
                }),
            SidebarMenuItem::new(tr(language, "Settings", "设置", "設定"))
                .icon(Icon::new(IconName::Settings).small())
                .active(matches!(active_view, ActiveView::Settings))
                .on_click(move |_, _, cx| {
                    settings_handle.update(cx, |state, cx| {
                        state.active_view = ActiveView::Settings;
                        cx.notify();
                    });
                }),
        ];

        workspace_items.push(
            SidebarMenuItem::new(tr(language, "Target Settings", "目标设置", "目標設定"))
                .icon(Icon::new(IconName::Folder).small())
                .active(matches!(active_view, ActiveView::TargetSettings))
                .on_click(move |_, _, cx| {
                    target_settings_handle.update(cx, |state, cx| {
                        if state.target_form.is_none() {
                            state.target_form = Some(TargetFormMode::Create);
                        }
                        state.active_view = ActiveView::TargetSettings;
                        cx.notify();
                    });
                }),
        );

        let workspace_menu = SidebarMenu::new().children(workspace_items);

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
                Tag::warning().small().rounded_full().child(format!(
                    "{pending} {}",
                    tr(language, "pending", "待处理", "待處理")
                ))
            } else {
                Tag::secondary().small().rounded_full().child(format!(
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

        let add_target_handle = self.state.clone();
        let sidebar = Sidebar::left()
            .header(
                SidebarHeader::new().child(
                    div()
                        .v_flex()
                        .gap_1()
                        .child(div().text_lg().font_semibold().child(tr(
                            language,
                            "SFTP Sync",
                            "SFTP 同步",
                            "SFTP 同步",
                        )))
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
            .child(
                SidebarGroup::new(tr(language, "Workspace", "工作区", "工作區"))
                    .child(workspace_menu),
            )
            .child(SidebarGroup::new(tr(language, "Targets", "目标", "目標")).child(sidebar_menu))
            .footer(
                SidebarFooter::new().child(
                    Button::new("add_target")
                        .ghost()
                        .small()
                        .icon(Icon::new(IconName::Plus).small())
                        .label(tr(language, "Add Target", "新增目标", "新增目標"))
                        .on_click(move |_, _, cx| {
                            add_target_handle.update(cx, |state, cx| {
                                state.active_view = ActiveView::TargetSettings;
                                state.target_form = Some(TargetFormMode::Create);
                                cx.notify();
                            });
                        }),
                ),
            );

        let target_section = GroupBox::new()
            .title(tr(language, "Connection", "连接", "連線"))
            .fill()
            .child(match active_target {
                Some(target) => {
                    let edit_handle = self.state.clone();
                    let delete_handle = self.state.clone();
                    let target_id = target.id;
                    let rule_list =
                        target
                            .rules
                            .iter()
                            .fold(div().v_flex().gap_2(), |builder, rule| {
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
                                                            cx.theme()
                                                                .muted_foreground
                                                                .opacity(0.9),
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
                            });

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
                                .items_center()
                                .flex_wrap()
                                .child({
                                    let test_status = connection_tests.get(&target.id).cloned();
                                    let is_testing = matches!(
                                        test_status,
                                        Some(ConnectionTestState::InProgress)
                                    );
                                    let test_handle = self.state.clone();
                                    let target_for_test = target.clone();
                                    Button::new(("test_connection", target.id))
                                        .info()
                                        .small()
                                        .label(tr(
                                            language,
                                            "Test Connection",
                                            "测试连接",
                                            "測試連線",
                                        ))
                                        .icon(Icon::new(IconName::SquareTerminal).small())
                                        .disabled(is_testing)
                                        .on_click(move |_, _, cx| {
                                            let handle = test_handle.clone();
                                            let target_clone = target_for_test.clone();
                                            cx.spawn(async move |cx| {
                                                let _ = handle.update(cx, |state, cx| {
                                                    state.connection_tests.insert(
                                                        target_clone.id,
                                                        ConnectionTestState::InProgress,
                                                    );
                                                    cx.notify();
                                                });

                                                let result =
                                                    connection::test_connection(&target_clone);

                                                let _ = handle.update(cx, |state, cx| {
                                                    let status = match result {
                                                        Ok(_) => ConnectionTestState::Success(
                                                            tr(
                                                                language,
                                                                "Connection OK",
                                                                "连接成功",
                                                                "連線成功",
                                                            )
                                                            .into(),
                                                        ),
                                                        Err(err) => ConnectionTestState::Failure(
                                                            err.to_string(),
                                                        ),
                                                    };
                                                    state
                                                        .connection_tests
                                                        .insert(target_clone.id, status);
                                                    cx.notify();
                                                });

                                                Ok::<_, Error>(())
                                            })
                                            .detach();
                                        })
                                })
                                .child(render_connection_status_tag(
                                    connection_tests.get(&target.id),
                                    language,
                                ))
                                .child(
                                    Button::new("plan_sync")
                                        .primary()
                                        .label(tr(
                                            language,
                                            "Plan Dry Run",
                                            "生成试运行计划",
                                            "產生試運行計畫",
                                        ))
                                        .icon(Icon::new(IconName::LayoutDashboard).small())
                                        .on_click(|_, _, _| {
                                            println!("TODO: trigger dry run planning")
                                        }),
                                )
                                .child(
                                    Button::new("sync_now")
                                        .success()
                                        .label(tr(language, "Execute Sync", "执行同步", "執行同步"))
                                        .icon(Icon::new(IconName::Check).small())
                                        .on_click(|_, _, _| println!("TODO: execute sync")),
                                )
                                .child(
                                    Button::new("edit_target")
                                        .ghost()
                                        .label(tr(
                                            language,
                                            "Edit Target",
                                            "编辑目标",
                                            "編輯目標",
                                        ))
                                        .icon(Icon::new(IconName::Settings).small())
                                        .on_click({
                                            let handle = edit_handle.clone();
                                            move |_, _, cx| {
                                                handle.update(cx, |state, cx| {
                                                    state.target_form = Some(TargetFormMode::Edit(target_id));
                                                    state.active_view = ActiveView::TargetSettings;
                                                    cx.notify();
                                                });
                                            }
                                        }),
                                )
                                .child(
                                    Button::new("delete_target")
                                        .danger()
                                        .label(tr(
                                            language,
                                            "Delete Target",
                                            "删除目标",
                                            "刪除目標",
                                        ))
                                        .icon(Icon::new(IconName::Delete).small())
                                        .on_click({
                                            let handle = delete_handle.clone();
                                            let target_name = target.name.clone();
                                            move |_, window, cx| {
                                                let handle = handle.clone();
                                                let target_name = target_name.clone();
                                                window.open_modal(cx, move |modal, _window, _cx| {
                                                    let message = format!(
                                                        "{}\n{}",
                                                        tr(
                                                            language,
                                                            "Are you sure you want to remove this target?",
                                                            "确定要删除该目标吗？",
                                                            "確定要刪除此目標嗎？",
                                                        ),
                                                        target_name,
                                                    );

                                                    modal
                                                        .confirm()
                                                        .title(tr(
                                                            language,
                                                            "Confirm Deletion",
                                                            "确认删除",
                                                            "確認刪除",
                                                        ))
                                                        .child(div().p_4().child(message))
                                                        .button_props(
                                                            ModalButtonProps::default()
                                                                .ok_text(tr(
                                                                    language,
                                                                    "Delete",
                                                                    "删除",
                                                                    "刪除",
                                                                ))
                                                                .ok_variant(ButtonVariant::Danger)
                                                                .cancel_text(tr(
                                                                    language,
                                                                    "Cancel",
                                                                    "取消",
                                                                    "取消",
                                                                )),
                                                        )
                                                        .on_ok({
                                                            let handle = handle.clone();
                                                            move |_, _, cx| {
                                                                handle.update(cx, |state, cx| {
                                                                    state.remote_targets.retain(|t| t.id != target_id);
                                                                    state.connection_tests.remove(&target_id);
                                                                    if state.active_target == Some(target_id) {
                                                                        state.active_target = state
                                                                            .remote_targets
                                                                            .first()
                                                                            .map(|t| t.id);
                                                                    }
                                                                    if matches!(
                                                                        state.target_form,
                                                                        Some(TargetFormMode::Edit(id)) if id == target_id
                                                                    ) {
                                                                        state.target_form = None;
                                                                        state.active_view = ActiveView::Dashboard;
                                                                    }
                                                                    save_state(
                                                                        &state.settings,
                                                                        &state.remote_targets,
                                                                    );
                                                                    cx.notify();
                                                                });
                                                                true
                                                            }
                                                        })
                                                        .on_cancel(|_, _, _| true)
                                                });
                                            }
                                        }),
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
            sessions
                .iter()
                .fold(div().v_flex().gap_3(), |builder, session| {
                    builder.child(render_session_card(session, &remote_targets, language, cx))
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

        let target_settings_box = if let Some(mode) = target_form_mode {
            let preset = match mode {
                TargetFormMode::Edit(id) => remote_targets.iter().find(|t| t.id == id).cloned(),
                TargetFormMode::Create => None,
            };

            let form_entity = self
                .target_form_view
                .get_or_insert_with(|| cx.new(|cx| TargetFormView::new(window, cx)))
                .clone();

            render_target_form_panel(
                language,
                form_entity,
                mode,
                preset,
                self.state.clone(),
                window,
                cx,
            )
        } else {
            GroupBox::new()
                .title(tr(language, "Target Settings", "目标设置", "目標設定"))
                .fill()
                .child(
                    div()
                        .p_6()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child(tr(
                            language,
                            "No target selected.",
                            "没有正在编辑的目标。",
                            "沒有正在編輯的目標。",
                        )),
                )
        };

        let target_settings_stack = div().v_flex().gap_4().p_6().child(target_settings_box);

        let main_column = match active_view {
            ActiveView::Dashboard => dashboard_stack,
            ActiveView::Settings => settings_stack,
            ActiveView::TargetSettings => target_settings_stack,
        };

        div()
            .relative()
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
            .when_some(Root::render_drawer_layer(window, cx), |this, layer| this.child(layer))
            .when_some(Root::render_modal_layer(window, cx), |this, layer| this.child(layer))
            .when_some(
                Root::render_notification_layer(window, cx),
                |this, layer| this.child(layer),
            )
    }
}

fn render_target_form_panel(
    language: Language,
    form: Entity<TargetFormView>,
    mode: TargetFormMode,
    preset: Option<RemoteTarget>,
    state_handle: Entity<AppState>,
    window: &mut Window,
    cx: &mut Context<AppView>,
) -> GroupBox {
    let preset_ref = preset.as_ref();
    form.update(cx, |form_view, cx| {
        form_view.ensure_mode(window, cx, mode, preset_ref);
    });

    let form_state = form.read(cx);
    let name_input = form_state.name.clone();
    let host_input = form_state.host.clone();
    let username_input = form_state.username.clone();
    let base_path_input = form_state.base_path.clone();
    let local_path_input = form_state.local_path.clone();
    let remote_path_input = form_state.remote_path.clone();
    let password_input = form_state.password.clone();
    let direction = form_state.direction;

    let name_value = current_input_value(&name_input, cx);
    let host_value = current_input_value(&host_input, cx);
    let username_value = current_input_value(&username_input, cx);
    let base_path_value = current_input_value(&base_path_input, cx);
    let local_value = current_input_value(&local_path_input, cx);
    let remote_value = current_input_value(&remote_path_input, cx);

    let password_value = current_input_value(&password_input, cx);

    let ready_to_submit = [
        &name_value,
        &host_value,
        &username_value,
        &base_path_value,
        &local_value,
        &remote_value,
        &password_value,
    ]
    .iter()
    .all(|value| !value.trim().is_empty());

    let direction_buttons = [
        SyncDirection::Push,
        SyncDirection::Pull,
        SyncDirection::Bidirectional,
    ]
    .into_iter()
    .fold(div().h_flex().gap_2(), |builder, dir| {
        let mut button = Button::new(direction_button_id(dir))
            .small()
            .label(direction_label(dir, language));
        if dir == direction {
            button = button.primary();
        } else {
            button = button.ghost();
        }
        builder.child(button.on_click({
            let handle = form.clone();
            let selected_direction = dir;
            move |_, _, cx| {
                handle.update(cx, |form, cx| {
                    form.direction = selected_direction;
                    cx.notify();
                });
            }
        }))
    });

    let cancel_handle = state_handle.clone();
    let cancel_button = Button::new("cancel_target_creation")
        .ghost()
        .label(tr(language, "Cancel", "取消", "取消"))
        .on_click(move |_, _, cx| {
            cancel_handle.update(cx, |state, cx| {
                state.target_form = None;
                state.active_view = ActiveView::Dashboard;
                cx.notify();
            });
        });

    let submit_handle = state_handle.clone();
    let form_handle = form.clone();
    let submit_button = Button::new("target_form_submit")
        .primary()
        .disabled(!ready_to_submit)
        .label(match mode {
            TargetFormMode::Create => tr(language, "Create Target", "创建目标", "建立目標"),
            TargetFormMode::Edit(_) => tr(language, "Save Changes", "保存更改", "儲存變更"),
        })
        .on_click(move |_, _, cx| match mode {
            TargetFormMode::Create => {
                let next_id = {
                    let state = submit_handle.read(cx);
                    state.next_target_id()
                };
                if let Some(new_target) =
                    form_handle.update(cx, |form, cx| form.build_target(next_id, cx))
                {
                    submit_handle.update(cx, |state, cx| {
                        state.remote_targets.push(new_target);
                        state.active_target = state.remote_targets.last().map(|target| target.id);
                        state.target_form = None;
                        state.active_view = ActiveView::Dashboard;
                        save_state(&state.settings, &state.remote_targets);
                        cx.notify();
                    });
                }
            }
            TargetFormMode::Edit(target_id) => {
                if let Some(updated) =
                    form_handle.update(cx, |form, cx| form.build_target(target_id, cx))
                {
                    submit_handle.update(cx, |state, cx| {
                        if let Some(existing) = state
                            .remote_targets
                            .iter_mut()
                            .find(|target| target.id == target_id)
                        {
                            *existing = updated;
                        }
                        save_state(&state.settings, &state.remote_targets);
                        state.target_form = None;
                        state.active_view = ActiveView::Dashboard;
                        cx.notify();
                    });
                }
            }
        });

    GroupBox::new()
        .title(match mode {
            TargetFormMode::Create => tr(language, "New Target", "新增目标", "新增目標"),
            TargetFormMode::Edit(_) => tr(language, "Edit Target", "编辑目标", "編輯目標"),
        })
        .fill()
        .child(
            div()
                .v_flex()
                .gap_3()
                .child(settings_row(
                    tr(language, "Name", "名称", "名稱"),
                    tr(
                        language,
                        "Friendly label shown in the sidebar.",
                        "显示在侧边栏中的名称。",
                        "顯示在側邊欄中的名稱。",
                    ),
                    TextInput::new(&name_input).small(),
                    cx,
                ))
                .child(settings_row(
                    tr(language, "Host", "主机", "主機"),
                    tr(
                        language,
                        "hostname:port for the remote server.",
                        "远程服务器的主机名和端口。",
                        "遠端伺服器的主機與連接埠。",
                    ),
                    TextInput::new(&host_input).small(),
                    cx,
                ))
                .child(settings_row(
                    tr(language, "Username", "用户名", "使用者名稱"),
                    tr(
                        language,
                        "Account used for SSH/SFTP authentication.",
                        "用于 SSH/SFTP 认证的账户。",
                        "用於 SSH/SFTP 驗證的帳號。",
                    ),
                    TextInput::new(&username_input).small(),
                    cx,
                ))
                .child(settings_row(
                    tr(language, "Password", "密码", "密碼"),
                    tr(
                        language,
                        "Stored locally for automatic authentication.",
                        "本地保存用于自动认证。",
                        "本地儲存用於自動驗證。",
                    ),
                    TextInput::new(&password_input).mask_toggle().small(),
                    cx,
                ))
                .child(settings_row(
                    tr(language, "Remote base path", "远程根路径", "遠端根路徑"),
                    tr(
                        language,
                        "Root directory on the remote machine.",
                        "远程主机上的根目录。",
                        "遠端主機上的根目錄。",
                    ),
                    TextInput::new(&base_path_input).small(),
                    cx,
                ))
                .child(settings_row(
                    tr(language, "Local path", "本地路径", "本地路徑"),
                    tr(
                        language,
                        "Workspace folder to watch and sync.",
                        "需要同步的本地工作目录。",
                        "需要同步的本地工作目錄。",
                    ),
                    TextInput::new(&local_path_input).small(),
                    cx,
                ))
                .child(settings_row(
                    tr(language, "Remote path", "远程路径", "遠端路徑"),
                    tr(
                        language,
                        "Destination relative to the base path.",
                        "相对于根路径的目标目录。",
                        "相對於根路徑的目標目錄。",
                    ),
                    TextInput::new(&remote_path_input).small(),
                    cx,
                ))
                .child(settings_row(
                    tr(language, "Direction", "同步方向", "同步方向"),
                    tr(
                        language,
                        "Choose how files should flow.",
                        "选择文件传输方向。",
                        "選擇檔案傳輸方向。",
                    ),
                    direction_buttons,
                    cx,
                ))
                .child(
                    div()
                        .h_flex()
                        .gap_2()
                        .justify_end()
                        .child(cancel_button)
                        .child(submit_button),
                ),
        )
}

fn current_input_value(input: &Entity<InputState>, cx: &mut Context<AppView>) -> String {
    input.read(cx).text().to_string()
}

fn render_connection_status_tag(status: Option<&ConnectionTestState>, language: Language) -> Tag {
    match status {
        Some(ConnectionTestState::InProgress) => Tag::warning().small().rounded_full().child(tr(
            language,
            "Testing...",
            "测试中...",
            "測試中...",
        )),
        Some(ConnectionTestState::Success(message)) => {
            Tag::success().small().rounded_full().child(message.clone())
        }
        Some(ConnectionTestState::Failure(reason)) => {
            Tag::danger().small().rounded_full().child(reason.clone())
        }
        None => Tag::secondary().small().rounded_full().child(tr(
            language,
            "Not tested",
            "尚未测试",
            "尚未測試",
        )),
    }
}

struct TargetFormView {
    name: Entity<InputState>,
    host: Entity<InputState>,
    username: Entity<InputState>,
    base_path: Entity<InputState>,
    local_path: Entity<InputState>,
    remote_path: Entity<InputState>,
    password: Entity<InputState>,
    direction: SyncDirection,
    loaded_from: Option<TargetId>,
}

impl TargetFormView {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            name: Self::spawn_input(window, cx, "Production", false),
            host: Self::spawn_input(window, cx, "prod.example.com:22", false),
            username: Self::spawn_input(window, cx, "deploy", false),
            base_path: Self::spawn_input(window, cx, "/srv/www", false),
            local_path: Self::spawn_input(window, cx, "./apps/web", false),
            remote_path: Self::spawn_input(window, cx, "/web", false),
            password: Self::spawn_input(window, cx, "••••••", true),
            direction: SyncDirection::Push,
            loaded_from: None,
        }
    }

    fn spawn_input(
        window: &mut Window,
        cx: &mut Context<Self>,
        placeholder: &str,
        masked: bool,
    ) -> Entity<InputState> {
        let placeholder_text = placeholder.to_string();
        cx.new(|cx| {
            let mut state = InputState::new(window, cx);
            state.set_placeholder(placeholder_text.clone(), window, cx);
            if masked {
                state.set_masked(true, window, cx);
            }
            state
        })
    }

    fn ensure_mode(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
        mode: TargetFormMode,
        preset: Option<&RemoteTarget>,
    ) {
        match mode {
            TargetFormMode::Create => {
                if self.loaded_from.is_some() {
                    self.reset(window, cx);
                }
            }
            TargetFormMode::Edit(target_id) => {
                if self.loaded_from != Some(target_id) {
                    if let Some(target) = preset {
                        self.prefill(window, cx, target);
                    } else {
                        self.reset(window, cx);
                        self.loaded_from = Some(target_id);
                    }
                }
            }
        }
    }

    fn reset(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.set_value(&self.name, "", window, cx);
        self.set_value(&self.host, "", window, cx);
        self.set_value(&self.username, "", window, cx);
        self.set_value(&self.base_path, "", window, cx);
        self.set_value(&self.local_path, "", window, cx);
        self.set_value(&self.remote_path, "", window, cx);
        self.set_value(&self.password, "", window, cx);
        self.direction = SyncDirection::Push;
        self.loaded_from = None;
    }

    fn prefill(&mut self, window: &mut Window, cx: &mut Context<Self>, target: &RemoteTarget) {
        self.set_value(&self.name, &target.name, window, cx);
        self.set_value(&self.host, &target.host, window, cx);
        self.set_value(&self.username, &target.username, window, cx);
        self.set_value(
            &self.base_path,
            target.base_path.to_str().unwrap_or_default(),
            window,
            cx,
        );
        if let Some(rule) = target.rules.first() {
            self.set_value(
                &self.local_path,
                rule.local.to_str().unwrap_or_default(),
                window,
                cx,
            );
            self.set_value(
                &self.remote_path,
                rule.remote.to_str().unwrap_or_default(),
                window,
                cx,
            );
            self.direction = rule.direction;
        } else {
            self.set_value(&self.local_path, "", window, cx);
            self.set_value(&self.remote_path, "", window, cx);
            self.direction = SyncDirection::Push;
        }
        self.set_value(&self.password, &target.password, window, cx);
        self.loaded_from = Some(target.id);
    }

    fn set_value(
        &self,
        input: &Entity<InputState>,
        value: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let text = value.to_string();
        let _ = input.update(cx, |state, cx| {
            state.set_value(text.clone(), window, cx);
        });
    }

    fn build_target(&self, next_id: TargetId, cx: &mut Context<Self>) -> Option<RemoteTarget> {
        let draft = TargetDraft {
            name: self.read(&self.name, cx),
            host: self.read(&self.host, cx),
            username: self.read(&self.username, cx),
            base_path: self.read(&self.base_path, cx),
            local_path: self.read(&self.local_path, cx),
            remote_path: self.read(&self.remote_path, cx),
            password: self.read(&self.password, cx),
            direction: self.direction,
        };
        draft.into_remote_target(next_id)
    }

    fn read(&self, input: &Entity<InputState>, cx: &mut Context<Self>) -> String {
        input.read(cx).text().to_string()
    }
}

struct TargetDraft {
    name: String,
    host: String,
    username: String,
    base_path: String,
    local_path: String,
    remote_path: String,
    password: String,
    direction: SyncDirection,
}

impl TargetDraft {
    fn is_valid(&self) -> bool {
        !self.name.trim().is_empty()
            && !self.host.trim().is_empty()
            && !self.username.trim().is_empty()
            && !self.base_path.trim().is_empty()
            && !self.local_path.trim().is_empty()
            && !self.remote_path.trim().is_empty()
            && !self.password.trim().is_empty()
    }

    fn into_remote_target(self, id: TargetId) -> Option<RemoteTarget> {
        if !self.is_valid() {
            return None;
        }
        Some(RemoteTarget {
            id,
            name: self.name.trim().to_string(),
            host: self.host.trim().to_string(),
            username: self.username.trim().to_string(),
            base_path: PathBuf::from(self.base_path.trim()),
            rules: vec![SyncRule {
                local: PathBuf::from(self.local_path.trim()),
                remote: PathBuf::from(self.remote_path.trim()),
                direction: self.direction,
            }],
            password: self.password.trim().to_string(),
        })
    }
}

fn direction_button_id(direction: SyncDirection) -> &'static str {
    match direction {
        SyncDirection::Push => "direction_push",
        SyncDirection::Pull => "direction_pull",
        SyncDirection::Bidirectional => "direction_bidi",
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
                .child(div().font_semibold().child(format!(
                    "{} #{}",
                    tr(language, "Session", "会话", "會話"),
                    session.id
                )))
                .child(badge),
        )
        .child(
            div()
                .h_flex()
                .gap_3()
                .items_center()
                .flex_wrap()
                .child(Tag::info().small().rounded_full().child(format!(
                    "{} {target_name}",
                    tr(language, "Target:", "目标：", "目標：")
                )))
                .child(Tag::secondary().small().rounded_full().child(format!(
                    "{} {}",
                    tr(language, "Pending:", "待处理：", "待處理："),
                    session.pending_actions
                )))
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
                save_state(&state.settings, &state.remote_targets);
                cx.notify();
            });
        });

    let watch_handle = state.clone();
    let watch_changes = Switch::new("watch_changes")
        .checked(settings.watch_local_changes)
        .on_click(move |next, _, cx| {
            watch_handle.update(cx, |state, cx| {
                state.settings.watch_local_changes = *next;
                save_state(&state.settings, &state.remote_targets);
                cx.notify();
            });
        });

    let confirm_handle = state.clone();
    let confirm_switch = Switch::new("confirm_destructive")
        .checked(settings.confirm_destructive)
        .on_click(move |next, _, cx| {
            confirm_handle.update(cx, |state, cx| {
                state.settings.confirm_destructive = *next;
                save_state(&state.settings, &state.remote_targets);
                cx.notify();
            });
        });

    let limit_handle = state.clone();
    let limit_switch = Switch::new("limit_bandwidth")
        .checked(settings.limit_bandwidth)
        .on_click(move |next, _, cx| {
            limit_handle.update(cx, |state, cx| {
                state.settings.limit_bandwidth = *next;
                save_state(&state.settings, &state.remote_targets);
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
                            save_state(&state.settings, &state.remote_targets);
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
                        save_state(&state.settings, &state.remote_targets);
                        cx.notify();
                    });
                }),
        );

    let language_handle = state.clone();
    let language_selector =
        LANGUAGE_CHOICES
            .iter()
            .fold(div().h_flex().gap_2(), |builder, (choice, label)| {
                let mut button = Button::new(language_button_id(*choice)).label(*label);
                if *choice == settings.language {
                    button = button.primary();
                } else {
                    button = button.ghost();
                }
                builder.child(button.on_click({
                    let handle = language_handle.clone();
                    let selected = *choice;
                    move |_, _, cx| {
                        handle.update(cx, |state, cx| {
                            state.settings.language = selected;
                            save_state(&state.settings, &state.remote_targets);
                            cx.notify();
                        });
                    }
                }))
            });

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
                    tr(
                        language,
                        "Watch local changes",
                        "监视本地更改",
                        "監視本地變更",
                    ),
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
        SyncStatus::Planning => tr(
            language,
            "Planning sync plan",
            "规划同步计划",
            "規畫同步計畫",
        )
        .into(),
        SyncStatus::AwaitingConfirmation => tr(
            language,
            "Awaiting user confirmation",
            "等待用户确认",
            "等待使用者確認",
        )
        .into(),
        SyncStatus::Running { progress } => match language {
            Language::English => format!(
                "Running ({:.0}% complete)",
                progress.clamp(0.0, 1.0) * 100.0
            ),
            Language::SimplifiedChinese => {
                format!("运行中（完成 {:.0}%）", progress.clamp(0.0, 1.0) * 100.0)
            }
            Language::TraditionalChinese => {
                format!("執行中（完成 {:.0}%）", progress.clamp(0.0, 1.0) * 100.0)
            }
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

fn tr(
    language: Language,
    en: &'static str,
    zh_hans: &'static str,
    zh_hant: &'static str,
) -> &'static str {
    match language {
        Language::English => en,
        Language::SimplifiedChinese => zh_hans,
        Language::TraditionalChinese => zh_hant,
    }
}
