// A native macOS toolbar (NSToolbar) for a wxFrame, with what the toolbar of
// wxWidgets 3.3 does not give on the Mac:
// - stable item identifiers, so macOS keeps the user's customization between
//   runs (wx names its items after memory addresses);
// - NSMenuToolbarItem: a button with a menu and its small arrow ("16 ⌄"),
//   with or without a primary action;
// - NSToolbarItemGroup: items that stay together (one capsule, one unit in
//   the customization palette);
// - the customization palette and the "Show Toolbar" switch;
// - the window's toolbar style (unified compact...).
//
// Clicks reach the frame as wxEVT_MENU commands with the item's id, like the
// menu bar's, so one handler serves both.

#import <AppKit/AppKit.h>
#include "../include/wxdragon.h"
#include <wx/toplevel.h>
#include <wx/event.h>

// What an item is; items are made from it whenever AppKit asks (the toolbar
// and the customization palette each need their own instance).
@interface WxdToolbarItemSpec : NSObject
@property(nonatomic, copy) NSString* identifier;
@property(nonatomic) int kind;       // 0 button, 1 menu button, 2 group
@property(nonatomic) int command;    // -1: none (a menu button opens its menu)
@property(nonatomic, copy) NSString* label;
@property(nonatomic, copy) NSString* symbol;   // SF Symbol name, may be nil
@property(nonatomic, copy) NSString* title;    // text shown in the button, may be nil
@property(nonatomic, copy) NSString* tooltip;
@property(nonatomic) BOOL enabled;
@property(nonatomic, strong) NSArray* menuEntries;  // of NSDictionary: title, command, flags
@property(nonatomic, strong) NSArray<NSString*>* members;  // a group's items
@end

@implementation WxdToolbarItemSpec
@end

@interface WxdToolbarController : NSObject <NSToolbarDelegate>
@property(nonatomic) wxTopLevelWindow* frame;
@property(nonatomic, strong) NSMutableDictionary<NSString*, WxdToolbarItemSpec*>* specs;
@property(nonatomic, strong) NSMutableArray<NSString*>* order;      // as added
@property(nonatomic, strong) NSArray<NSString*>* defaults;
@property(nonatomic, strong) NSToolbar* toolbar;
@end

static const int kFlagChecked = 1;
static const int kFlagDisabled = 2;
static const int kFlagSeparator = 4;

static NSString* WxdSpaceId(NSString* name)
{
    if ([name isEqualToString:@"space"])
        return NSToolbarSpaceItemIdentifier;
    if ([name isEqualToString:@"flexible-space"])
        return NSToolbarFlexibleSpaceItemIdentifier;
    return name;
}

@implementation WxdToolbarController

- (instancetype)init
{
    if (self = [super init]) {
        _specs = [NSMutableDictionary dictionary];
        _order = [NSMutableArray array];
        _defaults = @[];
    }
    return self;
}

- (void)send:(int)command
{
    if (!_frame || command < 0)
        return;
    wxCommandEvent event(wxEVT_MENU, command);
    event.SetEventObject(_frame);
    _frame->GetEventHandler()->ProcessEvent(event);
}

- (void)itemClicked:(id)sender
{
    [self send:(int)[sender tag]];
}

- (NSMenu*)menuFor:(WxdToolbarItemSpec*)spec
{
    NSMenu* menu = [[NSMenu alloc] initWithTitle:spec.label ?: @""];
    menu.autoenablesItems = NO;
    for (NSDictionary* e in spec.menuEntries) {
        int flags = [e[@"flags"] intValue];
        if (flags & kFlagSeparator) {
            [menu addItem:[NSMenuItem separatorItem]];
            continue;
        }
        NSMenuItem* item = [[NSMenuItem alloc] initWithTitle:e[@"title"]
                                                      action:@selector(itemClicked:)
                                               keyEquivalent:@""];
        item.target = self;
        item.tag = [e[@"command"] intValue];
        item.state = (flags & kFlagChecked) ? NSControlStateValueOn : NSControlStateValueOff;
        item.enabled = !(flags & kFlagDisabled);
        [menu addItem:item];
    }
    return menu;
}

- (NSToolbarItem*)makeItem:(WxdToolbarItemSpec*)spec
{
    NSToolbarItem* item = nil;
    if (spec.kind == 2) {
        NSMutableArray* subitems = [NSMutableArray array];
        for (NSString* member in spec.members) {
            WxdToolbarItemSpec* sub = _specs[member];
            if (sub && sub.kind != 2)
                [subitems addObject:[self makeItem:sub]];
        }
        NSToolbarItemGroup* group = [[NSToolbarItemGroup alloc] initWithItemIdentifier:spec.identifier];
        group.subitems = subitems;
        item = group;
    } else if (spec.kind == 1) {
        NSMenuToolbarItem* menuItem = [[NSMenuToolbarItem alloc] initWithItemIdentifier:spec.identifier];
        menuItem.menu = [self menuFor:spec];
        menuItem.showsIndicator = YES;
        item = menuItem;
    } else {
        item = [[NSToolbarItem alloc] initWithItemIdentifier:spec.identifier];
    }
    item.label = spec.label ?: @"";
    item.paletteLabel = spec.label ?: @"";
    if (spec.tooltip.length)
        item.toolTip = spec.tooltip;
    if (spec.kind != 2) {
        if (spec.symbol.length)
            item.image = [NSImage imageWithSystemSymbolName:spec.symbol accessibilityDescription:spec.label];
        if (spec.title.length)
            item.title = spec.title;
        item.bordered = YES;
        item.autovalidates = NO;
        item.enabled = spec.enabled;
        if (spec.command >= 0) {
            item.target = self;
            item.action = @selector(itemClicked:);
            item.tag = spec.command;
        }
    }
    return item;
}

