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

    // Sidebar
    CalendarToggle(String /* calendar_id */),

    // Views
    MonthDay(NaiveDate),
    WeekSlot(NaiveDate, u32 /* hour */),
    DaySlot(u32 /* hour */, u32 /* minute */),
    EventItem(String /* event_id */),

    // Modals
    ModalField(usize),
    ModalSave,
    ModalCancel,
    ModalEdit,
    ModalDelete,
    ModalClose,
    ModalAccountField(usize),
    ModalAccountConnect,
    ModalAccountDelete(String),
    ModalAccountAdd,
    ModalAccountPreset(usize),
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

    pub fn register(&mut self, rect: Rect, target: HitTarget) {
        if rect.width > 0 && rect.height > 0 {
            self.targets.push((rect, target));
        }
    }

    /// Finds the top-most target that contains the given coordinate (x, y)
    /// (Iterates in reverse so that modals and topmost elements take priority)
    pub fn check(&self, x: u16, y: u16) -> Option<HitTarget> {
        for (rect, target) in self.targets.iter().rev() {
            if x >= rect.x
                && x < rect.x.saturating_add(rect.width)
                && y >= rect.y
                && y < rect.y.saturating_add(rect.height)
            {
                return Some(target.clone());
            }
        }
        None
    }
}
