//! A macOS window whose sidebar and toolbar are AppKit's own: an
//! `NSSplitViewController` with the source list as its sidebar item and the
//! nsh surface as its content, under a unified toolbar that carries the system
//! sidebar toggle. nsh does not run the window; `AppView` is only embedded.

#![cfg(target_os = "macos")]

use imago_native_sidebar::Row;
use nsh_platform_macos::{App, AppOptions, AppView, Root};
use objc2::{
    define_class, msg_send, rc::Retained, runtime::AnyObject, runtime::ProtocolObject,
    runtime::Sel, sel, DefinedClass, MainThreadOnly,
};
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSBackingStoreType, NSBezelStyle, NSBorderType,
    NSButton, NSCellImagePosition, NSControlTextEditingDelegate, NSImage, NSImageView,
    NSLayoutConstraint, NSLineBreakMode, NSMenu, NSMenuItem, NSPopover, NSPopoverBehavior,
    NSResponder, NSScrollView, NSSearchField, NSSegmentSwitchTracking, NSSegmentedControl,
    NSSplitViewController, NSSplitViewItem, NSTableCellView, NSTableColumn, NSTableView,
    NSTableViewDataSource, NSTableViewDelegate, NSTableViewStyle, NSTextAlignment, NSTextField,
    NSToolbar, NSToolbarDelegate, NSToolbarDisplayMode, NSToolbarFlexibleSpaceItemIdentifier,
    NSToolbarItem, NSToolbarItemIdentifier, NSToolbarSidebarTrackingSeparatorItemIdentifier,
    NSToolbarToggleSidebarItemIdentifier, NSView, NSViewController, NSWindow, NSWindowStyleMask,
    NSWindowTitleVisibility, NSWindowToolbarStyle, NSWorkspace,
};
use objc2_foundation::{
    MainThreadMarker, NSArray, NSIndexSet, NSNotification, NSObject, NSObjectProtocol, NSPoint,
    NSRect, NSRectEdge, NSSet, NSSize, NSString, NSTimer,
};
use std::cell::RefCell;
use std::process::ExitCode;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard};

/// Row ids the user picked in the sidebar since they were last taken.
static PRESSES: Mutex<Vec<String>> = Mutex::new(Vec::new());

/// The ids of the sidebar rows picked since the last call, oldest first.
pub fn sidebar_presses() -> Vec<String> {
    std::mem::take(&mut *PRESSES.lock().unwrap_or_else(|poison| poison.into_inner()))
}

/// Whether the sidebar is collapsed, as of the split view's last layout.
static COLLAPSED: AtomicBool = AtomicBool::new(false);

/// Whether the user hid the sidebar: the title bar's controls then lie over
/// the content's leading edge.
pub fn sidebar_collapsed() -> bool {
    COLLAPSED.load(Ordering::Relaxed)
}

/// One entry of a book's contents.
#[derive(Clone)]
pub struct Contents {
    pub title: String,
    /// Zero-based page the entry opens at.
    pub page: usize,
    pub depth: usize,
}

/// The contents of the book in the reader window, listed by its popover.
static CONTENTS_LIST: Mutex<Vec<Contents>> = Mutex::new(Vec::new());
/// The highlights of the book in the reader window, listed by its notes popover.
static NOTES_LIST: Mutex<Vec<Contents>> = Mutex::new(Vec::new());
/// Zero-based pages picked in the contents or notes popover since the last call.
static READER_PRESSES: Mutex<Vec<usize>> = Mutex::new(Vec::new());
/// The title of a reader window asked for, until the main thread opens it.
static READER_OPEN: Mutex<Option<ReaderOpen>> = Mutex::new(None);

/// A reader window asked for, until the main thread opens it.
struct ReaderOpen {
    title: String,
    /// The file the window's title bar shows the icon of.
    path: String,
}

/// Sets the contents the reader window's popover lists.
pub fn set_contents(contents: Vec<Contents>) {
    *lock(&CONTENTS_LIST) = contents;
}

/// Sets the highlights the reader window's notes popover lists.
pub fn set_notes(notes: Vec<Contents>) {
    *lock(&NOTES_LIST) = notes;
}

/// The pages picked in the contents or notes popover since the last call, oldest first.
pub fn reader_presses() -> Vec<usize> {
    std::mem::take(&mut *lock(&READER_PRESSES))
}

/// What a reader window's toolbar asks the app to do.
pub enum ReaderCommand {
    /// Zooms in (`true`) or out.
    Zoom(bool),
    /// Switches to the tone at this index of [`ReaderState::tones`].
    Tone(usize),
    /// Switches reading mode on or off.
    TextMode,
    /// The search field's text as it changes; empty when cleared.
    Search(String),
}

/// The reading state the reader window's toolbar shows.
#[derive(Clone)]
pub struct ReaderState {
    pub zoom_percent: u32,
    pub zoom_enabled: bool,
    /// The labels of the tones, in the order [`ReaderCommand::Tone`] picks.
    pub tones: Vec<String>,
    pub tone: usize,
    pub text_mode: bool,
}

/// The toolbar's requests the app has not taken yet, oldest first.
static READER_COMMANDS: Mutex<Vec<ReaderCommand>> = Mutex::new(Vec::new());
/// The reading state the app last set.
static READER_STATE: Mutex<Option<ReaderState>> = Mutex::new(None);
/// Search hits the app set, until the main thread shows them.
static SEARCH_RESULTS: Mutex<Option<Vec<Contents>>> = Mutex::new(None);

/// The toolbar's requests since the last call, oldest first.
pub fn reader_commands() -> Vec<ReaderCommand> {
    std::mem::take(&mut *lock(&READER_COMMANDS))
}

/// Sets the reading state the reader window's toolbar shows.
pub fn set_reader_state(state: ReaderState) {
    *lock(&READER_STATE) = Some(state);
}

/// Shows `hits` under the reader window's search field; empty hides them.
pub fn set_search_results(hits: Vec<Contents>) {
    *lock(&SEARCH_RESULTS) = Some(hits);
}

/// Opens a reader window titled `title` for the file at `path` on the main
/// thread, closing the one open before. The app loads its own book for it, from
/// whatever it set up.
pub fn open_reader(title: &str, path: &str) {
    *lock(&READER_OPEN) = Some(ReaderOpen {
        title: title.to_owned(),
        path: path.to_owned(),
    });
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poison| poison.into_inner())
}