// The live items with this identifier (inside groups too).
- (void)forLive:(NSString*)identifier do:(void (^)(NSToolbarItem*))f
{
    for (NSToolbarItem* item in _toolbar.items) {
        if ([item.itemIdentifier isEqualToString:identifier])
            f(item);
        if ([item isKindOfClass:[NSToolbarItemGroup class]])
            for (NSToolbarItem* sub in ((NSToolbarItemGroup*)item).subitems)
                if ([sub.itemIdentifier isEqualToString:identifier])
                    f(sub);
    }
}

// The identifiers that stand alone in the toolbar (not those inside a group).
- (NSArray<NSString*>*)topLevel
{
    NSMutableSet* inGroups = [NSMutableSet set];
    for (NSString* ident in _order) {
        WxdToolbarItemSpec* spec = _specs[ident];
        if (spec.kind == 2)
            [inGroups addObjectsFromArray:spec.members];
    }
    NSMutableArray* result = [NSMutableArray array];
    for (NSString* ident in _order)
        if (![inGroups containsObject:ident])
            [result addObject:ident];
    return result;
}

- (NSToolbarItem*)toolbar:(NSToolbar*)toolbar
        itemForItemIdentifier:(NSToolbarItemIdentifier)identifier
    willBeInsertedIntoToolbar:(BOOL)flag
{
    WxdToolbarItemSpec* spec = _specs[identifier];
    return spec ? [self makeItem:spec] : nil;
}

- (NSArray<NSToolbarItemIdentifier>*)toolbarDefaultItemIdentifiers:(NSToolbar*)toolbar
{
    return _defaults;
}

- (NSArray<NSToolbarItemIdentifier>*)toolbarAllowedItemIdentifiers:(NSToolbar*)toolbar
{
    NSMutableArray* allowed = [[self topLevel] mutableCopy];
    [allowed addObject:NSToolbarSpaceItemIdentifier];
    [allowed addObject:NSToolbarFlexibleSpaceItemIdentifier];
    return allowed;
}

@end

static WxdToolbarController* Controller(void* handle)
{
    return (__bridge WxdToolbarController*)handle;
}

static NSString* Str(const char* s)
{
    return s ? [NSString stringWithUTF8String:s] : nil;
}

static WxdToolbarItemSpec* AddSpec(void* handle, const char* identifier, int kind, int command,
                                   const char* label, const char* symbol, const char* title,
                                   const char* tooltip)
{
    WxdToolbarController* c = Controller(handle);
    if (!c || !identifier)
        return nil;
    WxdToolbarItemSpec* spec = [WxdToolbarItemSpec new];
    spec.identifier = Str(identifier);
    spec.kind = kind;
    spec.command = command;
    spec.label = Str(label);
    spec.symbol = Str(symbol);
    spec.title = Str(title);
    spec.tooltip = Str(tooltip);
    spec.enabled = YES;
    spec.menuEntries = @[];
    c.specs[spec.identifier] = spec;
    [c.order addObject:spec.identifier];
    return spec;
}

