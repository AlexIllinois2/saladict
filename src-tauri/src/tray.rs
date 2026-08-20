use crate::clipboard::*;
use crate::config::{get, set};
use crate::window::config_window;
use crate::window::input_translate;
use crate::window::ocr_recognize;
use crate::window::ocr_translate;
use crate::window::updater_window;
use log::info;
use tauri::menu::{CheckMenuItem, IsMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::tray::{MouseButton, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, Wry};
use tauri_plugin_global_shortcut::GlobalShortcutExt;
use tauri_plugin_shell::ShellExt;
use crate::cmd::is_app_store_version;

struct TrayLabels {
    input_translate: &'static str,
    clipboard_monitor: &'static str,
    auto_copy: &'static str,
    copy_source: &'static str,
    copy_target: &'static str,
    copy_source_target: &'static str,
    copy_disable: &'static str,
    ocr_recognize: &'static str,
    ocr_translate: &'static str,
    config: &'static str,
    check_update: &'static str,
    view_log: &'static str,
    restart: &'static str,
    quit: &'static str,
}

const LABELS_EN: TrayLabels = TrayLabels {
    input_translate: "Input Translate",
    clipboard_monitor: "Clipboard Monitor",
    auto_copy: "Auto Copy",
    copy_source: "Source",
    copy_target: "Target",
    copy_source_target: "Source+Target",
    copy_disable: "Disable",
    ocr_recognize: "OCR Recognize",
    ocr_translate: "OCR Translate",
    config: "Config",
    check_update: "Check Update",
    view_log: "View Log",
    restart: "Restart",
    quit: "Quit",
};

const LABELS_ZH_CN: TrayLabels = TrayLabels {
    input_translate: "输入翻译",
    clipboard_monitor: "监听剪切板",
    auto_copy: "自动复制",
    copy_source: "原文",
    copy_target: "译文",
    copy_source_target: "原文+译文",
    copy_disable: "关闭",
    ocr_recognize: "文字识别",
    ocr_translate: "截图翻译",
    config: "偏好设置",
    check_update: "检查更新",
    view_log: "查看日志",
    restart: "重启应用",
    quit: "退出",
};

const LABELS_ZH_TW: TrayLabels = TrayLabels {
    input_translate: "輸入翻譯",
    clipboard_monitor: "偵聽剪貼簿",
    auto_copy: "自動複製",
    copy_source: "原文",
    copy_target: "譯文",
    copy_source_target: "原文+譯文",
    copy_disable: "關閉",
    ocr_recognize: "文字識別",
    ocr_translate: "截圖翻譯",
    config: "偏好設定",
    check_update: "檢查更新",
    view_log: "查看日誌",
    restart: "重啓程式",
    quit: "退出",
};

const LABELS_JA: TrayLabels = TrayLabels {
    input_translate: "翻訳を入力",
    clipboard_monitor: "クリップボードを監視する",
    auto_copy: "自動コピー",
    copy_source: "原文",
    copy_target: "訳文",
    copy_source_target: "原文+訳文",
    copy_disable: "閉じる",
    ocr_recognize: "テキスト認識",
    ocr_translate: "スクリーンショットの翻訳",
    config: "プリファレンス設定",
    check_update: "更新を確認する",
    view_log: "ログを見る",
    restart: "アプリの再起動",
    quit: "退出する",
};

const LABELS_KO: TrayLabels = TrayLabels {
    input_translate: "입력 번역",
    clipboard_monitor: "감청 전단판",
    auto_copy: "자동 복사",
    copy_source: "원문",
    copy_target: "번역문",
    copy_source_target: "원문+번역문",
    copy_disable: "닫기",
    ocr_recognize: "문자인식",
    ocr_translate: "스크린샷 번역",
    config: "기본 설정",
    check_update: "업데이트 확인",
    view_log: "로그 보기",
    restart: "응용 프로그램 다시 시작",
    quit: "퇴출",
};

const LABELS_FR: TrayLabels = TrayLabels {
    input_translate: "Traduction d'entrée",
    clipboard_monitor: "Surveiller le presse-papiers",
    auto_copy: "Copier automatiquement",
    copy_source: "Source",
    copy_target: "Cible",
    copy_source_target: "Source+Cible",
    copy_disable: "Désactiver",
    ocr_recognize: "Reconnaissance de texte",
    ocr_translate: "Traduction d'image",
    config: "Paramètres",
    check_update: "Vérifier les mises à jour",
    view_log: "Voir le journal",
    restart: "Redémarrer l'application",
    quit: "Quitter",
};

const LABELS_DE: TrayLabels = TrayLabels {
    input_translate: "Eingabeübersetzung",
    clipboard_monitor: "Zwischenablage überwachen",
    auto_copy: "Automatisch kopieren",
    copy_source: "Quelle",
    copy_target: "Ziel",
    copy_source_target: "Quelle+Ziel",
    copy_disable: "Deaktivieren",
    ocr_recognize: "Texterkennung",
    ocr_translate: "Bildübersetzung",
    config: "Einstellungen",
    check_update: "Auf Updates prüfen",
    view_log: "Protokoll anzeigen",
    restart: "Anwendung neu starten",
    quit: "Beenden",
};

const LABELS_RU: TrayLabels = TrayLabels {
    input_translate: "Ввод перевода",
    clipboard_monitor: "Следить за буфером обмена",
    auto_copy: "Автоматическое копирование",
    copy_source: "Источник",
    copy_target: "Цель",
    copy_source_target: "Источник+Цель",
    copy_disable: "Отключить",
    ocr_recognize: "Распознавание текста",
    ocr_translate: "Перевод изображения",
    config: "Настройки",
    check_update: "Проверить обновления",
    view_log: "Просмотр журнала",
    restart: "Перезапустить приложение",
    quit: "Выход",
};

const LABELS_FA: TrayLabels = TrayLabels {
    input_translate: "متن",
    clipboard_monitor: "گوش دادن به تخته برش",
    auto_copy: "کپی خودکار",
    copy_source: "منبع",
    copy_target: "هدف",
    copy_source_target: "منبع + هدف",
    copy_disable: "متن",
    ocr_recognize: "تشخیص متن",
    ocr_translate: "ترجمه عکس",
    config: "تنظیمات ترجیح",
    check_update: "بررسی بروزرسانی",
    view_log: "مشاهده گزارشات",
    restart: "راه‌اندازی مجدد برنامه",
    quit: "خروج",
};

const LABELS_PT_BR: TrayLabels = TrayLabels {
    input_translate: "Traduzir Entrada",
    clipboard_monitor: "Monitorando a área de transferência",
    auto_copy: "Copiar Automaticamente",
    copy_source: "Origem",
    copy_target: "Destino",
    copy_source_target: "Origem+Destino",
    copy_disable: "Desabilitar",
    ocr_recognize: "Reconhecimento de Texto",
    ocr_translate: "Tradução de Imagem",
    config: "Configurações",
    check_update: "Checar por Atualização",
    view_log: "Exibir Registro",
    restart: "Reiniciar aplicativo",
    quit: "Sair",
};

const LABELS_UK: TrayLabels = TrayLabels {
    input_translate: "Введення перекладу",
    clipboard_monitor: "Стежити за буфером обміну",
    auto_copy: "Автоматичне копіювання",
    copy_source: "Джерело",
    copy_target: "Мета",
    copy_source_target: "Джерело+Мета",
    copy_disable: "Відключивши",
    ocr_recognize: "Розпізнавання тексту",
    ocr_translate: "Переклад зображення",
    config: "Настройка",
    check_update: "Перевірити оновлення",
    view_log: "Перегляд журналу",
    restart: "Перезапустити додаток",
    quit: "Вихід",
};

fn tray_labels(language: &str) -> &'static TrayLabels {
    match language {
        "zh_cn" => &LABELS_ZH_CN,
        "zh_tw" => &LABELS_ZH_TW,
        "ja" => &LABELS_JA,
        "ko" => &LABELS_KO,
        "fr" => &LABELS_FR,
        "de" => &LABELS_DE,
        "ru" => &LABELS_RU,
        "pt_br" => &LABELS_PT_BR,
        "fa" => &LABELS_FA,
        "uk" => &LABELS_UK,
        _ => &LABELS_EN,
    }
}

