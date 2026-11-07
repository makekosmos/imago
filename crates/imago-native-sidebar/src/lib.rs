//! A sidebar that is a native control where the host has one. On macOS it is
//! an AppKit source list in the sidebar material (`NSTableView` `.sourceList`,
//! `NSVisualEffectView` `.sidebar`, SF Symbols, push buttons in a footer), and
//! [`components`] registers it. Elsewhere nothing is registered, so
//! [`native_registered`] is false and the app draws its own sidebar with NSH.
//!
//! The app registers [`components`] in its `AppOptions`, builds rows with
//! [`SidebarSpec`] and places [`native_sidebar`] in its layout.

use nsh_core::prelude::*;
use nsh_core::JsonValue;
use serde_json::json;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};

/// The name the native sidebar is registered as.
pub const NATIVE: &str = "imago.Sidebar";

static REGISTERED: AtomicBool = AtomicBool::new(false);

/// One sidebar row: its id is reported on press, its symbol is an SF Symbol name.
#[derive(Clone, Debug, PartialEq)]
pub struct Row {
    pub id: &'static str,
    pub label: String,
    pub symbol: Option<&'static str>,
}

/// What the sidebar shows. Closures are read again when their signals change.
pub struct SidebarSpec {
    /// The scrolling rows.
    pub items: Rc<dyn Fn() -> Vec<Row>>,
    /// Buttons pinned under the rows.
    pub footer: Vec<Row>,
    /// The highlighted row's id, or `None` when no row is highlighted.
    pub selected: Rc<dyn Fn() -> Option<&'static str>>,
    /// A line of small text under the footer.
    pub caption: Rc<dyn Fn() -> String>,
    /// Whether the sidebar uses the dark appearance.
    pub dark: Rc<dyn Fn() -> bool>,
    /// Called with the id of a pressed row or footer button.
    pub press: Rc<dyn Fn(&str)>,
}

/// Whether [`components`] has been called, so the native sidebar is available.
/// The runners of `nsh run` and `nsh package` do not register it.
pub fn native_registered() -> bool {
    REGISTERED.load(Ordering::Relaxed)
}

/// The sidebar as the native component, for [`components`]' host.
pub fn native_sidebar(spec: SidebarSpec) -> NativeView {
    let SidebarSpec {
        items,
        footer,
        selected,
        caption,
        dark,
        press,
    } = spec;
    native(NATIVE)
        .prop("items", move || rows_json(&items()))
        .prop("footer", rows_json(&footer))
        .prop("selected", move || json!(selected()))
        .prop("caption", move || json!(caption()))
        .prop("dark", move || json!(dark()))
        .flex_1()
        .on_native_event(move |event: &NativeEvent| {
            if let Some(id) = event.data.get("id").and_then(JsonValue::as_str) {
                press(id);
            }
        })
}

fn rows_json(rows: &[Row]) -> JsonValue {
    JsonValue::Array(
        rows.iter()
            .map(|row| json!({ "id": row.id, "label": row.label, "symbol": row.symbol }))
            .collect(),
    )
}

/// The factory registrations the macOS app passes in its options.
#[cfg(target_os = "macos")]
pub fn components() -> nsh_platform_macos::Components {
    let components = nsh_platform_macos::Components::default();
    let factory: nsh_platform_macos::ComponentFactory =
        Rc::new(|ctx| Box::new(appkit::Sidebar::new(ctx)));
    components.register(NATIVE, factory);
    REGISTERED.store(true, Ordering::Relaxed);
    components
}

#[cfg(target_os = "macos")]
mod appkit {
    use nsh_core::JsonValue;
    use nsh_platform_macos::{
        objc2::{
            define_class, msg_send,
            rc::Retained,
            runtime::{AnyObject, ProtocolObject},
            sel, DefinedClass, MainThreadOnly,
        },
        objc2_app_kit::{
            NSAppearance, NSAppearanceCustomization, NSAppearanceNameAqua,
            NSAppearanceNameDarkAqua, NSButton, NSCellImagePosition, NSControlTextEditingDelegate,
            NSFont, NSImage, NSImageView, NSLayoutAttribute, NSLayoutConstraint, NSScrollView,
            NSStackView, NSTableCellView, NSTableColumn, NSTableView,
            NSTableViewColumnAutoresizingStyle, NSTableViewDataSource, NSTableViewDelegate,
            NSTableViewStyle, NSTextAlignment, NSTextField, NSUserInterfaceLayoutOrientation,
            NSView, NSVisualEffectBlendingMode, NSVisualEffectMaterial, NSVisualEffectState,
            NSVisualEffectView,
        },
        objc2_foundation::{
            MainThreadMarker, NSArray, NSIndexSet, NSNotification, NSObject, NSObjectProtocol,
            NSRect, NSString,
        },
        ComponentContext, NativeComponent,
    };
    use serde_json::json;
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;