/// Shows a window titled `title` of `size` points: `rows` in the system
/// sidebar, and the nsh app from `root` as the content. Reader windows from
/// [`open_reader`] load the app from `reader_root`. `components` registers the
/// native components of every window before its app loads. Runs the app loop
/// until the process ends.
pub fn run(
    title: &str,
    size: (f64, f64),
    rows: Vec<Row>,
    root: fn() -> Root,
    reader_root: fn() -> Root,
    components: fn(&AppView),
) -> ExitCode {
    let mtm = MainThreadMarker::new().expect("run must be called on the main thread");
    let app = NSApplication::sharedApplication(mtm);
    app.setActivationPolicy(NSApplicationActivationPolicy::Regular);
    // Kept until the app loop ends: the menu items hold their target weakly.
    let zoom_menu = ZoomMenu::new(mtm);
    install_main_menu(mtm, &app, &zoom_menu);

    let frame = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(size.0, size.1));
    let style = NSWindowStyleMask::Titled
        | NSWindowStyleMask::FullSizeContentView
        | NSWindowStyleMask::Closable
        | NSWindowStyleMask::Miniaturizable
        | NSWindowStyleMask::Resizable;
    // SAFETY: NSWindow's designated initializer.
    let window = unsafe {
        NSWindow::initWithContentRect_styleMask_backing_defer(
            NSWindow::alloc(mtm),
            frame,
            style,
            NSBackingStoreType::Buffered,
            false,
        )
    };
    // SAFETY: the window lives in this function until the app loop ends.
    unsafe { window.setReleasedWhenClosed(false) };
    window.setTitle(&NSString::from_str(title));

    let view = AppView::new(
        mtm,
        AppOptions {
            title: title.to_owned(),
            size: (size.0 as i32, size.1 as i32),
            ..Default::default()
        },
    );
    components(&view);
    view.load(App::Rust(root()));

    let (source_list, _rows) = source_list(mtm, rows);
    let sidebar_controller = NSViewController::new(mtm);
    sidebar_controller.setView(&source_list);
    let content_controller = NSViewController::new(mtm);
    content_controller.setView(&view.view());

    let split = Split::new(mtm);
    let sidebar = NSSplitViewItem::sidebarWithViewController(&sidebar_controller);
    sidebar.setMinimumThickness(220.0);
    split.addSplitViewItem(&sidebar);
    split.addSplitViewItem(&NSSplitViewItem::splitViewItemWithViewController(
        &content_controller,
    ));
    window.setContentViewController(Some(&split));

    let toolbar_delegate = Toolbar::new(mtm);
    let toolbar =
        NSToolbar::initWithIdentifier(NSToolbar::alloc(mtm), &NSString::from_str("imago.window"));
    toolbar.setDelegate(Some(ProtocolObject::from_ref(&*toolbar_delegate)));
    toolbar.setDisplayMode(NSToolbarDisplayMode::IconOnly);
    window.setToolbar(Some(&toolbar));
    window.setToolbarStyle(NSWindowToolbarStyle::Unified);
    window.setTitlebarAppearsTransparent(true);
    window.setTitleVisibility(NSWindowTitleVisibility::Hidden);

    let ticker = Ticker::new(
        mtm,
        TickerIvars {
            reader_root,
            components,
        },
    );
    // SAFETY: the timer retains `ticker` and runs on the main loop, where the
    // windows live; it lasts as long as the app loop.
    unsafe {
        NSTimer::scheduledTimerWithTimeInterval_target_selector_userInfo_repeats(
            0.03,
            &ticker,
            sel!(tick:),
            None,
            true,
        );
    }

    window.center();
    window.makeKeyAndOrderFront(None);
    app.activate();
    app.run();
    ExitCode::SUCCESS
}

/// The system source list: one cell per row, in the sidebar material, the
/// first row selected.
fn source_list(mtm: MainThreadMarker, rows: Vec<Row>) -> (Retained<NSView>, Retained<Rows>) {
    let table = NSTableView::new(mtm);
    table.setStyle(NSTableViewStyle::SourceList);
    table.setHeaderView(None);
    table.setRowHeight(32.0);
    table.addTableColumn(&NSTableColumn::new(mtm));
    let source = Rows::new(mtm, rows);
    // SAFETY: `source` is returned to the caller, which keeps it alive for as
    // long as the table; the table holds its data source and delegate weakly.
    unsafe {
        table.setDataSource(Some(ProtocolObject::from_ref(&*source)));
        table.setDelegate(Some(ProtocolObject::from_ref(&*source)));
    }
    table.selectRowIndexes_byExtendingSelection(&NSIndexSet::indexSetWithIndex(0), false);
    let scroll = NSScrollView::new(mtm);
    scroll.setDocumentView(Some(&table));
    scroll.setDrawsBackground(false);
    (Retained::into_super(scroll), source)
}

pub struct RowsIvars {
    rows: Vec<Row>,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[ivars = RowsIvars]
    pub struct Rows;

    unsafe impl NSObjectProtocol for Rows {}

    unsafe impl NSControlTextEditingDelegate for Rows {}

    unsafe impl NSTableViewDataSource for Rows {
        #[unsafe(method(numberOfRowsInTableView:))]
        fn number_of_rows(&self, _table: &NSTableView) -> isize {
            self.ivars().rows.len() as isize
        }
    }

    unsafe impl NSTableViewDelegate for Rows {
        #[unsafe(method_id(tableView:viewForTableColumn:row:))]
        fn view_for_row(
            &self,
            _table: &NSTableView,
            _column: Option<&NSTableColumn>,
            row: isize,
        ) -> Option<Retained<NSView>> {
            let mtm = MainThreadMarker::from(self);
            self.ivars()
                .rows
                .get(row as usize)
                .map(|row| row_view(mtm, row))
        }

        #[unsafe(method(tableViewSelectionDidChange:))]
        fn selection_changed(&self, notification: &NSNotification) {
            let Some(table) = notification
                .object()
                .and_then(|object| object.downcast::<NSTableView>().ok())
            else {
                return;
            };
            let Ok(row) = usize::try_from(table.selectedRow()) else {
                return;
            };
            if let Some(row) = self.ivars().rows.get(row) {
                PRESSES
                    .lock()
                    .unwrap_or_else(|poison| poison.into_inner())
                    .push(row.id.to_owned());
            }
        }
    }
);

