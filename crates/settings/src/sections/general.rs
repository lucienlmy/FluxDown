//! 通用：启动与托盘、系统集成、界面可见性、自定义分类。

use gpui::App;
use gpui_component::{
    Icon, IconName,
    setting::{SettingField, SettingGroup, SettingPage},
};

use super::{SectionContext, categories};

pub(crate) fn page(ctx: &SectionContext, cx: &mut App) -> SettingPage {
    // 首次进入拉取系统集成状态（自启 / 文件关联 / URL scheme）。
    if ctx.store.read(cx).integration().is_none() && !ctx.store.read(cx).is_busy("integration") {
        ctx.store.update(cx, |store, cx| store.load_integration(cx));
    }

    SettingPage::new(ctx.t("settingsCatGeneral"))
        .icon(Icon::new(IconName::Settings2))
        .description(ctx.t("settingsCatGeneralDesc"))
        .group(startup_group(ctx, cx))
        .group(system_group(ctx, cx))
        .group(sidebar_group(ctx))
        .group(titlebar_group(ctx))
        .group(categories::group(ctx, cx))
}

fn startup_group(ctx: &SectionContext, cx: &mut App) -> SettingGroup {
    SettingGroup::new()
        .title(ctx.t("settingsGroupStartupTray"))
        .item(
            ctx.item(
                "autoStartup",
                Some("autoStartupDesc"),
                integration_switch(ctx, IntegrationKind::Autostart),
            )
            .disabled(!integration_supported(ctx, IntegrationKind::Autostart, cx)),
        )
        .item(
            ctx.item(
                "closeToTray",
                Some(tray_desc("closeToTrayDesc")),
                ctx.pref_switch("close_to_tray", true),
            )
            .disabled(!TRAY_SUPPORTED),
        )
        .item(
            ctx.item(
                "startMinimizedToTray",
                Some(tray_desc("startMinimizedToTrayDesc")),
                ctx.pref_switch("start_minimized_to_tray", false),
            )
            .disabled(!TRAY_SUPPORTED),
        )
}

/// 托盘仅 Windows / macOS 提供（GPUI 端 Linux 不做托盘，关窗即退出）。
const TRAY_SUPPORTED: bool = cfg!(any(windows, target_os = "macos"));

