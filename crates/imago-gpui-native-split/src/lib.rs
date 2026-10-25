//! Installs an AppKit Tahoe sidebar in a window created and owned by GPUI.
//!
//! This spike deliberately does not create an `NSApplication` or run loop.

#[cfg(target_os = "macos")]
mod macos {
    use block2::RcBlock;
    use imago_native_sidebar::Row;
    use objc2::{
        DefinedClass, MainThreadOnly, Message, define_class, msg_send, rc::Retained,
        runtime::ProtocolObject,
    };
    use objc2_app_kit::{
        NSAppearance, NSAppearanceCustomization, NSControlTextEditingDelegate, NSImage,
        NSImageView, NSLayoutConstraint, NSScrollView, NSSplitViewController, NSSplitViewItem,
        NSTableCellView, NSTableColumn, NSTableView, NSTableViewDataSource, NSTableViewDelegate,
        NSTableViewStyle, NSTextField, NSToolbar, NSToolbarDelegate, NSToolbarDisplayMode,
        NSToolbarFlexibleSpaceItemIdentifier, NSToolbarItem, NSToolbarItemIdentifier,
        NSToolbarSidebarTrackingSeparatorItemIdentifier, NSToolbarToggleSidebarItemIdentifier,
        NSView, NSViewController, NSVisualEffectBlendingMode, NSVisualEffectMaterial,
        NSVisualEffectState, NSVisualEffectView, NSWindow, NSWindowStyleMask,
        NSWindowTitleVisibility, NSWindowToolbarStyle,
    };
    use objc2_foundation::{
        MainThreadMarker, NSArray, NSIndexSet, NSNotification, NSObject, NSObjectProtocol,
        NSOperationQueue, NSRect, NSString,
    };
    use raw_window_handle::AppKitWindowHandle;
    use std::rc::Rc;

    /// Retains AppKit delegates whose Objective-C ownership is weak.
    pub struct NativeSplit {
        _split: Retained<Split>,
        _rows: Retained<Rows>,
        _toolbar_delegate: Retained<Toolbar>,
        window: Retained<NSWindow>,
    }

    impl NativeSplit {
        /// Set the AppKit appearance for light/dark screenshot checks.
        pub fn set_dark_appearance(&self, dark: bool) {
            let appearance = unsafe {
                let name = if dark {
                    objc2_app_kit::NSAppearanceNameDarkAqua
                } else {
                    objc2_app_kit::NSAppearanceNameAqua
                };
                NSAppearance::appearanceNamed(name)
            };
            self.window.setAppearance(appearance.as_deref());
        }
    }

