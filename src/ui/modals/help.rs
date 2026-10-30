use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::App;
use crate::mouse::HitTarget;
use crate::ui::theme::Theme;

pub fn render(f: &mut Frame, app: &mut App, area: Rect) {
    let popup_width = 72.min(area.width.saturating_sub(4));
    let popup_height = 24.min(area.height.saturating_sub(4));

    let popup_rect = Rect {
        x: area.x + (area.width.saturating_sub(popup_width)) / 2,
        y: area.y + (area.height.saturating_sub(popup_height)) / 2,
        width: popup_width,
        height: popup_height,
    };

    f.render_widget(Clear, popup_rect);

    let lang = app.lang;
    let title_text = match lang {
        crate::i18n::Language::Ko => " 키보드 & 마우스 조작 단축키 안내 ",
        crate::i18n::Language::En => " Keyboard & Mouse Shortcuts ",
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::PRIMARY))
        .title(Span::styled(title_text, Style::default().fg(Theme::TEXT_HIGHLIGHT).add_modifier(Modifier::BOLD)));
    let inner = block.inner(popup_rect);
    f.render_widget(block, popup_rect);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(18),
            Constraint::Length(1), // [ Close ]
        ])
        .split(inner);

    let help_text = match lang {
        crate::i18n::Language::Ko => vec![
            Line::from(vec![
                Span::styled("🖱  마우스 조작:", Style::default().fg(Theme::SECONDARY).add_modifier(Modifier::BOLD)),
            ]),
            Line::from("  • 상단 뷰 탭 클릭       : 월간 / 주간 / 일간 / 목록 뷰 즉시 전환"),
            Line::from("  • [<] / [>] 클릭        : 이전 / 다음 기간 이동 (마우스 휠 스크롤 가능)"),
            Line::from("  • [오늘] 클릭           : 오늘 날짜로 즉시 이동"),
            Line::from("  • 날짜 셀 클릭          : 해당 날짜 선택"),
            Line::from("  • 날짜 셀 더블 클릭     : 해당 날짜의 일간 뷰로 즉시 이동"),
            Line::from("  • 시간 슬롯 클릭        : 해당 시간대에 새 일정 등록 모달 열기"),
            Line::from("  • 일정 블록 클릭        : 일정 상세 보기 (수정 / 삭제 모달)"),
            Line::from("  • [x] 체크박스 클릭     : 캘린더 표시 / 숨김 토글"),
            Line::from(""),
            Line::from(vec![
                Span::styled("⌨  키보드 단축키:", Style::default().fg(Theme::PRIMARY).add_modifier(Modifier::BOLD)),
            ]),
            Line::from("  • 1 / 2 / 3 / 4         : 월간(1) / 주간(2) / 일간(3) / 목록(4) 전환"),
            Line::from("  • h / j / k / l         : 좌 / 하 / 상 / 우 날짜 및 시간대 이동"),
            Line::from("  • [ / ]                 : 이전 / 다음 월/주/일 이동"),
            Line::from("  • t                     : 오늘 날짜로 이동"),
            Line::from("  • c 또는 n              : 새 일정 생성"),
            Line::from("  • r                     : 클라우드 실시간 동기화 실행 (CalDAV/OAuth2)"),
            Line::from("  • A                     : 계정 관리 (Google / MS / Naver / Apple)"),
            Line::from("  • C                     : 캘린더 카테고리 관리 및 추가/삭제"),
            Line::from("  • o 또는 S              : 환경 설정 (주간 시작 요일 등)"),
            Line::from("  • L                     : 언어 즉시 전환 (한국어 ↔ English)"),
            Line::from("  • ?                     : 도움말 화면 표시"),
            Line::from("  • q / Esc               : 닫기 또는 프로그램 종료"),
        ],
        crate::i18n::Language::En => vec![
            Line::from(vec![
                Span::styled("🖱  Mouse Controls:", Style::default().fg(Theme::SECONDARY).add_modifier(Modifier::BOLD)),
            ]),
            Line::from("  • Click View Tabs       : Switch between Month, Week, Day, Agenda"),
            Line::from("  • Click [<] / [>]       : Navigate backward / forward in time (or wheel scroll)"),
            Line::from("  • Click [Today]         : Jump straight to today's date"),
            Line::from("  • Click Day Cell        : Select date in Month view"),
            Line::from("  • Double Click Day      : Open Day view for that date"),
            Line::from("  • Click Time Slot       : Create new event at that hour"),
            Line::from("  • Click Event Block     : View event details & edit/delete"),
            Line::from("  • Click [x] Checkbox    : Toggle calendar visibility ON/OFF"),
            Line::from(""),
            Line::from(vec![
                Span::styled("⌨  Keyboard Shortcuts:", Style::default().fg(Theme::PRIMARY).add_modifier(Modifier::BOLD)),
            ]),
            Line::from("  • 1 / 2 / 3 / 4         : Month / Week / Day / Agenda view"),
            Line::from("  • h / j / k / l         : Navigate days / weeks / hours"),
            Line::from("  • [ / ]                 : Previous / Next period"),
            Line::from("  • t                     : Jump to today"),
            Line::from("  • c / n                 : Create new event"),
            Line::from("  • r                     : Trigger sync (CalDAV / Google / MS)"),
            Line::from("  • A                     : Manage accounts & cloud providers"),
            Line::from("  • C                     : Manage & Create calendars / categories"),
            Line::from("  • o / S                 : Preferences (Week start day, etc.)"),
            Line::from("  • L                     : Toggle Language (Korean ↔ English)"),
            Line::from("  • ?                     : Toggle this help screen"),
            Line::from("  • q / Esc               : Close modal or quit"),
        ],
    };

    f.render_widget(Paragraph::new(help_text), chunks[0]);

    let btn_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(16),
            Constraint::Min(1),
        ])
        .split(chunks[1]);

    let btn_close = Paragraph::new(Line::from(vec![Span::raw(lang.btn_close())]))
        .style(Theme::button_style())
        .alignment(Alignment::Center);
    f.render_widget(btn_close, btn_chunks[1]);
    app.hit_map.register(btn_chunks[1], HitTarget::ModalClose);
}
