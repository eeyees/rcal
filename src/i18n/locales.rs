use super::Translations;
use chrono::{Datelike, NaiveDate};

fn format_cjk_month_year(date: NaiveDate) -> String {
    format!("{}年 {:02}月", date.year(), date.month())
}

fn format_cjk_date_long(date: NaiveDate) -> String {
    format!("{}年{}月{}日", date.year(), date.month(), date.day())
}

fn format_iso_month_year(date: NaiveDate) -> String {
    date.format("%B %Y").to_string()
}

fn format_iso_date_long(date: NaiveDate) -> String {
    date.format("%A, %B %d, %Y").to_string()
}

fn format_mn_month_year(date: NaiveDate) -> String {
    format!("{} оны {:02} сар", date.year(), date.month())
}

fn format_mn_date_long(date: NaiveDate) -> String {
    format!("{}-{:02}-{:02}", date.year(), date.month(), date.day())
}

fn msg_sidebar_cjk(shown: bool) -> &'static str {
    if shown {
        "显示侧边栏"
    } else {
        "隐藏侧边栏"
    }
}

fn msg_sidebar_cjkt(shown: bool) -> &'static str {
    if shown {
        "顯示側邊欄"
    } else {
        "隱藏側邊欄"
    }
}

fn msg_sidebar_default(shown: bool) -> &'static str {
    if shown {
        "Sidebar shown"
    } else {
        "Sidebar hidden"
    }
}

fn msg_jump_today_def(date_str: &str) -> String {
    format!("Jumped to today ({})", date_str)
}

fn msg_lang_changed_def(name: &str) -> String {
    format!("Language changed to '{}'.", name)
}

fn msg_week_start_changed_def(label: &str) -> String {
    format!("Week start day set to '{}'.", label)
}

fn msg_account_updated_def(name: &str) -> String {
    format!("Account '{}' updated.", name)
}

fn msg_account_added_def(name: &str) -> String {
    format!("Added account '{}'", name)
}

fn msg_calendar_created_def(name: &str) -> String {
    format!("Created calendar '{}'", name)
}

fn msg_calendar_deleted_name_def(name: &str) -> String {
    format!("Deleted calendar '{}'", name)
}

fn more_events_def(count: usize) -> String {
    format!(" +{} more", count)
}

fn more_events_zh(count: usize) -> String {
    format!(" +{} 更多", count)
}