    #[derive(Default)]
    struct Model {
        items: Vec<Row>,
        footer: Vec<Row>,
        caption: String,
        selected: String,
        dark: bool,
    }

    #[derive(Clone)]
    struct Row {
        id: String,
        label: String,
        symbol: Option<String>,
    }

    pub struct Sidebar {
        ctx: ComponentContext,
        mtm: MainThreadMarker,
        model: Rc<RefCell<Model>>,
        root: Retained<NSVisualEffectView>,
        table: Retained<NSTableView>,
        table_delegate: Retained<Table>,
        footer: Retained<NSStackView>,
        caption: Retained<NSTextField>,
        // Buttons send to their targets weakly, so the targets live here.
        targets: Vec<Retained<PressTarget>>,
    }

    impl Sidebar {
        pub fn new(ctx: ComponentContext) -> Self {
            let mtm = MainThreadMarker::new().expect("components are made on the main thread");
            let model = Rc::new(RefCell::new(Model::default()));
            let root =
                NSVisualEffectView::initWithFrame(NSVisualEffectView::alloc(mtm), NSRect::ZERO);
            root.setMaterial(NSVisualEffectMaterial::Sidebar);
            root.setBlendingMode(NSVisualEffectBlendingMode::BehindWindow);
            root.setState(NSVisualEffectState::FollowsWindowActiveState);

            let footer = stack(mtm, NSLayoutAttribute::Leading);
            let caption = NSTextField::labelWithString(&NSString::from_str(""), mtm);
            caption.setFont(Some(&NSFont::systemFontOfSize(11.0)));
            caption.setTranslatesAutoresizingMaskIntoConstraints(false);

            let table = NSTableView::new(mtm);
            table.setStyle(NSTableViewStyle::SourceList);
            table.setHeaderView(None);
            table.setRowHeight(32.0);
            table.setColumnAutoresizingStyle(
                NSTableViewColumnAutoresizingStyle::LastColumnOnlyAutoresizingStyle,
            );
            let column = NSTableColumn::new(mtm);
            table.addTableColumn(&column);
            let scroll = NSScrollView::new(mtm);
            scroll.setDocumentView(Some(&table));
            scroll.setDrawsBackground(false);
            scroll.setHasVerticalScroller(false);
            scroll.setTranslatesAutoresizingMaskIntoConstraints(false);

            let table_delegate = Table::new(
                mtm,
                TableIvars {
                    model: model.clone(),
                    ctx: ctx.clone(),
                    syncing: Cell::new(false),
                },
            );
            let data_source: &ProtocolObject<dyn NSTableViewDataSource> =
                ProtocolObject::from_ref(&*table_delegate);
            let delegate: &ProtocolObject<dyn NSTableViewDelegate> =
                ProtocolObject::from_ref(&*table_delegate);
            // SAFETY: the delegate outlives the table (kept in `table_delegate`),
            // and the table holds its data source and delegate weakly.
            unsafe {
                table.setDataSource(Some(data_source));
                table.setDelegate(Some(delegate));
            }

            root.addSubview(&scroll);
            root.addSubview(&footer);
            root.addSubview(&caption);
            let constraints = NSArray::from_retained_slice(&[
                scroll
                    .leadingAnchor()
                    .constraintEqualToAnchor_constant(&root.leadingAnchor(), 8.0),
                scroll
                    .trailingAnchor()
                    .constraintEqualToAnchor_constant(&root.trailingAnchor(), -8.0),
                scroll
                    .topAnchor()
                    .constraintEqualToAnchor_constant(&root.topAnchor(), 16.0),
                scroll
                    .bottomAnchor()
                    .constraintEqualToAnchor_constant(&footer.topAnchor(), -8.0),
                footer
                    .leadingAnchor()
                    .constraintEqualToAnchor_constant(&root.leadingAnchor(), 12.0),
                footer
                    .bottomAnchor()
                    .constraintEqualToAnchor_constant(&caption.topAnchor(), -8.0),
                caption
                    .leadingAnchor()
                    .constraintEqualToAnchor_constant(&root.leadingAnchor(), 12.0),
                caption
                    .bottomAnchor()
                    .constraintEqualToAnchor_constant(&root.bottomAnchor(), -16.0),
            ]);
            NSLayoutConstraint::activateConstraints(&constraints);

            Self {
                ctx,
                mtm,
                model,
                root,
                table,
                table_delegate,
                footer,
                caption,
                targets: Vec::new(),
            }
        }