/// One row: its symbol and title, in a cell view the source list styles.
fn row_view(mtm: MainThreadMarker, row: &Row) -> Retained<NSView> {
    let cell = NSTableCellView::initWithFrame(NSTableCellView::alloc(mtm), NSRect::ZERO);
    let icon = NSImageView::new(mtm);
    if let Some(name) = row.symbol {
        icon.setImage(
            NSImage::imageWithSystemSymbolName_accessibilityDescription(
                &NSString::from_str(name),
                None,
            )
            .as_deref(),
        );
    }
    let label = NSTextField::labelWithString(&NSString::from_str(&row.label), mtm);
    icon.setTranslatesAutoresizingMaskIntoConstraints(false);
    label.setTranslatesAutoresizingMaskIntoConstraints(false);
    cell.addSubview(&icon);
    cell.addSubview(&label);
    let constraints = NSArray::from_retained_slice(&[
        icon.leadingAnchor()
            .constraintEqualToAnchor_constant(&cell.leadingAnchor(), 8.0),
        icon.centerYAnchor()
            .constraintEqualToAnchor(&cell.centerYAnchor()),
        icon.widthAnchor().constraintEqualToConstant(16.0),
        icon.heightAnchor().constraintEqualToConstant(16.0),
        label
            .leadingAnchor()
            .constraintEqualToAnchor_constant(&icon.trailingAnchor(), 8.0),
        label
            .centerYAnchor()
            .constraintEqualToAnchor(&cell.centerYAnchor()),
    ]);
    NSLayoutConstraint::activateConstraints(&constraints);
    Retained::into_super(cell)
}

impl Rows {
    fn new(mtm: MainThreadMarker, rows: Vec<Row>) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(RowsIvars { rows });
        // SAFETY: `init` on a freshly allocated instance of this class.
        unsafe { msg_send![super(this), init] }
    }
}

define_class!(
    /// A split view controller that notes in [`COLLAPSED`] whether its first
    /// item, the sidebar, is collapsed.
    #[unsafe(super(NSSplitViewController, NSViewController, NSResponder, NSObject))]
    #[thread_kind = MainThreadOnly]
    struct Split;

    impl Split {
        #[unsafe(method(splitViewDidResizeSubviews:))]
        fn resized(&self, notification: &NSNotification) {
            // SAFETY: NSSplitViewController's own handler, which it lays out in.
            let () = unsafe { msg_send![super(self), splitViewDidResizeSubviews: notification] };
            let collapsed = self
                .splitViewItems()
                .firstObject()
                .is_some_and(|item| item.isCollapsed());
            COLLAPSED.store(collapsed, Ordering::Relaxed);
        }
    }
);

impl Split {
    fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = Self::alloc(mtm);
        // SAFETY: `init` on a freshly allocated instance of this class.
        unsafe { msg_send![this, init] }
    }
}

/// AppKit's own sidebar toggle, the sidebar's tracking separator and the space
/// after them: AppKit builds these items itself.
fn standard_items() -> Retained<NSArray<NSToolbarItemIdentifier>> {
    // SAFETY: AppKit's own identifiers, which it builds the items for.
    unsafe {
        NSArray::from_slice(&[
            NSToolbarFlexibleSpaceItemIdentifier,
            NSToolbarToggleSidebarItemIdentifier,
            NSToolbarSidebarTrackingSeparatorItemIdentifier,
            NSToolbarFlexibleSpaceItemIdentifier,
        ])
    }
}

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    struct Toolbar;

    unsafe impl NSObjectProtocol for Toolbar {}

    unsafe impl NSToolbarDelegate for Toolbar {
        #[unsafe(method_id(toolbarDefaultItemIdentifiers:))]
        fn default_items(
            &self,
            _toolbar: &NSToolbar,
        ) -> Retained<NSArray<NSToolbarItemIdentifier>> {
            standard_items()
        }

        #[unsafe(method_id(toolbar:itemForItemIdentifier:willBeInsertedIntoToolbar:))]
        fn item(
            &self,
            _toolbar: &NSToolbar,
            _identifier: &NSToolbarItemIdentifier,
            _inserted: bool,
        ) -> Option<Retained<NSToolbarItem>> {
            // AppKit builds the standard items itself for a delegate that
            // answers nil.
            None
        }

        #[unsafe(method_id(toolbarAllowedItemIdentifiers:))]
        fn allowed_items(
            &self,
            _toolbar: &NSToolbar,
        ) -> Retained<NSArray<NSToolbarItemIdentifier>> {
            standard_items()
        }
    }
);

impl Toolbar {
    fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(());
        // SAFETY: `init` on a freshly allocated instance of this class.
        unsafe { msg_send![super(this), init] }
    }
}

/// The table of contents in the popover: each entry's title and page, and a
/// press on one opens its page.
pub struct ContentsIvars {
    entries: Vec<Contents>,
    table: Retained<NSTableView>,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[ivars = ContentsIvars]
    struct ContentsTable;

    unsafe impl NSObjectProtocol for ContentsTable {}

    unsafe impl NSControlTextEditingDelegate for ContentsTable {}

    unsafe impl NSTableViewDataSource for ContentsTable {
        #[unsafe(method(numberOfRowsInTableView:))]
        fn number_of_rows(&self, _table: &NSTableView) -> isize {
            self.ivars().entries.len() as isize
        }
    }

    unsafe impl NSTableViewDelegate for ContentsTable {
        #[unsafe(method_id(tableView:viewForTableColumn:row:))]
        fn view_for_row(
            &self,
            _table: &NSTableView,
            _column: Option<&NSTableColumn>,
            row: isize,
        ) -> Option<Retained<NSView>> {
            let mtm = MainThreadMarker::from(self);
            self.ivars()
                .entries
                .get(row as usize)
                .map(|entry| contents_row_view(mtm, entry))
        }

        #[unsafe(method(tableViewSelectionDidChange:))]
        fn selection_changed(&self, notification: &NSNotification) {
            let Some(table) = notification
                .object()
                .and_then(|object| object.downcast::<NSTableView>().ok())
            else {
                return;
            };
            let Ok(row) = usize::try_from(table.selectedRow()) else {
                return;
            };
            if let Some(entry) = self.ivars().entries.get(row) {
                lock(&READER_PRESSES).push(entry.page);
            }
        }
    }
);

impl ContentsTable {
    fn new(mtm: MainThreadMarker, entries: Vec<Contents>) -> Retained<Self> {
        let table = NSTableView::new(mtm);
        table.setHeaderView(None);
        table.setRowHeight(28.0);
        table.addTableColumn(&NSTableColumn::new(mtm));
        let this = Self::alloc(mtm).set_ivars(ContentsIvars {
            entries,
            table: table.clone(),
        });
        // SAFETY: `init` on a freshly allocated instance of this class; the
        // table holds its data source and delegate weakly, so the caller keeps
        // the returned table alive with the popover.
        let this: Retained<Self> = unsafe { msg_send![super(this), init] };
        unsafe {
            table.setDataSource(Some(ProtocolObject::from_ref(&*this)));
            table.setDelegate(Some(ProtocolObject::from_ref(&*this)));
        }
        this
    }
}