fn tray_desc(key: &'static str) -> &'static str {
    if TRAY_SUPPORTED {
        key
    } else {
        "trayUnsupportedLinux"
    }
}

fn system_group(ctx: &SectionContext, cx: &mut App) -> SettingGroup {
    SettingGroup::new()
        .title(ctx.t("settingsGroupSystem"))
        .item(ctx.item(
            "clipboardWatch",
            Some("clipboardWatchDesc"),
            ctx.pref_switch("general.clipboard_watch", false),
        ))
        .item(
            ctx.item(
                "torrentFileAssociation",
                Some("torrentFileAssociationDesc"),
                integration_switch(ctx, IntegrationKind::Torrent),
            )
            .disabled(!integration_supported(ctx, IntegrationKind::Torrent, cx)),
        )
        .item(
            ctx.item(
                "magnetLinkAssociation",
                Some("magnetLinkAssociationDesc"),
                integration_switch(ctx, IntegrationKind::Scheme("magnet")),
            )
            .disabled(!integration_supported(
                ctx,
                IntegrationKind::Scheme("magnet"),
                cx,
            )),
        )
        .item(
            ctx.item(
                "ed2kLinkAssociation",
                Some("ed2kLinkAssociationDesc"),
                integration_switch(ctx, IntegrationKind::Scheme("ed2k")),
            )
            .disabled(!integration_supported(
                ctx,
                IntegrationKind::Scheme("ed2k"),
                cx,
            )),
        )
        .item(ctx.item(
            "keepAwakeWhileDownloading",
            Some("keepAwakeWhileDownloadingDesc"),
            ctx.pref_switch("download.keep_awake", false),
        ))
        .item(ctx.item(
            "analyticsEnabled",
            Some("analyticsEnabledDesc"),
            ctx.pref_switch("analytics_enabled", true),
        ))
}

fn sidebar_group(ctx: &SectionContext) -> SettingGroup {
    SettingGroup::new()
        .title(ctx.t("sidebarVisibility"))
        .description(ctx.t("sidebarVisibilityDesc"))
        .item(ctx.item(
            "showSidebarStatus",
            Some("showSidebarStatusDesc"),
            ctx.pref_switch("ui.show_sidebar_status", true),
        ))
        .item(ctx.item(
            "showSidebarQueues",
            Some("showSidebarQueuesDesc"),
            ctx.pref_switch("ui.show_sidebar_queues", true),
        ))
        .item(ctx.item(
            "showSidebarRss",
            Some("showSidebarRssDesc"),
            ctx.pref_switch("ui.show_sidebar_rss", true),
        ))
        .item(ctx.item(
            "showSidebarCategory",
            Some("showSidebarCategoryDesc"),
            ctx.pref_switch("ui.show_sidebar_category", true),
        ))
        .item(ctx.item(
            "showSidebarDevice",
            Some("showSidebarDeviceDesc"),
            show_sidebar_device_field(ctx),
        ))
}

/// `show_sidebar_device` 三态：未设置 = 登录后自动显示。开关显示有效值。
fn show_sidebar_device_field(ctx: &SectionContext) -> SettingField<bool> {
    let get = ctx.store();
    let set = ctx.store();
    SettingField::switch(
        move |cx: &App| {
            let store = get.read(cx);
            store
                .pref("show_sidebar_device")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or_else(|| store.session().is_some())
        },
        move |value, cx: &mut App| {
            set.update(cx, |store, cx| {
                store.set_pref_bool("show_sidebar_device", value, cx)
            });
        },
    )
}

fn titlebar_group(ctx: &SectionContext) -> SettingGroup {
    SettingGroup::new()
        .title(ctx.t("titlebarButtons"))
        .description(ctx.t("titlebarButtonsDesc"))
        .item(ctx.item(
            "showTitlebarPauseAll",
            Some("showTitlebarPauseAllDesc"),
            ctx.pref_switch("ui.show_titlebar_pause_all", true),
        ))
        .item(ctx.item(
            "showTitlebarResumeAll",
            Some("showTitlebarResumeAllDesc"),
            ctx.pref_switch("ui.show_titlebar_resume_all", true),
        ))
        .item(ctx.item(
            "showTitlebarSettings",
            Some("showTitlebarSettingsDesc"),
            ctx.pref_switch("ui.show_titlebar_settings", true),
        ))
        .item(ctx.item(
            "showTitlebarTheme",
            Some("showTitlebarThemeDesc"),
            ctx.pref_switch("ui.show_titlebar_theme", true),
        ))
}

#[derive(Clone, Copy)]
enum IntegrationKind {
    Autostart,
    Torrent,
    Scheme(&'static str),
}

fn integration_supported(ctx: &SectionContext, kind: IntegrationKind, cx: &App) -> bool {
    let store = ctx.store.read(cx);
    if store.is_read_only() {
        return false;
    }
    store.integration().is_some_and(|dto| match kind {
        IntegrationKind::Autostart => dto.autostart_supported,
        IntegrationKind::Torrent => dto.file_association_supported,
        IntegrationKind::Scheme(_) => dto.url_protocol_supported,
    })
}

/// 系统集成开关：值来自 agent 探测结果，切换即调用 agent 注册/注销。
fn integration_switch(ctx: &SectionContext, kind: IntegrationKind) -> SettingField<bool> {
    let get = ctx.store();
    let set = ctx.store();
    SettingField::switch(
        move |cx: &App| {
            get.read(cx).integration().is_some_and(|dto| match kind {
                IntegrationKind::Autostart => dto.autostart_enabled,
                IntegrationKind::Torrent => dto.torrent_associated,
                IntegrationKind::Scheme(scheme) => {
                    dto.url_protocols.get(scheme).copied().unwrap_or(false)
                }
            })
        },
        move |value, cx: &mut App| {
            set.update(cx, |store, cx| match kind {
                IntegrationKind::Autostart => store.set_autostart(value, cx),
                IntegrationKind::Torrent => {
                    store.set_pref_bool("torrent_assoc_user_disabled", !value, cx);
                    store.set_file_association(value, cx);
                }
                IntegrationKind::Scheme(scheme) => {
                    let key: &'static str = match scheme {
                        "magnet" => "magnet_assoc_user_disabled",
                        _ => "ed2k_assoc_user_disabled",
                    };
                    store.set_pref_bool(key, !value, cx);
                    store.set_url_protocol(scheme, value, cx);
                }
            });
        },
    )
}