    /// Replace the GPUI window content controller with a native split.
    ///
    /// # Safety
    /// `handle` must be the current window's live GPUIView handle. Call on the
    /// AppKit main thread after GPUI has attached the view to its window.
    pub unsafe fn install(
        handle: AppKitWindowHandle,
        rows: Vec<Row>,
        on_select: impl Fn(String) + 'static,
    ) -> NativeSplit {
        let mtm = MainThreadMarker::new().expect("AppKit install must run on main thread");
        // SAFETY: the caller guarantees this is a live GPUIView on the main thread.
        let gpui_view = unsafe { &*(handle.ns_view.as_ptr() as *const NSView) };
        let window = gpui_view
            .window()
            .expect("GPUIView is not attached to a window");

        let table = NSTableView::new(mtm);
        table.setStyle(NSTableViewStyle::SourceList);
        table.setHeaderView(None);
        table.setRowHeight(32.0);
        table.addTableColumn(&NSTableColumn::new(mtm));
        let source = Rows::new(
            mtm,
            rows,
            Rc::new(on_select),
            window.clone(),
            gpui_view.retain(),
        );
        unsafe {
            table.setDataSource(Some(ProtocolObject::from_ref(&*source)));
            table.setDelegate(Some(ProtocolObject::from_ref(&*source)));
        }
        if !source.ivars().rows.is_empty() {
            table.selectRowIndexes_byExtendingSelection(&NSIndexSet::indexSetWithIndex(0), false);
        }

        let scroll = NSScrollView::new(mtm);
        scroll.setDocumentView(Some(&table));
        scroll.setDrawsBackground(false);
        let effect = NSVisualEffectView::new(mtm);
        effect.setMaterial(NSVisualEffectMaterial::Sidebar);
        effect.setBlendingMode(NSVisualEffectBlendingMode::BehindWindow);
        effect.setState(NSVisualEffectState::FollowsWindowActiveState);
        effect.addSubview(&scroll);
        scroll.setTranslatesAutoresizingMaskIntoConstraints(false);
        let constraints = NSArray::from_retained_slice(&[
            scroll
                .leadingAnchor()
                .constraintEqualToAnchor(&effect.leadingAnchor()),
            scroll
                .trailingAnchor()
                .constraintEqualToAnchor(&effect.trailingAnchor()),
            scroll
                .topAnchor()
                .constraintEqualToAnchor(&effect.topAnchor()),
            scroll
                .bottomAnchor()
                .constraintEqualToAnchor(&effect.bottomAnchor()),
        ]);
        NSLayoutConstraint::activateConstraints(&constraints);

        let sidebar_controller = NSViewController::new(mtm);
        sidebar_controller.setView(&effect);
        let content_controller = NSViewController::new(mtm);
        content_controller.setView(gpui_view);
        let split = Split::new(mtm);
        let sidebar = NSSplitViewItem::sidebarWithViewController(&sidebar_controller);
        sidebar.setMinimumThickness(220.0);
        split.addSplitViewItem(&sidebar);
        split.addSplitViewItem(&NSSplitViewItem::splitViewItemWithViewController(
            &content_controller,
        ));
        window.setContentViewController(Some(&split));
        window.setStyleMask(window.styleMask() | NSWindowStyleMask::FullSizeContentView);
        window.setTitlebarAppearsTransparent(true);
        window.setTitleVisibility(NSWindowTitleVisibility::Hidden);
        window.setToolbarStyle(NSWindowToolbarStyle::Unified);

        let toolbar_delegate = Toolbar::new(mtm);
        let toolbar = NSToolbar::initWithIdentifier(
            NSToolbar::alloc(mtm),
            &NSString::from_str("imago.gpui.split"),
        );
        toolbar.setDelegate(Some(ProtocolObject::from_ref(&*toolbar_delegate)));
        toolbar.setDisplayMode(NSToolbarDisplayMode::IconOnly);
        window.setToolbar(Some(&toolbar));

        NativeSplit {
            _split: split,
            _rows: source,
            _toolbar_delegate: toolbar_delegate,
            window,
        }
    }

    struct RowsIvars {
        rows: Vec<Row>,
        on_select: Rc<dyn Fn(String)>,
        window: Retained<NSWindow>,
        gpui_view: Retained<NSView>,
    }