// -------------------------------------------------------------
// Macro for defining a locale cleanly
// -------------------------------------------------------------
macro_rules! define_locale {
    (
        $name:ident,
        code: $code:expr,
        label: $label:expr,
        short_label: $short_label:expr,
        views: [$v_m:expr, $v_w:expr, $v_d:expr, $v_a:expr],
        buttons: [$b_today:expr, $b_new:expr, $b_sync:expr, $b_syncing:expr, $b_accounts:expr, $b_settings:expr, $b_help:expr, $b_save:expr, $b_cancel:expr, $b_edit:expr, $b_delete:expr, $b_close:expr],
        headers: [$h0:expr, $h1:expr, $h2:expr, $h3:expr, $h4:expr, $h5:expr, $h6:expr],
        week_start_labels: [$w0:expr, $w1:expr, $w2:expr],
        fmt_my: $fmt_my:expr,
        fmt_dl: $fmt_dl:expr,
        sidebar: [$sb_cal:expr, $sb_manage:expr, $sb_hide:expr],
        footer: [$f_view:expr, $f_new:expr, $f_sync:expr, $f_lang:expr, $f_accounts:expr, $f_help:expr, $f_quit:expr],
        modals: [$m_new:expr, $m_edit:expr, $m_details:expr, $m_cal:expr, $m_acc:expr, $m_set:expr, $m_help:expr],
        fields: [$f_title:expr, $f_cal:expr, $f_date:expr, $f_start:expr, $f_end:expr, $f_all_day:expr, $f_loc:expr, $f_desc:expr],
        placeholders: [$p_title:expr, $p_loc:expr, $p_desc:expr],
        settings_titles: [$st_week:expr, $st_theme:expr, $st_lang:expr],
        all_day: $all_day:expr,
        more_events: $more_events:expr,
        status: [$msg_wel:expr, $msg_sav:expr, $msg_del:expr, $msg_acc_emp:expr, $msg_acc_rem:expr, $msg_cal_del:expr, $msg_cal_tog:expr],
        msg_sidebar: $msg_sidebar:expr
    ) => {
        pub static $name: Translations = Translations {
            code: $code,
            label: $label,
            short_label: $short_label,
            view_month: $v_m,
            view_week: $v_w,
            view_day: $v_d,
            view_agenda: $v_a,
            btn_today: $b_today,
            btn_new_event: $b_new,
            btn_sync: $b_sync,
            btn_syncing: $b_syncing,
            btn_accounts: $b_accounts,
            btn_settings: $b_settings,
            btn_help: $b_help,
            btn_save: $b_save,
            btn_cancel: $b_cancel,
            btn_edit: $b_edit,
            btn_delete: $b_delete,
            btn_close: $b_close,
            weekday_headers: [$h0, $h1, $h2, $h3, $h4, $h5, $h6],
            week_start_sun_headers: [$h0, $h1, $h2, $h3, $h4, $h5, $h6],
            week_start_mon_headers: [$h1, $h2, $h3, $h4, $h5, $h6, $h0],
            week_start_sat_headers: [$h6, $h0, $h1, $h2, $h3, $h4, $h5],
            week_start_labels: [$w0, $w1, $w2],
            format_month_year: $fmt_my,
            format_date_long: $fmt_dl,
            sidebar_calendars: $sb_cal,
            sidebar_manage: $sb_manage,
            sidebar_hide: $sb_hide,
            sidebar_quick_nav: "Quick Navigation:",
            sidebar_tips: &[
                " • Click tab: switch view",
                " • Click slot: add event",
                " • Click event: details/edit",
                " • [x] Checkbox: toggle cal",
                " • 'T' key: select theme",
                " • 'L' key: toggle language",
            ],
            sidebar_keys: "Keys:",
            sidebar_key_tips: &[
                " • b: Toggle sidebar",
                " • v/C: Select calendars",
                " • h/j/k/l: Navigate",
                " • c/n: New event",
                " • r: Sync now",
                " • T: Select theme",
                " • A: Accounts modal",
                " • t: Jump to today",
                " • q: Close / Quit",
            ],
            footer_keys_label: "Keys:",
            footer_view: $f_view,
            footer_sidebar: "Sidebar",
            footer_new: $f_new,
            footer_sync: $f_sync,
            footer_lang: $f_lang,
            footer_accounts: $f_accounts,
            footer_help: $f_help,
            footer_quit: $f_quit,
            modal_new_event: $m_new,
            modal_edit_event: $m_edit,
            modal_event_details: $m_details,
            field_title: $f_title,
            field_calendar: $f_cal,
            field_date: $f_date,
            field_start_time: $f_start,
            field_end_time: $f_end,
            field_all_day: $f_all_day,
            field_location: $f_loc,
            field_description: $f_desc,
            placeholder_title: $p_title,
            placeholder_location: $p_loc,
            placeholder_description: $p_desc,
            cycle_hint: "(Space to cycle)",
            yes_label: "[x] Yes",
            no_label: "[ ] No",
            no_location: "No location specified",
            no_description: "No description",
            modal_cal_create_title: $m_cal,
            modal_cal_manage_title: " Manage Calendars & Categories ",
            cal_name_label: " Name:          ",
            cal_name_placeholder: "(e.g. Work, Study, Fitness)",
            cal_colors: ["Blue", "Green", "Yellow", "Red", "Purple", "Cyan", "Orange"],
            cal_list_header: "Registered Calendars:",
            cal_status_visible: "[Visible]",
            cal_status_hidden: "[Hidden]",
            cal_default_badge: "(Default)",
            cal_btn_create: " [ + Create Calendar ] ",
            cal_key_hint: " [Space] Toggle  [1~9] Select num  [s/Enter] Save  [n] Add  [d] Delete  [Esc] Cancel ",
            modal_account_add_title: $m_acc,
            modal_account_manage_title: " Calendar Accounts & Sync ",
            account_list_header: " 📋 Configured Sync Accounts:",
            account_empty_msg: "  No cloud accounts linked yet.",
            account_never_synced: "Never",
            account_btn_add: " [ + Add Account (a/+) ] ",
            account_btn_sync_now: " [ ⟳ Sync Now ] ",
            modal_settings_title: $m_set,
            settings_week_start_title: $st_week,
            settings_week_start_desc: "  Choose which day starts the week:",
            settings_theme_title: $st_theme,
            settings_theme_desc: "  Click any theme or press [T] / [c] to cycle:",
            settings_lang_title: $st_lang,
            settings_hint_shortcuts: "Shortcuts: ",
            settings_hint_day_select: " Select Day  ",
            settings_hint_cycle: " Cycle  ",
            settings_hint_lang: " Language",
            settings_btn_close: "[ Close (Esc / Enter) ]",
            modal_help_title: $m_help,
            help_mouse_title: "🖱  Mouse Controls:",
            help_mouse_lines: &[
                "  • Click View Tabs       : Switch between views",
                "  • Click [<] / [>]       : Navigate backward / forward",
                "  • Click [Today]         : Jump straight to today",
                "  • Click Day Cell        : Create new event on that date",
                "  • Click Time Slot       : Create new event at that hour",
                "  • Click [x] Checkbox    : Toggle calendar visibility",
            ],
            help_keyboard_title: "⌨  Keyboard Shortcuts:",
            help_keyboard_lines: &[
                "  • 1 / 2 / 3 / 4         : Month / Week / Day / Agenda view",
                "  • b                     : Toggle left sidebar visibility",
                "  • v / C                 : Select & manage calendars (toggle visibility & save)",
                "  • h / j / k / l         : Navigate days / weeks / hours",
                "  • t                     : Jump to today",
                "  • c / n                 : Create new event",
                "  • r                     : Cloud sync now",
                "  • T                     : Open theme selection modal",
                "  • A                     : Accounts modal",
                "  • C                     : Manage calendars",
                "  • L                     : Toggle display language",
                "  • ?                     : Show help screen",
                "  • q / Esc               : Close modal / Quit",
            ],
            day_view_hint: "(Click any time slot to add an event)",
            all_day_label: $all_day,
            all_day_short: $all_day,
            add_all_day_tip: " + Click to add all-day event...",
            more_events: $more_events,
            msg_welcome: $msg_wel,
            msg_saved: $msg_sav,
            msg_deleted: $msg_del,
            msg_jump_today: msg_jump_today_def,
            msg_lang_changed: msg_lang_changed_def,
            msg_sidebar_toggle: $msg_sidebar,
            msg_week_start_changed: msg_week_start_changed_def,
            msg_account_username_empty: $msg_acc_emp,
            msg_account_updated: msg_account_updated_def,
            msg_account_added: msg_account_added_def,
            msg_account_removed: $msg_acc_rem,
            msg_calendar_created: msg_calendar_created_def,
            msg_calendar_deleted: $msg_cal_del,
            msg_calendar_deleted_name: msg_calendar_deleted_name_def,
            msg_calendar_toggled: $msg_cal_tog,
        };
    };
}

// 1. Chinese Simplified (zh-cn) - 简体中文
define_locale!(
    ZH_CN,
    code: "zh-cn",
    label: "简体中文",
    short_label: "简",
    views: ["1:月视图", "2:周视图", "3:日视图", "4:日程列表"],
    buttons: ["今天", "+ 新建", "⟳ 同步", "⟳ 同步中", "☁ 账户", "⚙ 设置", "? 帮助", "[ 保存 ]", "[ 取消 ]", "[ 编辑 (e) ]", "[ 删除 (d) ]", "[ 关闭 (q) ]"],
    headers: ["日", "一", "二", "三", "四", "五", "六"],
    week_start_labels: ["周日开始", "周一开始", "周六开始"],
    fmt_my: format_cjk_month_year,
    fmt_dl: format_cjk_date_long,
    sidebar: [" 我的日历 ", " + 管理日历 (C) ", "[ 隐藏 (b) ]"],
    footer: ["视图", "新建", "同步", "语言", "账户", "帮助", "退出"],
    modals: [" 新建日程 ", " 编辑日程 ", " 日程详情 ", " 创建新日历 ", " 添加账户 ", " ⚙ 设置与主题 ", " 键盘与鼠标快捷键 "],
    fields: [" 标题:       ", " 日历:       ", " 日期:       ", " 开始时间:   ", " 结束时间:   ", " 全天日程:   ", " 地点:       ", " 描述:       "],
    placeholders: ["(输入日程标题...)", "(可选地点)", "(可选备注)"],
    settings_titles: ["▶ 1. 一周首日", "▶ 2. 界面主题", "▶ 3. 显示语言"],
    all_day: "全天",
    more_events: more_events_zh,
    status: ["欢迎使用 rcal！按 ? 查看帮助。", "日程已保存。", "日程已删除。", "用户名不能为空。", "账户已移除。", "日历已删除。", "日历可见性已切换"],
    msg_sidebar: msg_sidebar_cjk
);

