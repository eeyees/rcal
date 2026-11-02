use crate::model::WeekStartDay;
use chrono::NaiveDate;
use ratatui::layout::Rect;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HitTarget {
    // Navigation & Tabs
    TabMonth,
    TabWeek,
    TabDay,
    TabAgenda,
    NavPrev,
    NavToday,
    NavNext,
    BtnNewEvent,
    BtnHelp,
    BtnSync,
    BtnAccounts,
    BtnLanguage,
    BtnTheme,
    BtnManageCalendars,
    BtnSettings,

    // Sidebar
    CalendarToggle(String /* calendar_id */),
    BtnToggleSidebar,
    MiniCalendarDay(NaiveDate),
    MiniCalPrev,
    MiniCalNext,

    // Views
    MonthDay(NaiveDate),
    WeekDayHeader(NaiveDate),
    WeekSlot(NaiveDate, u32 /* hour */),
    WeekAllDaySlot(NaiveDate),
    DaySlot(u32 /* hour */, u32 /* minute */),
    DayAllDaySlot(NaiveDate),
    EventItem(String /* event_id */),

    // Modals
    ModalBackdrop,
    ModalConsume,
    ModalField(usize),
    ModalSave,
    ModalCancel,
    ModalEdit,
    ModalDelete,
    ModalClose,
    ModalAccountField(usize),
    ModalAccountConnect,
    ModalAccountOAuthLogin,
    ModalAccountDelete(String),
    ModalAccountAdd,
    ModalAccountPreset(usize),
    ModalCalNameField,
    ModalCalColor(usize),
    ModalCalSave,
    ModalCalDelete(String),
    ModalCalNew,
    ModalCalToggle(String),
    ModalCalSaveVisibility,
    ModalSettingTab(crate::app::SettingsSection),
    ModalSettingWeekStart(WeekStartDay),
    ModalSettingNotificationToggle,
    ModalSettingNotificationLead(u32),
    ModalSettingNotificationSound,
    ModalSettingNotificationTest,
    ModalSettingTheme(crate::ui::theme::ThemeMode),
    ModalSettingThemeCategory(crate::ui::theme::ThemeCategory),
    ModalSettingThemePrev,
    ModalSettingThemeNext,
    ModalSelectLanguage(crate::i18n::Language),
    ModalSelectTheme(crate::ui::theme::ThemeMode),
    ModalThemeCategory(Option<crate::ui::theme::ThemeCategory>),
    ModalConfirmYes,
    ModalConfirmNo,
}

impl HitTarget {
    pub fn is_modal(&self) -> bool {
        matches!(
            self,
            HitTarget::ModalBackdrop
                | HitTarget::ModalConsume
                | HitTarget::ModalField(_)
                | HitTarget::ModalSave
                | HitTarget::ModalCancel
                | HitTarget::ModalEdit
                | HitTarget::ModalDelete
                | HitTarget::ModalClose
                | HitTarget::ModalAccountField(_)
                | HitTarget::ModalAccountConnect
                | HitTarget::ModalAccountOAuthLogin
                | HitTarget::ModalAccountDelete(_)
                | HitTarget::ModalAccountAdd
                | HitTarget::ModalAccountPreset(_)
                | HitTarget::ModalCalNameField
                | HitTarget::ModalCalColor(_)
                | HitTarget::ModalCalSave
                | HitTarget::ModalCalDelete(_)
                | HitTarget::ModalCalNew
                | HitTarget::ModalCalToggle(_)
                | HitTarget::ModalCalSaveVisibility
                | HitTarget::ModalSettingTab(_)
                | HitTarget::ModalSettingWeekStart(_)
                | HitTarget::ModalSettingNotificationToggle
                | HitTarget::ModalSettingNotificationLead(_)
                | HitTarget::ModalSettingNotificationSound
                | HitTarget::ModalSettingNotificationTest
                | HitTarget::ModalSettingTheme(_)
                | HitTarget::ModalSettingThemeCategory(_)
                | HitTarget::ModalSettingThemePrev
                | HitTarget::ModalSettingThemeNext
                | HitTarget::ModalSelectLanguage(_)
                | HitTarget::ModalSelectTheme(_)
                | HitTarget::ModalThemeCategory(_)
                | HitTarget::ModalConfirmYes
                | HitTarget::ModalConfirmNo
        )
    }
}

#[derive(Default, Debug, Clone)]
pub struct HitMap {
    targets: Vec<(Rect, HitTarget)>,
}

impl HitMap {
    pub fn new() -> Self {
        Self {
            targets: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.targets.clear();
    }

    pub fn targets(&self) -> &[(Rect, HitTarget)] {
        &self.targets
    }

    pub fn register(&mut self, rect: Rect, target: HitTarget) {
        if rect.width > 0 && rect.height > 0 {
            self.targets.push((rect, target));
        }
    }

    /// Finds the top-most target that contains the given coordinate (x, y)
    /// (Iterates in reverse so that modals and topmost elements take priority)
    pub fn check(&self, x: u16, y: u16) -> Option<HitTarget> {
        let has_modal = self.targets.iter().any(|(_, t)| t.is_modal());
        for (rect, target) in self.targets.iter().rev() {
            if x >= rect.x
                && x < rect.x.saturating_add(rect.width)
                && y >= rect.y
                && y < rect.y.saturating_add(rect.height)
            {
                if has_modal && !target.is_modal() {
                    return None;
                }
                return Some(target.clone());
            }
        }
        None
    }
}