    define_class!(
        #[unsafe(super(NSObject))]
        #[thread_kind = MainThreadOnly]
        #[ivars = RowsIvars]
        struct Rows;
        unsafe impl NSObjectProtocol for Rows {}
        unsafe impl NSControlTextEditingDelegate for Rows {}
        unsafe impl NSTableViewDataSource for Rows {
            #[unsafe(method(numberOfRowsInTableView:))]
            fn number_of_rows(&self, _: &NSTableView) -> isize {
                self.ivars().rows.len() as isize
            }
        }
        unsafe impl NSTableViewDelegate for Rows {
            #[unsafe(method_id(tableView:viewForTableColumn:row:))]
            fn view_for_row(
                &self,
                _: &NSTableView,
                _: Option<&NSTableColumn>,
                row: isize,
            ) -> Option<Retained<NSView>> {
                let mtm = MainThreadMarker::from(self);
                usize::try_from(row)
                    .ok()
                    .and_then(|index| self.ivars().rows.get(index))
                    .map(|row| {
                        let cell = NSTableCellView::initWithFrame(
                            NSTableCellView::alloc(mtm),
                            NSRect::ZERO,
                        );
                        let icon = NSImageView::new(mtm);
                        if let Some(symbol) = row.symbol {
                            icon.setImage(
                                NSImage::imageWithSystemSymbolName_accessibilityDescription(
                                    &NSString::from_str(symbol),
                                    None,
                                )
                                .as_deref(),
                            );
                        }
                        let label =
                            NSTextField::labelWithString(&NSString::from_str(&row.label), mtm);
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
                    })
            }
            #[unsafe(method(tableViewSelectionDidChange:))]
            fn selection_changed(&self, notification: &NSNotification) {
                let Some(table) = notification
                    .object()
                    .and_then(|o| o.downcast::<NSTableView>().ok())
                else {
                    return;
                };
                let Ok(index) = usize::try_from(table.selectedRow()) else {
                    return;
                };
                let Some(row) = self.ivars().rows.get(index) else {
                    return;
                };
                let id = row.id.to_owned();
                let window = &self.ivars().window;
                let view = &self.ivars().gpui_view;
                window.makeFirstResponder(Some(view));
                let callback = self.ivars().on_select.clone();
                let operation = RcBlock::new(move || callback(id.clone()));
                unsafe { NSOperationQueue::mainQueue().addOperationWithBlock(&operation) };
            }
        }
    );

    impl Rows {
        fn new(
            mtm: MainThreadMarker,
            rows: Vec<Row>,
            on_select: Rc<dyn Fn(String)>,
            window: Retained<NSWindow>,
            gpui_view: Retained<NSView>,
        ) -> Retained<Self> {
            let this = Self::alloc(mtm).set_ivars(RowsIvars {
                rows,
                on_select,
                window,
                gpui_view,
            });
            unsafe { msg_send![super(this), init] }
        }
    }

    define_class!(
        #[unsafe(super(NSSplitViewController, NSViewController, NSObject))]
        #[thread_kind = MainThreadOnly]
        struct Split;
    );
    impl Split {
        fn new(mtm: MainThreadMarker) -> Retained<Self> {
            unsafe { msg_send![Self::alloc(mtm), init] }
        }
    }

    define_class!(
        #[unsafe(super(NSObject))]
        #[thread_kind = MainThreadOnly]
        struct Toolbar;
        unsafe impl NSObjectProtocol for Toolbar {}
        unsafe impl NSToolbarDelegate for Toolbar {
            #[unsafe(method_id(toolbarDefaultItemIdentifiers:))]
            fn default_items(&self, _: &NSToolbar) -> Retained<NSArray<NSToolbarItemIdentifier>> {
                standard_items()
            }
            #[unsafe(method_id(toolbarAllowedItemIdentifiers:))]
            fn allowed_items(&self, _: &NSToolbar) -> Retained<NSArray<NSToolbarItemIdentifier>> {
                standard_items()
            }
            #[unsafe(method_id(toolbar:itemForItemIdentifier:willBeInsertedIntoToolbar:))]
            fn item(
                &self,
                _: &NSToolbar,
                _: &NSToolbarItemIdentifier,
                _: bool,
            ) -> Option<Retained<NSToolbarItem>> {
                None
            }
        }
    );
    impl Toolbar {
        fn new(mtm: MainThreadMarker) -> Retained<Self> {
            unsafe { msg_send![Self::alloc(mtm), init] }
        }
    }

    fn standard_items() -> Retained<NSArray<NSToolbarItemIdentifier>> {
        unsafe {
            NSArray::from_slice(&[
                NSToolbarFlexibleSpaceItemIdentifier,
                NSToolbarToggleSidebarItemIdentifier,
                NSToolbarSidebarTrackingSeparatorItemIdentifier,
                NSToolbarFlexibleSpaceItemIdentifier,
            ])
        }
    }
}

#[cfg(target_os = "macos")]
pub use macos::{NativeSplit, install};