// 2. Chinese Traditional (zh-tw) - 繁體中文
define_locale!(
    ZH_TW,
    code: "zh-tw",
    label: "繁體中文",
    short_label: "繁",
    views: ["1:月檢視", "2:週檢視", "3:日檢視", "4:待辦日程"],
    buttons: ["今天", "+ 新增", "⟳ 同步", "⟳ 同步中", "☁ 帳號", "⚙ 設定", "? 說明", "[ 儲存 ]", "[ 取消 ]", "[ 編輯 (e) ]", "[ 刪除 (d) ]", "[ 關閉 (q) ]"],
    headers: ["日", "一", "二", "三", "四", "五", "六"],
    week_start_labels: ["週日開始", "週一开始", "週六開始"],
    fmt_my: format_cjk_month_year,
    fmt_dl: format_cjk_date_long,
    sidebar: [" 我的日曆 ", " + 管理日曆 (C) ", "[ 隱藏 (b) ]"],
    footer: ["檢視", "新增", "同步", "語言", "帳號", "說明", "離開"],
    modals: [" 新增日程 ", " 編輯日程 ", " 日程詳情 ", " 建立新日曆 ", " 新增帳號 ", " ⚙ 設定與佈景主題 ", " 鍵盤與滑鼠快捷鍵 "],
    fields: [" 標題:       ", " 日曆:       ", " 日期:       ", " 開始時間:   ", " 結束時間:   ", " 整天日程:   ", " 地點:       ", " 說明:       "],
    placeholders: ["(輸入日程標題...)", "(選填地點)", "(選填備註)"],
    settings_titles: ["▶ 1. 每週首日", "▶ 2. 佈景主題", "▶ 3. 顯示語言"],
    all_day: "整天",
    more_events: more_events_zh,
    status: ["歡迎使用 rcal！按 ? 查看說明。", "日程已儲存。", "日程已刪除。", "使用者名稱不能為空。", "帳號已移除。", "日曆已刪除。", "日曆可見性已切換"],
    msg_sidebar: msg_sidebar_cjkt
);

// 3. Mongolian (mn) - Монгол хэл
define_locale!(
    MN,
    code: "mn",
    label: "Монгол",
    short_label: "MN",
    views: ["1:Сар", "2:Долоо хоног", "3:Өдөр", "4:Төлөвлөгөө"],
    buttons: ["Өнөөдөр", "+ Шинэ", "⟳ Синк", "⟳ Синк...", "☁ Бүртгэл", "⚙ Тохиргоо", "? Тусламж", "[ Хадгалах ]", "[ Цуцлах ]", "[ Засах (e) ]", "[ Устгах (d) ]", "[ Хаах (q) ]"],
    headers: ["Ням", "Дав", "Мяг", "Лха", "Пүр", "Баа", "Бям"],
    week_start_labels: ["Ням гараг", "Даваа гараг", "Бямба гараг"],
    fmt_my: format_mn_month_year,
    fmt_dl: format_mn_date_long,
    sidebar: [" Миний хуанли ", " + Удирдах (C) ", "[ Нуух (b) ]"],
    footer: ["Харах", "Шинэ", "Синк", "Хэл", "Бүртгэл", "Тусламж", "Гарах"],
    modals: [" Шинэ үйл явдал ", " Засах ", " Дэлгэрэнгүй ", " Хуанли үүсгэх ", " Бүртгэл нэмэх ", " ⚙ Тохиргоо ба Загвар ", " Товчлуурын тусламж "],
    fields: [" Гарчиг:      ", " Хуанли:      ", " Огноо:       ", " Эхлэх цаг:   ", " Дуусах цаг:  ", " Бүтэн өдөр:  ", " Байршил:     ", " Тайлбар:     "],
    placeholders: ["(Гарчиг бичих...)", "(Байршил сонгох)", "(Тэмдэглэл бичих)"],
    settings_titles: ["▶ 1. Долоо хоногийн эхний өдөр", "▶ 2. Өнгөний загвар", "▶ 3. Системийн хэл"],
    all_day: "Бүтэн өдөр",
    more_events: more_events_def,
    status: ["rcal-д тавтай морил! ? дарж тусламж авна уу.", "Хадгалагдлаа.", "Устгагдлаа.", "Нэвтрэх нэр хоосон байж болохгүй.", "Бүртгэл устгагдлаа.", "Хуанли устгагдлаа.", "Төлөв шинэчлэгдлээ"],
    msg_sidebar: msg_sidebar_default
);

// 4. Spanish (es) - Español
define_locale!(
    ES,
    code: "es",
    label: "Español",
    short_label: "ES",
    views: ["1:Mes", "2:Semana", "3:Día", "4:Agenda"],
    buttons: ["Hoy", "+ Nuevo", "⟳ Sincronizar", "⟳ Sincronizando", "☁ Cuentas", "⚙ Ajustes", "? Ayuda", "[ Guardar ]", "[ Cancelar ]", "[ Editar (e) ]", "[ Eliminar (d) ]", "[ Cerrar (q) ]"],
    headers: ["DOM", "LUN", "MAR", "MIÉ", "JUE", "VIE", "SÁB"],
    week_start_labels: ["Inicio el domingo", "Inicio el lunes", "Inicio el sábado"],
    fmt_my: format_iso_month_year,
    fmt_dl: format_iso_date_long,
    sidebar: [" Mis Calendarios ", " + Gestionar (C) ", "[ Ocultar (b) ]"],
    footer: ["Vista", "Nuevo", "Sinc.", "Idioma", "Cuentas", "Ayuda", "Salir"],
    modals: [" Nuevo Evento ", " Editar Evento ", " Detalles ", " Crear Calendario ", " Añadir Cuenta ", " ⚙ Ajustes y Temas ", " Atajos de Teclado y Ratón "],
    fields: [" Título:       ", " Calendario:  ", " Fecha:       ", " Inicio:      ", " Fin:         ", " Todo el día: ", " Ubicación:   ", " Descripción: "],
    placeholders: ["(Título del evento...)", "(Ubicación opcional)", "(Notas opcionales)"],
    settings_titles: ["▶ 1. Primer día de la semana", "▶ 2. Tema de color", "▶ 3. Idioma"],
    all_day: "Todo el día",
    more_events: more_events_def,
    status: ["¡Bienvenido a rcal! Pulsa ? para ver los atajos.", "Evento guardado.", "Evento eliminado.", "El usuario no puede estar vacío.", "Cuenta eliminada.", "Calendario eliminado.", "Visibilidad cambiada"],
    msg_sidebar: msg_sidebar_default
);