/// One contents row: the title indented by depth, and the page it opens.
fn contents_row_view(mtm: MainThreadMarker, entry: &Contents) -> Retained<NSView> {
    let cell = NSTableCellView::initWithFrame(NSTableCellView::alloc(mtm), NSRect::ZERO);
    let title = NSTextField::labelWithString(&NSString::from_str(&entry.title), mtm);
    title.setLineBreakMode(NSLineBreakMode::ByTruncatingTail);
    let page =
        NSTextField::labelWithString(&NSString::from_str(&(entry.page + 1).to_string()), mtm);
    title.setTranslatesAutoresizingMaskIntoConstraints(false);
    page.setTranslatesAutoresizingMaskIntoConstraints(false);
    cell.addSubview(&title);
    cell.addSubview(&page);
    let indent = POPOVER_ROW_INSET + POPOVER_DEPTH_INDENT * entry.depth.min(4) as f64;
    let constraints = NSArray::from_retained_slice(&[
        title
            .leadingAnchor()
            .constraintEqualToAnchor_constant(&cell.leadingAnchor(), indent),
        title
            .centerYAnchor()
            .constraintEqualToAnchor(&cell.centerYAnchor()),
        title
            .trailingAnchor()
            .constraintLessThanOrEqualToAnchor_constant(&page.leadingAnchor(), -POPOVER_ROW_GAP),
        page.trailingAnchor()
            .constraintEqualToAnchor_constant(&cell.trailingAnchor(), -POPOVER_ROW_INSET),
        page.centerYAnchor()
            .constraintEqualToAnchor(&cell.centerYAnchor()),
    ]);
    NSLayoutConstraint::activateConstraints(&constraints);
    Retained::into_super(cell)
}

/// The reader window on screen, with its contents popover once shown.
type ReaderSettings = (
    Retained<NSPopover>,
    Retained<NSTextField>,
    Retained<NSButton>,
    Retained<NSButton>,
);

struct ReaderWindow {
    window: Retained<NSWindow>,
    _view: Rc<AppView>,
    toolbar: Retained<ReaderToolbar>,
    /// The contents popover, while it is open.
    contents: Option<(Retained<NSPopover>, Retained<ContentsTable>)>,
    /// The search popover, once it has been shown.
    search: Option<SearchPanel>,
    /// The settings popover, while it is open, with its zoom label.
    settings: Option<ReaderSettings>,
}

thread_local! {
    static READER_WINDOW: RefCell<Option<ReaderWindow>> = const { RefCell::new(None) };
}

/// Height of a list popover's title.
const POPOVER_TITLE_HEIGHT: f64 = 36.0;
/// Width of a list popover, and the rows it shows before it scrolls.
const POPOVER_WIDTH: f64 = 280.0;
const POPOVER_MAX_ROWS: usize = 12;
/// Height of a list row, and the padding of a list popover's table.
const POPOVER_ROW_HEIGHT: f64 = 28.0;
const POPOVER_PADDING: f64 = 8.0;
/// Left and right inset of a row, the indent per depth, and the gap before the page.
const POPOVER_ROW_INSET: f64 = 12.0;
const POPOVER_DEPTH_INDENT: f64 = 14.0;
const POPOVER_ROW_GAP: f64 = 8.0;
/// Width past which a reader window's title is cut with an ellipsis.
const TITLE_MAX_WIDTH: f64 = 300.0;
/// Height of the reader toolbar's title and groups.
const TOOLBAR_ITEM_HEIGHT: f64 = 32.0;
/// Width of one segment of a reader toolbar group.
const SEGMENT_WIDTH: f64 = 36.0;
/// Segments of the left group, and of the right group.
const LEFT_CONTENTS: isize = 0;
const LEFT_NOTES: isize = 1;
const RIGHT_SEARCH: isize = 0;
const RIGHT_SETTINGS: isize = 1;
const RIGHT_MODE: isize = 2;

const SEARCH_WIDTH: f64 = 320.0;
const SEARCH_MARGIN: f64 = 12.0;
const SEARCH_FIELD_HEIGHT: f64 = 28.0;
const SEARCH_GAP: f64 = 8.0;
const SEARCH_ROW_HEIGHT: f64 = 24.0;
/// Hits shown at once; the rest scroll.
const SEARCH_ROWS: usize = 8;

/// The toolbar of a reader window: the contents and notes group at the left,
/// the book's title with its file's icon centred, and the search, settings and
/// reading mode group at the right.
pub struct ReaderToolbarIvars {
    title: String,
    path: String,
    left: Retained<NSSegmentedControl>,
    right: Retained<NSSegmentedControl>,
    /// The field the search popover shows.
    search_field: Retained<NSSearchField>,
}

/// The search popover: the field on top, the hits below.
struct SearchPanel {
    popover: Retained<NSPopover>,
    root: Retained<NSView>,
    field: Retained<NSSearchField>,
    scroll: Retained<NSScrollView>,
    /// Kept here: the table holds its data source weakly.
    table: Retained<ContentsTable>,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[ivars = ReaderToolbarIvars]
    struct ReaderToolbar;

    unsafe impl NSObjectProtocol for ReaderToolbar {}

    unsafe impl NSToolbarDelegate for ReaderToolbar {
        #[unsafe(method_id(toolbarDefaultItemIdentifiers:))]
        fn default_items(
            &self,
            _toolbar: &NSToolbar,
        ) -> Retained<NSArray<NSToolbarItemIdentifier>> {
            reader_items()
        }

        #[unsafe(method_id(toolbarAllowedItemIdentifiers:))]
        fn allowed_items(
            &self,
            _toolbar: &NSToolbar,
        ) -> Retained<NSArray<NSToolbarItemIdentifier>> {
            reader_items()
        }