fn build_tray_menu(
    app: &AppHandle,
    labels: &TrayLabels,
    clipboard_monitor: bool,
    copy_mode: &str,
) -> tauri::Result<Menu<Wry>> {
    let input_translate = MenuItem::with_id(
        app,
        "input_translate",
        labels.input_translate,
        true,
        None::<&str>,
    )?;
    let clipboard_monitor_item = CheckMenuItem::with_id(
        app,
        "clipboard_monitor",
        labels.clipboard_monitor,
        true,
        clipboard_monitor,
        None::<&str>,
    )?;
    let copy_source = CheckMenuItem::with_id(
        app,
        "copy_source",
        labels.copy_source,
        true,
        copy_mode == "source",
        None::<&str>,
    )?;
    let copy_target = CheckMenuItem::with_id(
        app,
        "copy_target",
        labels.copy_target,
        true,
        copy_mode == "target",
        None::<&str>,
    )?;
    let copy_source_target = CheckMenuItem::with_id(
        app,
        "copy_source_target",
        labels.copy_source_target,
        true,
        copy_mode == "source_target",
        None::<&str>,
    )?;
    let copy_disable = CheckMenuItem::with_id(
        app,
        "copy_disable",
        labels.copy_disable,
        true,
        copy_mode == "disable",
        None::<&str>,
    )?;
    let auto_copy_menu = Submenu::with_items(
        app,
        labels.auto_copy,
        true,
        &[
            &copy_source,
            &copy_target,
            &copy_source_target,
            &PredefinedMenuItem::separator(app)?,
            &copy_disable,
        ],
    )?;
    let ocr_recognize =
        MenuItem::with_id(app, "ocr_recognize", labels.ocr_recognize, true, None::<&str>)?;
    let ocr_translate =
        MenuItem::with_id(app, "ocr_translate", labels.ocr_translate, true, None::<&str>)?;
    let config = MenuItem::with_id(app, "config", labels.config, true, None::<&str>)?;
    let check_update =
        MenuItem::with_id(app, "check_update", labels.check_update, true, None::<&str>)?;
    let view_log = MenuItem::with_id(app, "view_log", labels.view_log, true, None::<&str>)?;
    let restart = MenuItem::with_id(app, "restart", labels.restart, true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", labels.quit, true, None::<&str>)?;
    let separator_before_ocr = PredefinedMenuItem::separator(app)?;
    let separator_before_config = PredefinedMenuItem::separator(app)?;
    let separator_before_quit = PredefinedMenuItem::separator(app)?;

    let mut items: Vec<&dyn IsMenuItem<Wry>> = vec![
        &input_translate,
        &clipboard_monitor_item,
        &auto_copy_menu,
        &separator_before_ocr,
        &ocr_recognize,
        &ocr_translate,
        &separator_before_config,
        &config,
    ];
    if !is_app_store_version() {
        items.push(&check_update);
    }
    if get("dev_mode").and_then(|v| v.as_bool()).unwrap_or(false) {
        items.push(&view_log);
    }
    items.push(&separator_before_quit);
    if !is_app_store_version() {
        items.push(&restart);
    }
    items.push(&quit);

    Menu::with_items(app, &items)
}

#[tauri::command]
pub fn update_tray(app_handle: AppHandle, mut language: String, mut copy_mode: String) {
    if language.is_empty() {
        language = match get("app_language") {
            Some(v) => v.as_str().unwrap().to_string(),
            None => {
                "en".to_string()
            }
        };
    }
    if copy_mode.is_empty() {
        copy_mode = match get("translate_auto_copy") {
            Some(v) => v.as_str().unwrap().to_string(),
            None => {
                set("translate_auto_copy", "disable");
                "disable".to_string()
            }
        };
    }

    info!(
        "Update tray with language: {}, copy mode: {}",
        language, copy_mode
    );

    if let Some(tray) = app_handle.tray_by_id("main") {
        let enable_clipboard_monitor = match get("clipboard_monitor") {
            Some(v) => v.as_bool().unwrap(),
            None => {
                set("clipboard_monitor", false);
                false
            }
        };

        match build_tray_menu(
            &app_handle,
            tray_labels(language.as_str()),
            enable_clipboard_monitor,
            copy_mode.as_str(),
        ) {
            Ok(menu) => {
                let _ = tray.set_menu(Some(menu));
            }
            Err(e) => {
                log::warn!("Failed to build tray menu: {}", e);
            }
        }

        #[cfg(not(target_os = "linux"))]
        {
            let _ = tray.set_tooltip(Some(&format!(
                "{} {}",
                app_handle.package_info().name,
                app_handle.package_info().version
            )));
        }
    }
}

pub fn tray_event_handler(_app: &AppHandle, event: TrayIconEvent) {
    match event {
        TrayIconEvent::Click { button, .. } => {
            if button == MouseButton::Left {
                on_tray_click();
            }
        }
        _ => {}
    }
}

pub fn handle_menu_event(app: &AppHandle, id: &str) {
    match id {
        "input_translate" => on_input_translate_click(),
        "copy_source" => on_auto_copy_click(app, "source"),
        "clipboard_monitor" => on_clipboard_monitor_click(app),
        "copy_target" => on_auto_copy_click(app, "target"),
        "copy_source_target" => on_auto_copy_click(app, "source_target"),
        "copy_disable" => on_auto_copy_click(app, "disable"),
        "ocr_recognize" => on_ocr_recognize_click(),
        "ocr_translate" => on_ocr_translate_click(),
        "config" => on_config_click(),
        "check_update" => on_check_update_click(),
        "view_log" => on_view_log_click(app),
        "restart" => on_restart_click(app),
        "quit" => on_quit_click(app),
        _ => {}
    }
}

fn on_tray_click() {
    let event = match get("tray_click_event") {
        Some(v) => v.as_str().unwrap().to_string(),
        None => {
            set("tray_click_event", "config");
            "config".to_string()
        }
    };
    match event.as_str() {
        "config" => config_window(),
        "translate" => input_translate(),
        "ocr_recognize" => ocr_recognize(),
        "ocr_translate" => ocr_translate(),
        "disable" => {}
        _ => config_window(),
    }
}
fn on_input_translate_click() {
    input_translate();
}
fn on_clipboard_monitor_click(app: &AppHandle) {
    let enable_clipboard_monitor = match get("clipboard_monitor") {
        Some(v) => v.as_bool().unwrap(),
        None => {
            set("clipboard_monitor", false);
            false
        }
    };
    let current = !enable_clipboard_monitor;
    // Update Config File
    set("clipboard_monitor", current);
    // Update State and Start Monitor
    let state = app.state::<ClipboardMonitorEnableWrapper>();
    state
        .0
        .lock()
        .unwrap()
        .replace_range(.., &current.to_string());
    if current {
        start_clipboard_monitor(app.clone());
    }
    // Update Tray Menu Status
    update_tray(app.clone(), "".to_string(), "".to_string());
}
fn on_auto_copy_click(app: &AppHandle, mode: &str) {
    info!("Set copy mode to: {}", mode);
    set("translate_auto_copy", mode);
    app.emit("translate_auto_copy_changed", mode).unwrap();
    update_tray(app.clone(), "".to_string(), mode.to_string());
}
fn on_ocr_recognize_click() {
    ocr_recognize();
}
fn on_ocr_translate_click() {
    ocr_translate();
}

fn on_config_click() {
    config_window();
}

fn on_check_update_click() {
    updater_window();
}
fn on_view_log_click(app: &AppHandle) {
    let log_path = app.path().app_log_dir().unwrap();
    app.shell().open(log_path.to_string_lossy(), None).unwrap();
}
fn on_restart_click(app: &AppHandle) {
    info!("============== Restart App ==============");
    app.restart();
}
fn on_quit_click(app: &AppHandle) {
    let _ = app.global_shortcut().unregister_all();
    info!("============== Quit App ==============");
    app.exit(0);
}
