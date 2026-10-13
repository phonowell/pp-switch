// Cocoa integration; no resident helper or extra runtime.
#import <Cocoa/Cocoa.h>
#import <CoreServices/CoreServices.h>
#import <QuartzCore/QuartzCore.h>

int pp_dark_appearance(void) {
    NSAppearance *appearance = [[NSApplication sharedApplication] effectiveAppearance];
    NSString *match = [appearance bestMatchFromAppearancesWithNames:
        @[NSAppearanceNameAqua, NSAppearanceNameDarkAqua]];
    return [match isEqualToString:NSAppearanceNameDarkAqua];
}

static void (*appearance_changed)(int);
@interface PPAppearanceObserver : NSObject
@end
@implementation PPAppearanceObserver
- (void)observeValueForKeyPath:(NSString *)path ofObject:(id)object
    change:(NSDictionary *)change context:(void *)context {
    if (appearance_changed) appearance_changed(pp_dark_appearance());
}
@end

void pp_watch_appearance(void (*callback)(int)) {
    appearance_changed = callback;
    static PPAppearanceObserver *observer;
    if (!observer) {
        observer = [PPAppearanceObserver new];
        [[NSApplication sharedApplication] addObserver:observer
            forKeyPath:@"effectiveAppearance" options:NSKeyValueObservingOptionInitial context:NULL];
    }
}

void pp_round_menu(void *handle) {
    NSWindow *window = (__bridge NSWindow *)handle;
    if (!window || ([window styleMask] & NSWindowStyleMaskTitled)) return;
    NSView *view = [window contentView];
    if ([view layer].cornerRadius == 7 && [view layer].masksToBounds) return;
    [window setOpaque:NO];
    [window setBackgroundColor:[NSColor clearColor]];
    [view setWantsLayer:YES];
    [view layer].cornerRadius = 7;
    [view layer].masksToBounds = YES;
    [window setHasShadow:YES];
    [window invalidateShadow];
}

static void (*receive_url)(const char *);
@interface PPLinkReceiver : NSObject
- (void)openURL:(NSAppleEventDescriptor *)event reply:(NSAppleEventDescriptor *)reply;
@end
@implementation PPLinkReceiver
- (void)openURL:(NSAppleEventDescriptor *)event reply:(NSAppleEventDescriptor *)reply {
    NSString *url = [[event paramDescriptorForKeyword:keyDirectObject] stringValue];
    if (url && [url lengthOfBytesUsingEncoding:NSUTF8StringEncoding] <= 256 * 1024 && receive_url)
        receive_url([url UTF8String]);
}
@end

void pp_listen_urls(void (*callback)(const char *)) {
    receive_url = callback;
    static PPLinkReceiver *receiver;
    if (!receiver) receiver = [PPLinkReceiver new];
    [[NSAppleEventManager sharedAppleEventManager] setEventHandler:receiver
        andSelector:@selector(openURL:reply:) forEventClass:kInternetEventClass
        andEventID:kAEGetURL];
}

// Test the installed handler, not just a stale bundle identifier left in LS.
static BOOL has_other_handler(NSString *scheme, NSURL *bundleURL) {
    NSURL *url = [NSURL URLWithString:[scheme stringByAppendingString:@"://v1/import"]];
    NSURL *handler = [[NSWorkspace sharedWorkspace] URLForApplicationToOpenURL:url];
    // Opening the bundle can make LS discover us before explicit registration.
    // That must not prevent claiming an otherwise unowned scheme.
    return handler && ![[[handler URLByResolvingSymlinksInPath] path]
        isEqualToString:[[bundleURL URLByResolvingSymlinksInPath] path]];
}

int pp_register_urls(void) {
    NSBundle *bundle = [NSBundle mainBundle];
    if (![[bundle bundlePath] hasSuffix:@".app"] || ![bundle bundleIdentifier]) return 0;
    // Capture existing associations before LS learns about this bundle.
    BOOL ccs = has_other_handler(@"ccswitch", [bundle bundleURL]);
    BOOL pps = has_other_handler(@"ppswitch", [bundle bundleURL]);
    if (LSRegisterURL((__bridge CFURLRef)[bundle bundleURL], true) != noErr) return 0;
    CFStringRef ident = (__bridge CFStringRef)[bundle bundleIdentifier];
    if (!pps && LSSetDefaultHandlerForURLScheme(CFSTR("ppswitch"), ident) != noErr) return 0;
    if (!ccs && LSSetDefaultHandlerForURLScheme(CFSTR("ccswitch"), ident) != noErr) return 0;
    return 1;
}