        fn rebuild(&mut self) {
            let appearance = if self.model.borrow().dark {
                // SAFETY: AppKit's appearance name constants.
                unsafe { NSAppearanceNameDarkAqua }
            } else {
                // SAFETY: AppKit's appearance name constants.
                unsafe { NSAppearanceNameAqua }
            };
            self.root
                .setAppearance(NSAppearance::appearanceNamed(appearance).as_deref());

            let footer = self.model.borrow().footer.clone();
            let caption = self.model.borrow().caption.clone();
            self.caption.setStringValue(&NSString::from_str(&caption));
            sync(self.table_delegate.ivars(), &self.table);

            self.targets.clear();
            let mut views = Vec::new();
            for row in &footer {
                views.push(self.button(row));
            }
            fill(&self.footer, views);
        }

        fn button(&mut self, row: &Row) -> Retained<NSView> {
            let target = PressTarget::new(self.mtm, row.id.clone(), self.ctx.clone());
            // SAFETY: the target outlives the button (kept in `targets`) and the
            // action is a method `PressTarget` implements.
            let button = unsafe {
                NSButton::buttonWithTitle_target_action(
                    &NSString::from_str(&row.label),
                    Some(&target),
                    Some(sel!(press:)),
                    self.mtm,
                )
            };
            button.setBordered(false);
            button.setAlignment(NSTextAlignment::Left);
            button.setImagePosition(NSCellImagePosition::ImageLeading);
            button.setFont(Some(&NSFont::systemFontOfSize(13.0)));
            if let Some(image) = symbol(row) {
                button.setImage(Some(&image));
            }
            self.targets.push(target);
            Retained::into_super(Retained::into_super(button))
        }
    }

    impl NativeComponent for Sidebar {
        fn view(&self) -> Retained<NSView> {
            Retained::into_super(self.root.clone())
        }

        fn set_prop(&mut self, name: &str, value: Option<&JsonValue>) {
            {
                let mut model = self.model.borrow_mut();
                match name {
                    "items" => model.items = rows(value),
                    "footer" => model.footer = rows(value),
                    "caption" => model.caption = text(value),
                    "selected" => model.selected = text(value),
                    "dark" => model.dark = value.and_then(JsonValue::as_bool).unwrap_or(false),
                    _ => return,
                }
            }
            self.rebuild();
        }
    }

    /// Shows the model's selected row in the table without reporting it as a press.
    fn sync(ivars: &TableIvars, table: &NSTableView) {
        ivars.syncing.set(true);
        table.reloadData();
        {
            let model = ivars.model.borrow();
            match model.items.iter().position(|row| row.id == model.selected) {
                Some(row) => {
                    let rows = NSIndexSet::indexSetWithIndex(row);
                    table.selectRowIndexes_byExtendingSelection(&rows, false);
                }
                // SAFETY: no sender, as `deselectAll:` takes it.
                None => unsafe { table.deselectAll(None) },
            }
        }
        ivars.syncing.set(false);
    }