extern "C" {

WXD_EXPORTED void*
wxd_MacToolbar_Create(wxd_Window_t* frame, const char* identifier)
{
    wxWindow* window = reinterpret_cast<wxWindow*>(frame);
    wxTopLevelWindow* top = window ? wxDynamicCast(window, wxTopLevelWindow) : nullptr;
    if (!top || !identifier)
        return nullptr;
    NSWindow* nswindow = (NSWindow*)top->GetWXWindow();
    if (!nswindow)
        return nullptr;
    WxdToolbarController* c = [WxdToolbarController new];
    c.frame = top;
    NSToolbar* toolbar = [[NSToolbar alloc] initWithIdentifier:Str(identifier)];
    // NSToolbar keeps its delegate weakly: the controller (which holds the
    // toolbar) is kept by the frame and let go when the frame is destroyed.
    toolbar.delegate = c;
    c.toolbar = toolbar;
    void* handle = (void*)CFBridgingRetain(c);
    top->Bind(wxEVT_DESTROY, [top, handle](wxWindowDestroyEvent& event) {
        if (event.GetEventObject() == top) {
            WxdToolbarController* dying = (__bridge WxdToolbarController*)handle;
            dying.toolbar.delegate = nil;
            dying.frame = nullptr;
            CFBridgingRelease(handle);
        }
        event.Skip();
    });
    return handle;
}

WXD_EXPORTED void
wxd_MacToolbar_AddButton(void* tb, const char* identifier, int command, const char* label,
                         const char* symbol, const char* tooltip)
{
    AddSpec(tb, identifier, 0, command, label, symbol, nullptr, tooltip);
}

WXD_EXPORTED void
wxd_MacToolbar_AddMenuButton(void* tb, const char* identifier, int command, const char* label,
                             const char* symbol, const char* title, const char* tooltip)
{
    AddSpec(tb, identifier, 1, command, label, symbol, title, tooltip);
}

WXD_EXPORTED void
wxd_MacToolbar_AddGroup(void* tb, const char* identifier, const char* label, int count,
                        const char* const* members)
{
    WxdToolbarItemSpec* spec = AddSpec(tb, identifier, 2, -1, label, nullptr, nullptr, nullptr);
    if (!spec)
        return;
    NSMutableArray* list = [NSMutableArray array];
    for (int i = 0; i < count; ++i)
        if (members[i])
            [list addObject:Str(members[i])];
    spec.members = list;
}

WXD_EXPORTED void
wxd_MacToolbar_SetDefaultItems(void* tb, int count, const char* const* identifiers)
{
    WxdToolbarController* c = Controller(tb);
    if (!c)
        return;
    NSMutableArray* list = [NSMutableArray array];
    for (int i = 0; i < count; ++i)
        if (identifiers[i])
            [list addObject:WxdSpaceId(Str(identifiers[i]))];
    c.defaults = list;
}

// style: 0 automatic, 1 expanded, 2 preference, 3 unified, 4 unified compact
// (NSWindowToolbarStyle).
WXD_EXPORTED void
wxd_MacToolbar_Install(void* tb, int style, bool customizable)
{
    WxdToolbarController* c = Controller(tb);
    if (!c || !c.toolbar || !c.frame)
        return;
    NSWindow* window = (NSWindow*)c.frame->GetWXWindow();
    NSToolbar* toolbar = c.toolbar;
    toolbar.displayMode = NSToolbarDisplayModeIconOnly;
    toolbar.allowsUserCustomization = customizable;
    // Saved under the toolbar's identifier in the app's preferences.
    toolbar.autosavesConfiguration = customizable;
    if (@available(macOS 11.0, *))
        window.toolbarStyle = (NSWindowToolbarStyle)style;
    window.toolbar = toolbar;
}

WXD_EXPORTED void
wxd_MacToolbar_SetEnabled(void* tb, const char* identifier, bool enabled)
{
    WxdToolbarController* c = Controller(tb);
    NSString* ident = Str(identifier);
    WxdToolbarItemSpec* spec = c ? c.specs[ident] : nil;
    if (!spec || spec.enabled == enabled)
        return;
    spec.enabled = enabled;
    [c forLive:ident do:^(NSToolbarItem* item) { item.enabled = enabled; }];
}

WXD_EXPORTED void
wxd_MacToolbar_SetTitle(void* tb, const char* identifier, const char* title)
{
    WxdToolbarController* c = Controller(tb);
    NSString* ident = Str(identifier);
    WxdToolbarItemSpec* spec = c ? c.specs[ident] : nil;
    NSString* text = Str(title) ?: @"";
    if (!spec || [spec.title isEqualToString:text])
        return;
    spec.title = text;
    [c forLive:ident do:^(NSToolbarItem* item) { item.title = text; }];
}

// flags: 1 checked, 2 disabled, 4 separator.
WXD_EXPORTED void
wxd_MacToolbar_SetMenu(void* tb, const char* identifier, int count, const int* commands,
                       const char* const* titles, const int* flags)
{
    WxdToolbarController* c = Controller(tb);
    NSString* ident = Str(identifier);
    WxdToolbarItemSpec* spec = c ? c.specs[ident] : nil;
    if (!spec || spec.kind != 1)
        return;
    NSMutableArray* entries = [NSMutableArray array];
    for (int i = 0; i < count; ++i)
        [entries addObject:@{
            @"title" : Str(titles[i]) ?: @"",
            @"command" : @(commands[i]),
            @"flags" : @(flags[i]),
        }];
    spec.menuEntries = entries;
    [c forLive:ident do:^(NSToolbarItem* item) {
        if ([item isKindOfClass:[NSMenuToolbarItem class]])
            ((NSMenuToolbarItem*)item).menu = [c menuFor:spec];
    }];
}

WXD_EXPORTED void
wxd_MacToolbar_RunCustomizationPalette(void* tb)
{
    WxdToolbarController* c = Controller(tb);
    if (c && c.toolbar && c.toolbar.allowsUserCustomization)
        [c.toolbar runCustomizationPalette:nil];
}

WXD_EXPORTED void
wxd_MacToolbar_SetVisible(void* tb, bool visible)
{
    WxdToolbarController* c = Controller(tb);
    if (c && c.toolbar)
        c.toolbar.visible = visible;
}

WXD_EXPORTED bool
wxd_MacToolbar_IsVisible(void* tb)
{
    WxdToolbarController* c = Controller(tb);
    return c && c.toolbar && c.toolbar.visible;
}

} // extern "C"
