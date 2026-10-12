// Cocoa integration; no resident helper or extra runtime.
#import <Cocoa/Cocoa.h>
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
