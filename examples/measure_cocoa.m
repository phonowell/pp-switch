// Bare Cocoa reference for issue #1. No FLTK/Rust runtime or application state.
// Dedicated CI/desktop only; same 440x502 content size and three-second idle.
#import <Cocoa/Cocoa.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

@interface PPBaselineView : NSView
@end
@implementation PPBaselineView
- (BOOL)acceptsFirstResponder { return YES; }
- (void)keyDown:(NSEvent *)event {
    if (![event.characters isEqualToString:@" "]) { [super keyDown:event]; return; }
    [self displayIfNeeded];
    puts("READY"); fflush(stdout);
    dispatch_after(dispatch_time(DISPATCH_TIME_NOW, 3 * NSEC_PER_SEC), dispatch_get_main_queue(), ^{
        [NSApp stop:nil];
        [NSApp postEvent:[NSEvent otherEventWithType:NSEventTypeApplicationDefined
            location:NSZeroPoint modifierFlags:0 timestamp:0 windowNumber:0 context:nil
            subtype:0 data1:0 data2:0] atStart:NO];
    });
}
@end

int main(int argc, const char **argv) {
    BOOL allowed = getenv("CI") && strcmp(getenv("CI"), "true") == 0;
    for (int index = 1; index < argc; index++) if (!strcmp(argv[index], "--allow-desktop-input")) allowed = YES;
    if (!allowed) { fputs("SKIPPED: Cocoa measurement requires a dedicated desktop\n", stderr); return 2; }
    @autoreleasepool {
        [NSApplication sharedApplication];
        [NSApp setActivationPolicy:NSApplicationActivationPolicyRegular];
        NSWindow *window = [[NSWindow alloc] initWithContentRect:NSMakeRect(0, 0, 440, 502)
            styleMask:NSWindowStyleMaskTitled | NSWindowStyleMaskClosable | NSWindowStyleMaskMiniaturizable
            backing:NSBackingStoreBuffered defer:NO];
        PPBaselineView *view = [[PPBaselineView alloc] initWithFrame:NSMakeRect(0, 0, 440, 502)];
        window.contentView = view;
        window.title = @"PPS bare Cocoa reference";
        [window center]; [window makeKeyAndOrderFront:nil];
        [window makeFirstResponder:view];
        [NSApp activateIgnoringOtherApps:YES];
        dispatch_async(dispatch_get_main_queue(), ^{
            [NSApp postEvent:[NSEvent keyEventWithType:NSEventTypeKeyDown location:NSZeroPoint
                modifierFlags:0 timestamp:[NSProcessInfo processInfo].systemUptime
                windowNumber:window.windowNumber context:nil characters:@" "
                charactersIgnoringModifiers:@" " isARepeat:NO keyCode:49] atStart:NO];
        });
        [NSApp run];
    }
    return 0;
}