    /// One row's cell: its symbol and its label, in the source list's style.
    fn cell(row: &Row, mtm: MainThreadMarker) -> Retained<NSView> {
        let cell = NSTableCellView::initWithFrame(NSTableCellView::alloc(mtm), NSRect::ZERO);
        let icon = NSImageView::new(mtm);
        if let Some(image) = symbol(row) {
            icon.setImage(Some(&image));
        }
        let label = NSTextField::labelWithString(&NSString::from_str(&row.label), mtm);
        label.setFont(Some(&NSFont::systemFontOfSize(13.0)));
        icon.setTranslatesAutoresizingMaskIntoConstraints(false);
        label.setTranslatesAutoresizingMaskIntoConstraints(false);
        cell.addSubview(&icon);
        cell.addSubview(&label);
        // SAFETY: both are subviews of `cell`, which keeps them for as long as it lives.
        unsafe {
            cell.setImageView(Some(&icon));
            cell.setTextField(Some(&label));
        }
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
                .trailingAnchor()
                .constraintLessThanOrEqualToAnchor_constant(&cell.trailingAnchor(), -8.0),
            label
                .centerYAnchor()
                .constraintEqualToAnchor(&cell.centerYAnchor()),
        ]);
        NSLayoutConstraint::activateConstraints(&constraints);
        Retained::into_super(cell)
    }

    fn symbol(row: &Row) -> Option<Retained<NSImage>> {
        let name = row.symbol.as_deref()?;
        NSImage::imageWithSystemSymbolName_accessibilityDescription(&NSString::from_str(name), None)
    }

    fn stack(mtm: MainThreadMarker, alignment: NSLayoutAttribute) -> Retained<NSStackView> {
        let stack = NSStackView::new(mtm);
        stack.setOrientation(NSUserInterfaceLayoutOrientation::Vertical);
        stack.setAlignment(alignment);
        stack.setSpacing(4.0);
        stack.setTranslatesAutoresizingMaskIntoConstraints(false);
        stack
    }

    /// Replaces the stack's arranged views with `views`.
    fn fill(stack: &NSStackView, views: Vec<Retained<NSView>>) {
        for old in stack.arrangedSubviews().iter() {
            stack.removeArrangedSubview(&old);
            old.removeFromSuperview();
        }
        for view in &views {
            stack.addArrangedSubview(view);
        }
    }

    fn text(value: Option<&JsonValue>) -> String {
        value
            .and_then(JsonValue::as_str)
            .unwrap_or_default()
            .to_owned()
    }

    fn rows(value: Option<&JsonValue>) -> Vec<Row> {
        value
            .and_then(JsonValue::as_array)
            .map(|rows| {
                rows.iter()
                    .filter_map(|row| {
                        Some(Row {
                            id: row.get("id")?.as_str()?.to_owned(),
                            label: row.get("label")?.as_str()?.to_owned(),
                            symbol: row
                                .get("symbol")
                                .and_then(JsonValue::as_str)
                                .map(str::to_owned),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    struct TableIvars {
        model: Rc<RefCell<Model>>,
        ctx: ComponentContext,
        /// Set while the model writes the selection, so it is not a press.
        syncing: Cell<bool>,
    }

    define_class!(
        #[unsafe(super(NSObject))]
        #[thread_kind = MainThreadOnly]
        #[ivars = TableIvars]
        struct Table;

        unsafe impl NSObjectProtocol for Table {}

        unsafe impl NSTableViewDataSource for Table {
            #[unsafe(method(numberOfRowsInTableView:))]
            fn number_of_rows(&self, _table: &NSTableView) -> isize {
                self.ivars().model.borrow().items.len() as isize
            }
        }

        unsafe impl NSControlTextEditingDelegate for Table {}

        unsafe impl NSTableViewDelegate for Table {
            #[unsafe(method_id(tableView:viewForTableColumn:row:))]
            fn view_for_row(
                &self,
                _table: &NSTableView,
                _column: Option<&NSTableColumn>,
                row: isize,
            ) -> Option<Retained<NSView>> {
                let model = self.ivars().model.borrow();
                let mtm = MainThreadMarker::from(self);
                model.items.get(row as usize).map(|row| cell(row, mtm))
            }

            #[unsafe(method(tableViewSelectionDidChange:))]
            fn selection_changed(&self, notification: &NSNotification) {
                let ivars = self.ivars();
                if ivars.syncing.get() {
                    return;
                }
                let Some(table) = notification
                    .object()
                    .and_then(|object| object.downcast::<NSTableView>().ok())
                else {
                    return;
                };
                let row = table.selectedRow();
                let id = ivars
                    .model
                    .borrow()
                    .items
                    .get(row as usize)
                    .map(|row| row.id.clone());
                if let Some(id) = id {
                    ivars.ctx.emit("press", json!({ "id": id }));
                }
                // A row the app does not select (the theme) goes back to the app's selection.
                sync(ivars, &table);
            }
        }
    );

    impl Table {
        fn new(mtm: MainThreadMarker, ivars: TableIvars) -> Retained<Self> {
            let this = Self::alloc(mtm).set_ivars(ivars);
            // SAFETY: `init` on a freshly allocated instance of this class.
            unsafe { msg_send![super(this), init] }
        }
    }

    struct PressIvars {
        id: String,
        ctx: ComponentContext,
    }

    define_class!(
        #[unsafe(super(NSObject))]
        #[thread_kind = MainThreadOnly]
        #[ivars = PressIvars]
        struct PressTarget;

        unsafe impl NSObjectProtocol for PressTarget {}

        impl PressTarget {
            #[unsafe(method(press:))]
            fn press(&self, _sender: &AnyObject) {
                let ivars = self.ivars();
                ivars.ctx.emit("press", json!({ "id": ivars.id }));
            }
        }
    );

    impl PressTarget {
        fn new(mtm: MainThreadMarker, id: String, ctx: ComponentContext) -> Retained<Self> {
            let this = Self::alloc(mtm).set_ivars(PressIvars { id, ctx });
            // SAFETY: `init` on a freshly allocated instance of this class.
            unsafe { msg_send![super(this), init] }
        }
    }
}