        #[unsafe(method_id(toolbar:itemForItemIdentifier:willBeInsertedIntoToolbar:))]
        fn item(
            &self,
            _toolbar: &NSToolbar,
            identifier: &NSToolbarItemIdentifier,
            _inserted: bool,
        ) -> Option<Retained<NSToolbarItem>> {
            let mtm = MainThreadMarker::from(self);
            let ivars = self.ivars();
            match identifier.to_string().as_str() {
                "imago.reader.left" => Some(toolbar_item(mtm, identifier, &ivars.left)),
                "imago.reader.right" => Some(toolbar_item(mtm, identifier, &ivars.right)),
                "imago.reader.title" => {
                    let title = title_button(mtm, &ivars.title, &ivars.path);
                    Some(toolbar_item(mtm, identifier, &title))
                }
                _ => None,
            }
        }
    }

    impl ReaderToolbar {
        #[unsafe(method(leftPressed:))]
        fn left_pressed(&self, _sender: Option<&AnyObject>) {
            let ivars = self.ivars();
            match ivars.left.selectedSegment() {
                LEFT_CONTENTS => {
                    show_reader_list(MainThreadMarker::from(self), &ivars.left, LEFT_CONTENTS, "Оглавление", &CONTENTS_LIST)
                }
                LEFT_NOTES => {
                    show_reader_list(MainThreadMarker::from(self), &ivars.left, LEFT_NOTES, "Заметки", &NOTES_LIST)
                }
                _ => {}
            }
        }

        #[unsafe(method(rightPressed:))]
        fn right_pressed(&self, _sender: Option<&AnyObject>) {
            let ivars = self.ivars();
            let mtm = MainThreadMarker::from(self);
            match ivars.right.selectedSegment() {
                RIGHT_SEARCH => show_reader_search(mtm, &ivars.right, RIGHT_SEARCH),
                RIGHT_SETTINGS => show_reader_settings(mtm, self, RIGHT_SETTINGS),
                RIGHT_MODE => push_command(ReaderCommand::TextMode),
                _ => {}
            }
        }

        #[unsafe(method(zoomIn:))]
        fn zoom_in(&self, _sender: Option<&AnyObject>) {
            push_command(ReaderCommand::Zoom(true));
        }

        #[unsafe(method(zoomOut:))]
        fn zoom_out(&self, _sender: Option<&AnyObject>) {
            push_command(ReaderCommand::Zoom(false));
        }

        #[unsafe(method(toneChanged:))]
        fn tone_changed(&self, sender: Option<&AnyObject>) {
            let Some(control) = sender.and_then(|sender| sender.downcast_ref::<NSSegmentedControl>())
            else {
                return;
            };
            if let Ok(tone) = usize::try_from(control.selectedSegment()) {
                push_command(ReaderCommand::Tone(tone));
            }
        }

        #[unsafe(method(searchChanged:))]
        fn search_changed(&self, _sender: Option<&AnyObject>) {
            let query = self.ivars().search_field.stringValue().to_string();
            push_command(ReaderCommand::Search(query));
        }
    }
);

impl ReaderToolbar {
    fn new(mtm: MainThreadMarker, title: &str, path: &str) -> Retained<Self> {
        let left = segment_group(mtm, &["list.bullet", "tray"]);
        let right = segment_group(mtm, &["magnifyingglass", "textformat.size", "book"]);
        let search_field = NSSearchField::new(mtm);
        search_field.setPlaceholderString(Some(&NSString::from_str("Поиск в книге")));
        search_field.setSendsSearchStringImmediately(true);
        // The field's own recent searches, kept by AppKit.
        search_field.setRecentsAutosaveName(Some(&NSString::from_str("imago.reader.search")));
        let this = Self::alloc(mtm).set_ivars(ReaderToolbarIvars {
            title: title.to_owned(),
            path: path.to_owned(),
            left,
            right,
            search_field,
        });
        // SAFETY: `init` on a freshly allocated instance of this class.
        let this: Retained<Self> = unsafe { msg_send![super(this), init] };
        let target: &AnyObject = &this;
        let ivars = this.ivars();
        // SAFETY: the targets are this toolbar, which the reader window keeps alive.
        unsafe {
            ivars.left.setTarget(Some(target));
            ivars.left.setAction(Some(sel!(leftPressed:)));
            ivars.right.setTarget(Some(target));
            ivars.right.setAction(Some(sel!(rightPressed:)));
            ivars.search_field.setTarget(Some(target));
            ivars.search_field.setAction(Some(sel!(searchChanged:)));
        }
        this
    }
}

/// The reader toolbar's items, left to right. The title is the centred item,
/// so it sits in the middle of the window whatever the groups' widths.
fn reader_items() -> Retained<NSArray<NSToolbarItemIdentifier>> {
    // SAFETY: AppKit's own flexible space identifier, which it builds the item for.
    unsafe {
        NSArray::from_slice(&[
            &*NSString::from_str("imago.reader.left"),
            NSToolbarFlexibleSpaceItemIdentifier,
            &*NSString::from_str("imago.reader.title"),
            NSToolbarFlexibleSpaceItemIdentifier,
            &*NSString::from_str("imago.reader.right"),
        ])
    }
}

fn toolbar_item(
    mtm: MainThreadMarker,
    identifier: &NSToolbarItemIdentifier,
    view: &NSView,
) -> Retained<NSToolbarItem> {
    let item = NSToolbarItem::initWithItemIdentifier(NSToolbarItem::alloc(mtm), identifier);
    item.setView(Some(view));
    item
}

fn push_command(command: ReaderCommand) {
    lock(&READER_COMMANDS).push(command);
}

/// A glass group of momentary segments, one per system symbol: a press runs the
/// action and leaves no segment selected.
fn segment_group(mtm: MainThreadMarker, symbols: &[&str]) -> Retained<NSSegmentedControl> {
    let group = NSSegmentedControl::new(mtm);
    group.setSegmentCount(symbols.len() as isize);
    group.setTrackingMode(NSSegmentSwitchTracking::Momentary);
    for (segment, name) in symbols.iter().enumerate() {
        set_segment_symbol(&group, segment as isize, name);
        group.setWidth_forSegment(SEGMENT_WIDTH, segment as isize);
    }
    group
}

fn set_segment_symbol(group: &NSSegmentedControl, segment: isize, name: &str) {
    let image = NSImage::imageWithSystemSymbolName_accessibilityDescription(
        &NSString::from_str(name),
        None,
    );
    group.setImage_forSegment(image.as_deref(), segment);
}

/// The rect of `segment` in `group`, where a popover for it points.
fn segment_rect(group: &NSSegmentedControl, segment: isize) -> NSRect {
    let bounds = group.bounds();
    NSRect::new(
        NSPoint::new(SEGMENT_WIDTH * segment as f64, bounds.origin.y),
        NSSize::new(SEGMENT_WIDTH, bounds.size.height),
    )
}