// 5. French (fr) - Français
define_locale!(
    FR,
    code: "fr",
    label: "Français",
    short_label: "FR",
    views: ["1:Mois", "2:Semaine", "3:Jour", "4:Agenda"],
    buttons: ["Aujourd'hui", "+ Nouveau", "⟳ Synchro", "⟳ Synchro...", "☁ Comptes", "⚙ Réglages", "? Aide", "[ Enregistrer ]", "[ Annuler ]", "[ Modifier (e) ]", "[ Supprimer (d) ]", "[ Fermer (q) ]"],
    headers: ["DIM", "LUN", "MAR", "MER", "JEU", "VEN", "SAM"],
    week_start_labels: ["Début dimanche", "Début lundi", "Début samedi"],
    fmt_my: format_iso_month_year,
    fmt_dl: format_iso_date_long,
    sidebar: [" Mes Agendas ", " + Gérer (C) ", "[ Masquer (b) ]"],
    footer: ["Vue", "Nouveau", "Synchro", "Langue", "Comptes", "Aide", "Quitter"],
    modals: [" Nouvel Événement ", " Modifier Événement ", " Détails ", " Créer un Agenda ", " Ajouter Compte ", " ⚙ Réglages & Thèmes ", " Raccourcis Clavier & Souris "],
    fields: [" Titre:        ", " Agenda:       ", " Date:        ", " Début:       ", " Fin:         ", " Journée:     ", " Lieu:        ", " Description: "],
    placeholders: ["(Titre de l'événement...)", "(Lieu optionnel)", "(Notes)"],
    settings_titles: ["▶ 1. Premier jour de la semaine", "▶ 2. Thèmes de couleur", "▶ 3. Langue d'affichage"],
    all_day: "Toute la journée",
    more_events: more_events_def,
    status: ["Bienvenue sur rcal ! Appuyez sur ? pour l'aide.", "Événement enregistré.", "Événement supprimé.", "Nom d'utilisateur requis.", "Compte supprimé.", "Agenda supprimé.", "Visibilité modifiée"],
    msg_sidebar: msg_sidebar_default
);

// 6. German (de) - Deutsch
define_locale!(
    DE,
    code: "de",
    label: "Deutsch",
    short_label: "DE",
    views: ["1:Monat", "2:Woche", "3:Tag", "4:Agenda"],
    buttons: ["Heute", "+ Neu", "⟳ Sync", "⟳ Synchronisiere", "☁ Konten", "⚙ Optionen", "? Hilfe", "[ Speichern ]", "[ Abbrechen ]", "[ Bearbeiten (e) ]", "[ Löschen (d) ]", "[ Schließen (q) ]"],
    headers: ["SO", "MO", "DI", "MI", "DO", "FR", "SA"],
    week_start_labels: ["Sonntag zuerst", "Montag zuerst", "Samstag zuerst"],
    fmt_my: format_iso_month_year,
    fmt_dl: format_iso_date_long,
    sidebar: [" Meine Kalender ", " + Verwalten (C) ", "[ Verbergen (b) ]"],
    footer: ["Ansicht", "Neu", "Sync", "Sprache", "Konten", "Hilfe", "Beenden"],
    modals: [" Neuer Termin ", " Termin bearbeiten ", " Details ", " Kalender erstellen ", " Konto verbinden ", " ⚙ Einstellungen & Farbschemata ", " Tastatur & Maus Tastenkürzel "],
    fields: [" Titel:        ", " Kalender:     ", " Datum:        ", " Beginn:       ", " Ende:         ", " Ganztägig:    ", " Ort:          ", " Beschreibung: "],
    placeholders: ["(Titel eingeben...)", "(Optionaler Ort)", "(Notizen)"],
    settings_titles: ["▶ 1. Erster Wochentag", "▶ 2. Farbschema", "▶ 3. Anzeigesprache"],
    all_day: "Ganztägig",
    more_events: more_events_def,
    status: ["Willkommen bei rcal! Drücke ? für Hilfe.", "Termin gespeichert.", "Termin gelöscht.", "Benutzername darf nicht leer sein.", "Konto entfernt.", "Kalender gelöscht.", "Sichtbarkeit umgeschaltet"],
    msg_sidebar: msg_sidebar_default
);

// 7. Russian (ru) - Русский
define_locale!(
    RU,
    code: "ru",
    label: "Русский",
    short_label: "RU",
    views: ["1:Месяц", "2:Неделя", "3:День", "4:Повестка"],
    buttons: ["Сегодня", "+ Создать", "⟳ Синхр.", "⟳ Синхронизация", "☁ Аккаунты", "⚙ Опции", "? Справка", "[ Сохранить ]", "[ Отмена ]", "[ Изменить (e) ]", "[ Удалить (d) ]", "[ Закрыть (q) ]"],
    headers: ["ВС", "ПН", "ВТ", "СР", "ЧТ", "ПТ", "СБ"],
    week_start_labels: ["Воскресенье", "Понедельник", "Суббота"],
    fmt_my: format_iso_month_year,
    fmt_dl: format_iso_date_long,
    sidebar: [" Мои календари ", " + Управление (C) ", "[ Скрыть (b) ]"],
    footer: ["Вид", "Создать", "Синхр", "Язык", "Аккаунты", "Справка", "Выход"],
    modals: [" Новое событие ", " Редактировать ", " Детали события ", " Создать календарь ", " Добавить аккаунт ", " ⚙ Настройки и Темы ", " Горячие клавиши "],
    fields: [" Заголовок:   ", " Календарь:   ", " Дата:        ", " Начало:      ", " Окончание:   ", " Весь день:   ", " Место:       ", " Описание:    "],
    placeholders: ["(Название события...)", "(Место проведения)", "(Заметки)"],
    settings_titles: ["▶ 1. Первый день недели", "▶ 2. Цветовая тема", "▶ 3. Язык интерфейса"],
    all_day: "Весь день",
    more_events: more_events_def,
    status: ["Добро пожаловать в rcal! Нажмите ? для справки.", "Событие сохранено.", "Событие удалено.", "Имя пользователя не может быть пустым.", "Аккаунт удален.", "Календарь удален.", "Видимость переключена"],
    msg_sidebar: msg_sidebar_default
);