define_class!(
    /// The View menu's zoom items. They zoom the reader window only while it is
    /// the key window, since the menu is the app's, not the reader's.
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    struct ZoomMenu;

    impl ZoomMenu {
        #[unsafe(method(zoomIn:))]
        fn zoom_in(&self, _sender: Option<&AnyObject>) {
            zoom_reader(true);
        }

        #[unsafe(method(zoomOut:))]
        fn zoom_out(&self, _sender: Option<&AnyObject>) {
            zoom_reader(false);
        }
    }
);

impl ZoomMenu {
    fn new(mtm: MainThreadMarker) -> Retained<Self> {
        // SAFETY: `init` on a freshly allocated instance of this class.
        unsafe { msg_send![Self::alloc(mtm), init] }
    }
}

fn zoom_reader(larger: bool) {
    let key = READER_WINDOW.with(|slot| {
        slot.borrow()
            .as_ref()
            .is_some_and(|reader| reader.window.isKeyWindow())
    });
    if key {
        push_command(ReaderCommand::Zoom(larger));
    }
}

/// The menu bar: the app menu with Quit, and View with the zoom items on ⌘=
/// and ⌘-.
fn install_main_menu(mtm: MainThreadMarker, app: &NSApplication, zoom: &ZoomMenu) {
    let main = NSMenu::new(mtm);
    let app_menu = NSMenu::new(mtm);
    app_menu.addItem(&menu_item(mtm, "Завершить", sel!(terminate:), "q", None));
    main.addItem(&submenu_item(mtm, "", &app_menu));

    let view_menu = NSMenu::initWithTitle(NSMenu::alloc(mtm), &NSString::from_str("Вид"));
    let target: &AnyObject = zoom;
    view_menu.addItem(&menu_item(
        mtm,
        "Увеличить",
        sel!(zoomIn:),
        "=",
        Some(target),
    ));
    view_menu.addItem(&menu_item(
        mtm,
        "Уменьшить",
        sel!(zoomOut:),
        "-",
        Some(target),
    ));
    main.addItem(&submenu_item(mtm, "Вид", &view_menu));
    app.setMainMenu(Some(&main));
}

fn menu_item(
    mtm: MainThreadMarker,
    title: &str,
    action: Sel,
    key: &str,
    target: Option<&AnyObject>,
) -> Retained<NSMenuItem> {
    // SAFETY: the action is a selector the zoom menu or AppKit answers.
    let item = unsafe {
        NSMenuItem::initWithTitle_action_keyEquivalent(
            NSMenuItem::alloc(mtm),
            &NSString::from_str(title),
            Some(action),
            &NSString::from_str(key),
        )
    };
    // SAFETY: the target is the zoom menu, which `run` keeps alive for the app loop.
    unsafe { item.setTarget(target) };
    item
}

fn submenu_item(mtm: MainThreadMarker, title: &str, menu: &NSMenu) -> Retained<NSMenuItem> {
    // SAFETY: no action, so nothing is sent on a press.
    let item = unsafe {
        NSMenuItem::initWithTitle_action_keyEquivalent(
            NSMenuItem::alloc(mtm),
            &NSString::from_str(title),
            None,
            &NSString::from_str(""),
        )
    };
    item.setSubmenu(Some(menu));
    item
}

/// The book's title as a glass button: the icon of its file, then the title,
/// cut with an ellipsis past [`TITLE_MAX_WIDTH`].
fn title_button(mtm: MainThreadMarker, title: &str, path: &str) -> Retained<NSButton> {
    let button = NSButton::new(mtm);
    button.setBezelStyle(NSBezelStyle::Glass);
    button.setTitle(&NSString::from_str(title));
    let icon = NSWorkspace::sharedWorkspace().iconForFile(&NSString::from_str(path));
    icon.setSize(NSSize::new(18.0, 18.0));
    button.setImage(Some(&icon));
    button.setImagePosition(NSCellImagePosition::ImageLeading);
    if let Some(cell) = button.cell() {
        cell.setLineBreakMode(NSLineBreakMode::ByTruncatingTail);
    }
    // The toolbar sizes the item by its constraints, so the cap is one too.
    button.setTranslatesAutoresizingMaskIntoConstraints(false);
    button
        .widthAnchor()
        .constraintLessThanOrEqualToConstant(TITLE_MAX_WIDTH)
        .setActive(true);
    button
        .heightAnchor()
        .constraintEqualToConstant(TOOLBAR_ITEM_HEIGHT)
        .setActive(true);
    button
}

/// Opens a reader window for `request`, with the nsh app from `root`, closing
/// the reader window open before.
fn open_reader_window(
    mtm: MainThreadMarker,
    request: &ReaderOpen,
    root: fn() -> Root,
    components: fn(&AppView),
) {
    let frame = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(1000.0, 800.0));
    let style = NSWindowStyleMask::Titled
        | NSWindowStyleMask::FullSizeContentView
        | NSWindowStyleMask::Closable
        | NSWindowStyleMask::Miniaturizable
        | NSWindowStyleMask::Resizable;
    // SAFETY: NSWindow's designated initializer.
    let window = unsafe {
        NSWindow::initWithContentRect_styleMask_backing_defer(
            NSWindow::alloc(mtm),
            frame,
            style,
            NSBackingStoreType::Buffered,
            false,
        )
    };
    // SAFETY: the window is kept in `READER_WINDOW` until it is replaced.
    unsafe { window.setReleasedWhenClosed(false) };
    window.setTitle(&NSString::from_str(&request.title));

    let view = AppView::new(
        mtm,
        AppOptions {
            title: request.title.clone(),
            size: (1000, 800),
            ..Default::default()
        },
    );
    components(&view);
    view.load(App::Rust(root()));
    window.setContentView(Some(&view.view()));

    let toolbar_delegate = ReaderToolbar::new(mtm, &request.title, &request.path);
    let toolbar =
        NSToolbar::initWithIdentifier(NSToolbar::alloc(mtm), &NSString::from_str("imago.reader"));
    toolbar.setDelegate(Some(ProtocolObject::from_ref(&*toolbar_delegate)));
    toolbar.setDisplayMode(NSToolbarDisplayMode::IconOnly);
    toolbar.setCenteredItemIdentifiers(&NSSet::from_slice(&[&*NSString::from_str(
        "imago.reader.title",
    )]));
    window.setToolbar(Some(&toolbar));
    window.setToolbarStyle(NSWindowToolbarStyle::Unified);

    // The app draws under the transparent title bar; the toolbar carries the
    // reader's controls.
    window.setTitlebarAppearsTransparent(true);
    window.setTitleVisibility(NSWindowTitleVisibility::Hidden);
    window.center();
    window.makeKeyAndOrderFront(None);

    let previous = READER_WINDOW.with(|slot| {
        slot.borrow_mut().replace(ReaderWindow {
            window: window.clone(),
            _view: view,
            toolbar: toolbar_delegate,
            contents: None,
            search: None,
            settings: None,
        })
    });
    if let Some(previous) = previous {
        previous.window.close();
    }
}