// 8. Portuguese (pt) - Português
define_locale!(
    PT,
    code: "pt",
    label: "Português",
    short_label: "PT",
    views: ["1:Mês", "2:Semana", "3:Dia", "4:Agenda"],
    buttons: ["Hoje", "+ Novo", "⟳ Sincronizar", "⟳ Sincronizando", "☁ Contas", "⚙ Ajustes", "? Ajuda", "[ Salvar ]", "[ Cancelar ]", "[ Editar (e) ]", "[ Excluir (d) ]", "[ Fechar (q) ]"],
    headers: ["DOM", "SEG", "TER", "QUA", "QUI", "SEX", "SÁB"],
    week_start_labels: ["Início no domingo", "Início na segunda", "Início no sábado"],
    fmt_my: format_iso_month_year,
    fmt_dl: format_iso_date_long,
    sidebar: [" Meus Calendários ", " + Gerenciar (C) ", "[ Ocultar (b) ]"],
    footer: ["Visão", "Novo", "Sinc", "Idioma", "Contas", "Ajuda", "Sair"],
    modals: [" Novo Evento ", " Editar Evento ", " Detalhes ", " Criar Calendário ", " Adicionar Conta ", " ⚙ Configurações & Temas ", " Atalhos de Teclado & Mouse "],
    fields: [" Título:       ", " Calendário:  ", " Data:        ", " Início:      ", " Fim:         ", " Dia inteiro: ", " Local:       ", " Descrição:   "],
    placeholders: ["(Título do evento...)", "(Local opcional)", "(Observações)"],
    settings_titles: ["▶ 1. Primeiro dia da semana", "▶ 2. Temas de cores", "▶ 3. Idioma de exibição"],
    all_day: "Dia inteiro",
    more_events: more_events_def,
    status: ["Bem-vindo ao rcal! Pressione ? para ajuda.", "Evento salvo com sucesso.", "Evento excluído.", "Usuário não pode estar vazio.", "Conta removida.", "Calendário excluído.", "Visibilidade alternada"],
    msg_sidebar: msg_sidebar_default
);

// 9. Italian (it) - Italiano
define_locale!(
    IT,
    code: "it",
    label: "Italiano",
    short_label: "IT",
    views: ["1:Mese", "2:Settimana", "3:Giorno", "4:Agenda"],
    buttons: ["Oggi", "+ Nuovo", "⟳ Sincronizza", "⟳ Sincronizzazione", "☁ Account", "⚙ Impostazioni", "? Guida", "[ Salva ]", "[ Annulla ]", "[ Modifica (e) ]", "[ Elimina (d) ]", "[ Chiudi (q) ]"],
    headers: ["DOM", "LUN", "MAR", "MER", "GIO", "VEN", "SAB"],
    week_start_labels: ["Inizia domenica", "Inizia lunedì", "Inizia sabato"],
    fmt_my: format_iso_month_year,
    fmt_dl: format_iso_date_long,
    sidebar: [" I miei calendari ", " + Gestisci (C) ", "[ Nascondi (b) ]"],
    footer: ["Vista", "Nuovo", "Sinc", "Lingua", "Account", "Guida", "Esci"],
    modals: [" Nuovo Evento ", " Modifica Evento ", " Dettagli ", " Crea Calendario ", " Aggiungi Account ", " ⚙ Impostazioni e Temi ", " Scorciatoie Tastiera & Mouse "],
    fields: [" Titolo:       ", " Calendario:  ", " Data:        ", " Inizio:      ", " Fine:        ", " Tutto il gg: ", " Luogo:       ", " Descrizione: "],
    placeholders: ["(Titolo evento...)", "(Luogo facoltativo)", "(Note)"],
    settings_titles: ["▶ 1. Primo giorno della settimana", "▶ 2. Temi di colore", "▶ 3. Lingua"],
    all_day: "Tutto il giorno",
    more_events: more_events_def,
    status: ["Benvenuto in rcal! Premi ? per la guida.", "Evento salvato.", "Evento eliminato.", "Il nome utente è obbligatorio.", "Account rimosso.", "Calendario eliminato.", "Visibilità modificata"],
    msg_sidebar: msg_sidebar_default
);

// 10. Vietnamese (vi) - Tiếng Việt
define_locale!(
    VI,
    code: "vi",
    label: "Tiếng Việt",
    short_label: "VI",
    views: ["1:Tháng", "2:Tuần", "3:Ngày", "4:Lịch trình"],
    buttons: ["Hôm nay", "+ Tạo mới", "⟳ Đồng bộ", "⟳ Đang đồng bộ", "☁ Tài khoản", "⚙ Cài đặt", "? Trợ giúp", "[ Lưu ]", "[ Hủy ]", "[ Sửa (e) ]", "[ Xóa (d) ]", "[ Đóng (q) ]"],
    headers: ["CN", "T2", "T3", "T4", "T5", "T6", "T7"],
    week_start_labels: ["Bắt đầu Chủ Nhật", "Bắt đầu Thứ Hai", "Bắt đầu Thứ Bảy"],
    fmt_my: format_iso_month_year,
    fmt_dl: format_iso_date_long,
    sidebar: [" Lịch của tôi ", " + Quản lý (C) ", "[ Ẩn (b) ]"],
    footer: ["Xem", "Mới", "Đồng bộ", "Ngôn ngữ", "Tài khoản", "Giúp", "Thoát"],
    modals: [" Sự kiện mới ", " Chỉnh sửa ", " Chi tiết sự kiện ", " Tạo lịch mới ", " Thêm tài khoản ", " ⚙ Cài đặt & Giao diện ", " Phím tắt & Thao tác chuột "],
    fields: [" Tiêu đề:     ", " Lịch:        ", " Ngày:        ", " Bắt đầu:     ", " Kết thúc:    ", " Cả ngày:     ", " Địa điểm:    ", " Mô tả:       "],
    placeholders: ["(Nhập tiêu đề...)", "(Địa điểm)", "(Ghi chú)"],
    settings_titles: ["▶ 1. Ngày đầu tuần", "▶ 2. Giao diện màu sắc", "▶ 3. Ngôn ngữ hiển thị"],
    all_day: "Cả ngày",
    more_events: more_events_def,
    status: ["Chào mừng đến rcal! Nhấn ? để xem trợ giúp.", "Đã lưu sự kiện.", "Đã xóa sự kiện.", "Tên người dùng không được để trống.", "Đã xóa tài khoản.", "Đã xóa lịch.", "Đã thay đổi hiển thị"],
    msg_sidebar: msg_sidebar_default
);