/// A popover titled `title` listing `entries` as a source list, sized to
/// them up to a limit; a press on one opens its page.
fn list_popover(
    mtm: MainThreadMarker,
    title: &str,
    entries: Vec<Contents>,
    behavior: NSPopoverBehavior,
) -> (Retained<NSPopover>, Retained<ContentsTable>) {
    let rows = entries.len().clamp(1, POPOVER_MAX_ROWS);
    let size = NSSize::new(
        POPOVER_WIDTH,
        POPOVER_TITLE_HEIGHT + rows as f64 * POPOVER_ROW_HEIGHT + POPOVER_PADDING,
    );
    let table = ContentsTable::new(mtm, entries);
    table.ivars().table.setStyle(NSTableViewStyle::SourceList);
    table.ivars().table.setRowHeight(POPOVER_ROW_HEIGHT);
    let scroll = NSScrollView::new(mtm);
    scroll.setDocumentView(Some(&table.ivars().table));
    scroll.setBorderType(NSBorderType::NoBorder);
    scroll.setDrawsBackground(false);
    scroll.setFrame(NSRect::new(
        NSPoint::new(0.0, POPOVER_PADDING / 2.0),
        NSSize::new(
            size.width,
            size.height - POPOVER_TITLE_HEIGHT - POPOVER_PADDING,
        ),
    ));
    let heading = NSTextField::labelWithString(&NSString::from_str(title), mtm);
    heading.setAlignment(NSTextAlignment::Center);
    heading.setFrame(NSRect::new(
        NSPoint::new(0.0, size.height - POPOVER_TITLE_HEIGHT),
        NSSize::new(size.width, POPOVER_TITLE_HEIGHT),
    ));
    let root = NSView::new(mtm);
    root.setFrame(NSRect::new(NSPoint::new(0.0, 0.0), size));
    root.addSubview(&heading);
    root.addSubview(&scroll);
    let controller = NSViewController::new(mtm);
    controller.setView(&root);
    let popover = NSPopover::new(mtm);
    popover.setContentViewController(Some(&controller));
    popover.setContentSize(size);
    popover.setBehavior(behavior);
    (popover, table)
}

/// Shows a list popover titled `title` below the toolbar's `segment`, with
/// the entries of `list`.
fn show_reader_list(
    mtm: MainThreadMarker,
    anchor: &NSSegmentedControl,
    segment: isize,
    title: &str,
    list: &Mutex<Vec<Contents>>,
) {
    let (popover, table) =
        list_popover(mtm, title, lock(list).clone(), NSPopoverBehavior::Transient);
    popover.showRelativeToRect_ofView_preferredEdge(
        segment_rect(anchor, segment),
        anchor,
        NSRectEdge::MinY,
    );
    READER_WINDOW.with(|slot| {
        if let Some(reader) = slot.borrow_mut().as_mut() {
            reader.contents = Some((popover, table));
        }
    });
}

/// Shows the search popover below the search segment. Its field stays in it,
/// so it is built once and shown again after that.
fn show_reader_search(mtm: MainThreadMarker, anchor: &NSSegmentedControl, segment: isize) {
    READER_WINDOW.with(|slot| {
        let mut slot = slot.borrow_mut();
        let Some(reader) = slot.as_mut() else {
            return;
        };
        let field = reader.toolbar.ivars().search_field.clone();
        let panel = reader
            .search
            .get_or_insert_with(|| search_panel(mtm, field));
        panel.popover.showRelativeToRect_ofView_preferredEdge(
            segment_rect(anchor, segment),
            anchor,
            NSRectEdge::MinY,
        );
    });
}

/// The search popover: the field, then the hits below it. It is only as tall
/// as the field and the hits it has.
fn search_panel(mtm: MainThreadMarker, field: Retained<NSSearchField>) -> SearchPanel {
    let table = ContentsTable::new(mtm, Vec::new());
    table.ivars().table.setStyle(NSTableViewStyle::SourceList);
    let scroll = NSScrollView::new(mtm);
    scroll.setDocumentView(Some(&table.ivars().table));
    scroll.setBorderType(NSBorderType::NoBorder);
    scroll.setDrawsBackground(false);
    let root = NSView::new(mtm);
    root.addSubview(&field);
    root.addSubview(&scroll);
    let controller = NSViewController::new(mtm);
    controller.setView(&root);
    let popover = NSPopover::new(mtm);
    popover.setContentViewController(Some(&controller));
    popover.setBehavior(NSPopoverBehavior::Transient);
    let panel = SearchPanel {
        popover,
        root,
        field,
        scroll,
        table,
    };
    fit_search_panel(&panel, 0);
    panel
}

/// Sizes the search popover to its field and `rows` hits, at most
/// [`SEARCH_ROWS`] of them shown at once.
fn fit_search_panel(panel: &SearchPanel, rows: usize) {
    let list = rows.min(SEARCH_ROWS) as f64 * SEARCH_ROW_HEIGHT;
    let gap = if rows == 0 { 0.0 } else { SEARCH_GAP };
    let height = 2.0 * SEARCH_MARGIN + SEARCH_FIELD_HEIGHT + gap + list;
    let top = height - SEARCH_MARGIN - SEARCH_FIELD_HEIGHT;
    panel.root.setFrame(NSRect::new(
        NSPoint::new(0.0, 0.0),
        NSSize::new(SEARCH_WIDTH, height),
    ));
    panel.field.setFrame(NSRect::new(
        NSPoint::new(SEARCH_MARGIN, top),
        NSSize::new(SEARCH_WIDTH - 2.0 * SEARCH_MARGIN, SEARCH_FIELD_HEIGHT),
    ));
    panel.scroll.setFrame(NSRect::new(
        NSPoint::new(0.0, SEARCH_MARGIN),
        NSSize::new(SEARCH_WIDTH, list),
    ));
    panel.scroll.setHidden(rows == 0);
    panel
        .popover
        .setContentSize(NSSize::new(SEARCH_WIDTH, height));
}

/// Shows `hits` in the search popover, and fits it to them; empty clears it.
fn show_search_results(mtm: MainThreadMarker, hits: Vec<Contents>) {
    READER_WINDOW.with(|slot| {
        let mut slot = slot.borrow_mut();
        let Some(panel) = slot.as_mut().and_then(|reader| reader.search.as_mut()) else {
            return;
        };
        let rows = hits.len();
        let table = ContentsTable::new(mtm, hits);
        table.ivars().table.setStyle(NSTableViewStyle::SourceList);
        panel.scroll.setDocumentView(Some(&table.ivars().table));
        panel.table = table;
        fit_search_panel(panel, rows);
    });
}

/// Shows the settings popover below the settings button: the zoom with its
/// steps, and the tones as a segmented control.
fn show_reader_settings(mtm: MainThreadMarker, toolbar: &ReaderToolbar, segment: isize) {
    let Some(state) = lock(&READER_STATE).clone() else {
        return;
    };
    let target: &AnyObject = toolbar;
    let (width, height) = (280.0, 96.0);
    let root = NSView::new(mtm);
    root.setFrame(NSRect::new(
        NSPoint::new(0.0, 0.0),
        NSSize::new(width, height),
    ));

    let zoom_title = NSTextField::labelWithString(&NSString::from_str("Масштаб"), mtm);
    zoom_title.setFrame(NSRect::new(
        NSPoint::new(16.0, 56.0),
        NSSize::new(100.0, 20.0),
    ));
    root.addSubview(&zoom_title);

    let zoom_label = NSTextField::labelWithString(
        &NSString::from_str(&format!("{}%", state.zoom_percent)),
        mtm,
    );
    zoom_label.setAlignment(NSTextAlignment::Center);
    zoom_label.setFrame(NSRect::new(
        NSPoint::new(160.0, 56.0),
        NSSize::new(60.0, 20.0),
    ));
    root.addSubview(&zoom_label);

    let zoom_out = NSButton::new(mtm);
    zoom_out.setTitle(&NSString::from_str("−"));
    zoom_out.setBezelStyle(NSBezelStyle::Push);
    zoom_out.setFrame(NSRect::new(
        NSPoint::new(124.0, 52.0),
        NSSize::new(32.0, 28.0),
    ));
    zoom_out.setEnabled(state.zoom_enabled);
    unsafe {
        zoom_out.setTarget(Some(target));
        zoom_out.setAction(Some(sel!(zoomOut:)));
    }
    root.addSubview(&zoom_out);

    let zoom_in = NSButton::new(mtm);
    zoom_in.setTitle(&NSString::from_str("+"));
    zoom_in.setBezelStyle(NSBezelStyle::Push);
    zoom_in.setFrame(NSRect::new(
        NSPoint::new(224.0, 52.0),
        NSSize::new(32.0, 28.0),
    ));
    zoom_in.setEnabled(state.zoom_enabled);
    unsafe {
        zoom_in.setTarget(Some(target));
        zoom_in.setAction(Some(sel!(zoomIn:)));
    }
    root.addSubview(&zoom_in);

    let tones = NSSegmentedControl::new(mtm);
    tones.setSegmentCount(state.tones.len() as isize);
    for (index, label) in state.tones.iter().enumerate() {
        tones.setLabel_forSegment(&NSString::from_str(label), index as isize);
    }
    tones.setSelectedSegment(state.tone as isize);
    tones.setFrame(NSRect::new(
        NSPoint::new(16.0, 12.0),
        NSSize::new(width - 32.0, 28.0),
    ));
    // SAFETY: the target is the toolbar, which the reader window keeps alive.
    unsafe {
        tones.setTarget(Some(target));
        tones.setAction(Some(sel!(toneChanged:)));
    }
    root.addSubview(&tones);

    let controller = NSViewController::new(mtm);
    controller.setView(&root);
    let popover = NSPopover::new(mtm);
    popover.setContentViewController(Some(&controller));
    popover.setContentSize(NSSize::new(width, height));
    popover.setBehavior(NSPopoverBehavior::Transient);
    let anchor = toolbar.ivars().right.clone();
    popover.showRelativeToRect_ofView_preferredEdge(
        segment_rect(&anchor, segment),
        &anchor,
        NSRectEdge::MinY,
    );

    READER_WINDOW.with(|slot| {
        if let Some(reader) = slot.borrow_mut().as_mut() {
            reader.settings = Some((popover, zoom_label, zoom_out, zoom_in));
        }
    });
}

/// Shows the reading state the app last set on the reader window's toolbar,
/// and the zoom of an open settings popover.
fn sync_reader_toolbar() {
    let Some(state) = lock(&READER_STATE).clone() else {
        return;
    };
    READER_WINDOW.with(|slot| {
        let slot = slot.borrow();
        let Some(reader) = slot.as_ref() else {
            return;
        };
        let ivars = reader.toolbar.ivars();
        set_segment_symbol(
            &ivars.right,
            RIGHT_MODE,
            if state.text_mode { "book.fill" } else { "book" },
        );
        if let Some((_, zoom_label, zoom_out, zoom_in)) = &reader.settings {
            zoom_label.setStringValue(&NSString::from_str(&format!("{}%", state.zoom_percent)));
            zoom_out.setEnabled(state.zoom_enabled);
            zoom_in.setEnabled(state.zoom_enabled);
        }
    });
}

pub struct TickerIvars {
    /// The app each reader window loads.
    reader_root: fn() -> Root,
    /// Registers the native components of each reader window.
    components: fn(&AppView),
}

define_class!(
    /// Opens the reader windows [`open_reader`] asks for, on the main thread,
    /// from a timer.
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[ivars = TickerIvars]
    struct Ticker;

    impl Ticker {
        #[unsafe(method(tick:))]
        fn tick(&self, _timer: &NSTimer) {
            let mtm = MainThreadMarker::from(self);
            if let Some(hits) = lock(&SEARCH_RESULTS).take() {
                show_search_results(mtm, hits);
            }
            sync_reader_toolbar();
            let Some(request) = lock(&READER_OPEN).take() else {
                return;
            };
            open_reader_window(
                mtm,
                &request,
                self.ivars().reader_root,
                self.ivars().components,
            );
        }
    }
);

impl Ticker {
    fn new(mtm: MainThreadMarker, ivars: TickerIvars) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(ivars);
        // SAFETY: `init` on a freshly allocated instance of this class.
        unsafe { msg_send![super(this), init] }
    }
}