// 11. Indonesian (id) - Bahasa Indonesia
define_locale!(
    ID,
    code: "id",
    label: "Bahasa Indonesia",
    short_label: "ID",
    views: ["1:Bulan", "2:Minggu", "3:Hari", "4:Agenda"],
    buttons: ["Hari Ini", "+ Baru", "⟳ Sinkron", "⟳ Menyinkronkan", "☁ Akun", "⚙ Pengaturan", "? Bantuan", "[ Simpan ]", "[ Batal ]", "[ Ubah (e) ]", "[ Hapus (d) ]", "[ Tutup (q) ]"],
    headers: ["MIN", "SEN", "SEL", "RAB", "KAM", "JUM", "SAB"],
    week_start_labels: ["Mulai Minggu", "Mulai Senin", "Mulai Sabtu"],
    fmt_my: format_iso_month_year,
    fmt_dl: format_iso_date_long,
    sidebar: [" Kalender Saya ", " + Kelola (C) ", "[ Sembunyi (b) ]"],
    footer: ["Tampilan", "Baru", "Sinkron", "Bahasa", "Akun", "Bantuan", "Keluar"],
    modals: [" Acara Baru ", " Ubah Acara ", " Detail Acara ", " Buat Kalender ", " Tambah Akun ", " ⚙ Pengaturan & Tema ", " Pintasan Keyboard & Mouse "],
    fields: [" Judul:       ", " Kalender:    ", " Tanggal:     ", " Mulai:       ", " Selesai:     ", " Sepanjang hr:", " Lokasi:      ", " Deskripsi:   "],
    placeholders: ["(Ketik judul acara...)", "(Lokasi opsional)", "(Catatan)"],
    settings_titles: ["▶ 1. Hari Pertama Mingguan", "▶ 2. Tema Warna", "▶ 3. Bahasa Tampilan"],
    all_day: "Sepanjang hari",
    more_events: more_events_def,
    status: ["Selamat datang di rcal! Tekan ? untuk bantuan.", "Acara berhasil disimpan.", "Acara dihapus.", "Nama pengguna tidak boleh kosong.", "Akun dihapus.", "Kalender dihapus.", "Visibilitas kalender diubah"],
    msg_sidebar: msg_sidebar_default
);

// 12. Thai (th) - ไทย
define_locale!(
    TH,
    code: "th",
    label: "ไทย",
    short_label: "TH",
    views: ["1:เดือน", "2:สัปดาห์", "3:วัน", "4:กำหนดการ"],
    buttons: ["วันนี้", "+ ใหม่", "⟳ ซิงค์", "⟳ ซิงค์...", "☁ บัญชี", "⚙ ตั้งค่า", "? ช่วยเหลือ", "[ บันทึก ]", "[ ยกเลิก ]", "[ แก้ไข (e) ]", "[ ลบ (d) ]", "[ ปิด (q) ]"],
    headers: ["อา.", "จ.", "อ.", "พ.", "พฤ.", "ศ.", "ส."],
    week_start_labels: ["เริ่มวันอาทิตย์", "เริ่มวันจันทร์", "เริ่มวันเสาร์"],
    fmt_my: format_iso_month_year,
    fmt_dl: format_iso_date_long,
    sidebar: [" ปฏิทินของฉัน ", " + จัดการ (C) ", "[ ซ่อน (b) ]"],
    footer: ["มุมมอง", "ใหม่", "ซิงค์", "ภาษา", "บัญชี", "ช่วย", "ออก"],
    modals: [" กิจกรรมใหม่ ", " แก้ไขกิจกรรม ", " รายละเอียด ", " สร้างปฏิทินใหม่ ", " เพิ่มบัญชี ", " ⚙ ตั้งค่า & ธีมสี ", " แป้นพิมพ์ลัด & เมาส์ "],
    fields: [" ชื่อเรื่อง:   ", " ปฏิทิน:      ", " วันที่:       ", " เวลาเริ่ม:   ", " สิ้นสุด:     ", " ตลอดวัน:     ", " สถานที่:     ", " รายละเอียด:  "],
    placeholders: ["(ระบุชื่อกิจกรรม...)", "(สถานที่)", "(บันทึกย่อ)"],
    settings_titles: ["▶ 1. วันเริ่มต้นสัปดาห์", "▶ 2. ธีมสี", "▶ 3. ภาษาที่แสดงผล"],
    all_day: "ตลอดทั้งวัน",
    more_events: more_events_def,
    status: ["ยินดีต้อนรับสู่ rcal! กด ? เพื่อดูปุ่มลัด", "บันทึกกิจกรรมแล้ว", "ลบกิจกรรมแล้ว", "ชื่อผู้ใช้ต้องไม่ว่างเปล่า", "ลบบัญชีแล้ว", "ลบปฏิทินแล้ว", "สลับการมองเห็นปฏิทิน"],
    msg_sidebar: msg_sidebar_default
);

// 13. Arabic (ar) - العربية
define_locale!(
    AR,
    code: "ar",
    label: "العربية",
    short_label: "AR",
    views: ["1:شهر", "2:أسبوع", "3:يوم", "4:جدول"],
    buttons: ["اليوم", "+ جديد", "⟳ مزامنة", "⟳ جاري...", "☁ الحسابات", "⚙ إعدادات", "? مساعدة", "[ حفظ ]", "[ إلغاء ]", "[ تعديل (e) ]", "[ حذف (d) ]", "[ إغلاق (q) ]"],
    headers: ["أحد", "إثنين", "ثلاثاء", "أربعاء", "خميس", "جمعة", "سبت"],
    week_start_labels: ["يبدأ الأحد", "يبدأ الإثنين", "يبدأ السبت"],
    fmt_my: format_iso_month_year,
    fmt_dl: format_iso_date_long,
    sidebar: [" تقويماتي ", " + إدارة (C) ", "[ إخفاء (b) ]"],
    footer: ["عرض", "جديد", "مزامنة", "لغة", "حسابات", "مساعدة", "خروج"],
    modals: [" حدث جديد ", " تعديل الحدث ", " تفاصيل الحدث ", " إنشاء تقويم ", " إضافة حساب ", " ⚙ الإعدادات والمظهر ", " اختصارات لوحة المفاتيح "],
    fields: [" العنوان:     ", " التقويم:     ", " التاريخ:     ", " البدء:       ", " الانتهاء:    ", " طوال اليوم:  ", " الموقع:      ", " الوصف:       "],
    placeholders: ["(عنوان الحدث...)", "(الموقع الاختياري)", "(ملاحظات)"],
    settings_titles: ["▶ 1. أول أيام الأسبوع", "▶ 2. سمات الألوان", "▶ 3. لغة العرض"],
    all_day: "طوال اليوم",
    more_events: more_events_def,
    status: ["مرحباً بك في rcal! اضغط ? للمساعدة.", "تم حفظ الحدث.", "تم حذف الحدث.", "اسم المستخدم مطلوب.", "تمت إزالة الحساب.", "تم حذف التقويم.", "تم تغيير الظهور"],
    msg_sidebar: msg_sidebar_default
);

// 14. Hindi (hi) - हिन्दी
define_locale!(
    HI,
    code: "hi",
    label: "हिन्दी",
    short_label: "HI",
    views: ["1:माह", "2:सप्ताह", "3:दिन", "4:कार्यसूची"],
    buttons: ["आज", "+ नया", "⟳ सिंक", "⟳ सिंक...", "☁ खाते", "⚙ सेटिंग्स", "? सहायता", "[ सहेजें ]", "[ रद्द करें ]", "[ संपादित (e) ]", "[ हटाएं (d) ]", "[ बंद करें (q) ]"],
    headers: ["रवि", "सोम", "मंगल", "बुध", "गुरु", "शुक्र", "शनि"],
    week_start_labels: ["रविवार से शुरू", "सोमवार से शुरू", "शनिवार से शुरू"],
    fmt_my: format_iso_month_year,
    fmt_dl: format_iso_date_long,
    sidebar: [" मेरे कैलेंडर ", " + प्रबंधन (C) ", "[ छिपाएं (b) ]"],
    footer: ["दृश्य", "नया", "सिंक", "भाषा", "खाते", "मदद", "निकास"],
    modals: [" नया इवेंट ", " इवेंट संपादित करें ", " विवरण ", " नया कैलेंडर ", " खाता जोड़ें ", " ⚙ सेटिंग्स और थीम्स ", " कीबोर्ड शॉर्टकट "],
    fields: [" शीर्षक:       ", " कैलेंडर:     ", " तारीख:       ", " प्रारंभ:     ", " समाप्ति:     ", " पूरे दिन:    ", " स्थान:       ", " विवरण:       "],
    placeholders: ["(इवेंट का नाम...)", "(स्थान)", "(विवरण)"],
    settings_titles: ["▶ 1. सप्ताह का पहला दिन", "▶ 2. रंग थीम", "▶ 3. भाषा चुनें"],
    all_day: "पूरा दिन",
    more_events: more_events_def,
    status: ["rcal में आपका स्वागत है! शॉर्टकट के लिए ? दबाएं।", "सफलतापूर्वक सहेजा गया।", "हटा दिया गया।", "यूज़रनेम आवश्यक है।", "खाता हटा दिया गया।", "कैलेंडर हटा दिया गया।", "दृश्यता बदली गई"],
    msg_sidebar: msg_sidebar_default
);

// 15. Turkish (tr) - Türkçe
define_locale!(
    TR,
    code: "tr",
    label: "Türkçe",
    short_label: "TR",
    views: ["1:Ay", "2:Hafta", "3:Gün", "4:Gündem"],
    buttons: ["Bugün", "+ Yeni", "⟳ Eşitle", "⟳ Eşitleniyor", "☁ Hesaplar", "⚙ Ayarlar", "? Yardım", "[ Kaydet ]", "[ İptal ]", "[ Düzenle (e) ]", "[ Sil (d) ]", "[ Kapat (q) ]"],
    headers: ["PAZ", "PZT", "SAL", "ÇAR", "PER", "CUM", "CTS"],
    week_start_labels: ["Pazar başlar", "Pazartesi başlar", "Cumartesi başlar"],
    fmt_my: format_iso_month_year,
    fmt_dl: format_iso_date_long,
    sidebar: [" Takvimlerim ", " + Yönet (C) ", "[ Gizle (b) ]"],
    footer: ["Görünüm", "Yeni", "Eşitle", "Dil", "Hesaplar", "Yardım", "Çıkış"],
    modals: [" Yeni Etkinlik ", " Etkinliği Düzenle ", " Ayrıntılar ", " Takvim Oluştur ", " Hesap Ekle ", " ⚙ Ayarlar ve Temalar ", " Klavye & Fare Kısayolları "],
    fields: [" Başlık:       ", " Takvim:      ", " Tarih:       ", " Başlangıç:   ", " Bitiş:       ", " Tüm Gün:     ", " Konum:       ", " Açıklama:    "],
    placeholders: ["(Etkinlik başlığı...)", "(İsteğe bağlı konum)", "(Notlar)"],
    settings_titles: ["▶ 1. Haftanın İlk Günü", "▶ 2. Renk Temaları", "▶ 3. Görüntüleme Dili"],
    all_day: "Tüm gün",
    more_events: more_events_def,
    status: ["rcal'e hoş geldiniz! Kısayollar için ? tuşuna basın.", "Etkinlik kaydedildi.", "Etkinlik silindi.", "Kullanıcı adı boş bırakılamaz.", "Hesap kaldırıldı.", "Takvim silindi.", "Görünürlük değiştirildi"],
    msg_sidebar: msg_sidebar_default
);

// 16. Dutch (nl) - Nederlands
define_locale!(
    NL,
    code: "nl",
    label: "Nederlands",
    short_label: "NL",
    views: ["1:Maand", "2:Week", "3:Dag", "4:Agenda"],
    buttons: ["Vandaag", "+ Nieuw", "⟳ Sync", "⟳ Synchroniseren", "☁ Accounts", "⚙ Opties", "? Help", "[ Opslaan ]", "[ Annuleren ]", "[ Bewerken (e) ]", "[ Verwijderen (d) ]", "[ Sluiten (q) ]"],
    headers: ["ZO", "MA", "DI", "WO", "DO", "VR", "ZA"],
    week_start_labels: ["Zondag eerst", "Maandag eerst", "Zaterdag eerst"],
    fmt_my: format_iso_month_year,
    fmt_dl: format_iso_date_long,
    sidebar: [" Mijn Agenda's ", " + Beheren (C) ", "[ Verbergen (b) ]"],
    footer: ["Weergave", "Nieuw", "Sync", "Taal", "Accounts", "Help", "Afsluiten"],
    modals: [" Nieuwe Afspraak ", " Afspraak Bewerken ", " Details ", " Agenda Aanmaken ", " Account Koppelen ", " ⚙ Instellingen & Thema's ", " Sneltoetsen "],
    fields: [" Titel:        ", " Agenda:       ", " Datum:        ", " Starttijd:    ", " Eindtijd:     ", " Hele dag:     ", " Locatie:      ", " Beschrijving: "],
    placeholders: ["(Titel invoeren...)", "(Optionele locatie)", "(Notities)"],
    settings_titles: ["▶ 1. Eerste dag van de week", "▶ 2. Kleurenthema", "▶ 3. Taal"],
    all_day: "Hele dag",
    more_events: more_events_def,
    status: ["Welkom bij rcal! Druk op ? voor help.", "Afspraak opgeslagen.", "Afspraak verwijderd.", "Gebruikersnaam mag niet leeg zijn.", "Account verwijderd.", "Agenda verwijderd.", "Zichtbaarheid aangepast"],
    msg_sidebar: msg_sidebar_default
);

// 17. Polish (pl) - Polski
define_locale!(
    PL,
    code: "pl",
    label: "Polski",
    short_label: "PL",
    views: ["1:Miesiąc", "2:Tydzień", "3:Dzień", "4:Plan"],
    buttons: ["Dzisiaj", "+ Nowe", "⟳ Synchronizuj", "⟳ Synchronizacja", "☁ Konta", "⚙ Ustawienia", "? Pomoc", "[ Zapisz ]", "[ Anuluj ]", "[ Edytuj (e) ]", "[ Usuń (d) ]", "[ Zamknij (q) ]"],
    headers: ["ND", "PN", "WT", "ŚR", "CZ", "PT", "SO"],
    week_start_labels: ["Początek w niedzielę", "Początek w poniedziałek", "Początek w sobotę"],
    fmt_my: format_iso_month_year,
    fmt_dl: format_iso_date_long,
    sidebar: [" Moje Kalendarze ", " + Zarządzaj (C) ", "[ Ukryj (b) ]"],
    footer: ["Widok", "Nowe", "Sync", "Język", "Konta", "Pomoc", "Wyjdź"],
    modals: [" Nowe Wydarzenie ", " Edytuj Wydarzenie ", " Szczegóły ", " Utwórz Kalendarz ", " Dodaj Konto ", " ⚙ Ustawienia & Motywy ", " Skróty Klawiszowe "],
    fields: [" Tytuł:        ", " Kalendarz:    ", " Data:         ", " Początek:     ", " Koniec:       ", " Cały dzień:   ", " Lokalizacja:  ", " Opis:         "],
    placeholders: ["(Tytuł wydarzenia...)", "(Opcjonalna lokalizacja)", "(Notatki)"],
    settings_titles: ["▶ 1. Pierwszy dzień tygodnia", "▶ 2. Motywy kolorystyczne", "▶ 3. Język"],
    all_day: "Cały dzień",
    more_events: more_events_def,
    status: ["Witaj w rcal! Naciśnij ? aby uzyskać pomoc.", "Wydarzenie zapisane.", "Wydarzenie usunięte.", "Nazwa użytkownika nie może być pusta.", "Konto usunięte.", "Kalendarz usunięty.", "Widoczność przełączona"],
    msg_sidebar: msg_sidebar_default
);

// 18. Swedish (sv) - Svenska
define_locale!(
    SV,
    code: "sv",
    label: "Svenska",
    short_label: "SV",
    views: ["1:Månad", "2:Vecka", "3:Dag", "4:Agenda"],
    buttons: ["Idag", "+ Ny", "⟳ Synk", "⟳ Synkroniserar", "☁ Konton", "⚙ Inställningar", "? Hjälp", "[ Spara ]", "[ Avbryt ]", "[ Redigera (e) ]", "[ Ta bort (d) ]", "[ Stäng (q) ]"],
    headers: ["SÖN", "MÅN", "TIS", "ONS", "TOR", "FRE", "LÖR"],
    week_start_labels: ["Söndag först", "Måndag först", "Lördag först"],
    fmt_my: format_iso_month_year,
    fmt_dl: format_iso_date_long,
    sidebar: [" Mina Kalendrar ", " + Hantera (C) ", "[ Dölj (b) ]"],
    footer: ["Vy", "Ny", "Synk", "Språk", "Konton", "Hjälp", "Avsluta"],
    modals: [" Ny Händelse ", " Redigera Händelse ", " Detaljer ", " Skapa Kalender ", " Lägg till Konto ", " ⚙ Inställningar & Teman ", " Kortkommandon "],
    fields: [" Titel:        ", " Kalender:     ", " Datum:        ", " Starttid:     ", " Sluttid:      ", " Heldags:      ", " Plats:        ", " Beskrivning:  "],
    placeholders: ["(Händelsetitel...)", "(Valfri plats)", "(Anteckningar)"],
    settings_titles: ["▶ 1. Veckans första dag", "▶ 2. Färgteman", "▶ 3. Språk"],
    all_day: "Heldag",
    more_events: more_events_def,
    status: ["Välkommen till rcal! Tryck på ? för hjälp.", "Händelsen sparades.", "Händelsen raderades.", "Användarnamn kan inte vara tomt.", "Konto borttaget.", "Kalender raderad.", "Synlighet ändrades"],
    msg_sidebar: msg_sidebar_default
);
