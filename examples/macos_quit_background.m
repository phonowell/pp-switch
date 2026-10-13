// Test-only presentation suppression; production never links this object.
// Real FLTK windows/delegate/wait remain in use, without ordering windows or
// activating this shared workstation. This is lifecycle, not GUI acceptance.
#import <Cocoa/Cocoa.h>
#import <CoreServices/CoreServices.h>
#import <objc/runtime.h>
#include <assert.h>
#include <unistd.h>

static void suppress_sender(id self, SEL selector, id sender) {
    (void)self; (void)selector; (void)sender;
}
static void suppress_activation(id self, SEL selector, BOOL flag) {
    (void)self; (void)selector; (void)flag;
}
static void suppress_key(id self, SEL selector) { (void)self; (void)selector; }
static void suppress_order(id self, SEL selector, NSWindowOrderingMode mode, NSInteger relative) {
    (void)self; (void)selector; (void)mode; (void)relative;
}
static void replace(Class cls, SEL selector, IMP implementation) {
    Method method = class_getInstanceMethod(cls, selector);
    assert(method);
    method_setImplementation(method, implementation);
}
static pid_t foreground_pid;
static void check_background(void) {
    assert(![NSApp isActive]);
    assert([[[NSWorkspace sharedWorkspace] frontmostApplication] processIdentifier] == foreground_pid);
    for (NSWindow *window in [NSApp windows]) {
        assert(![window isVisible] && ![window isKeyWindow]);
    }
}
void pp_quit_background_prepare(void) {
    foreground_pid = [[[NSWorkspace sharedWorkspace] frontmostApplication] processIdentifier];
    [NSApplication sharedApplication];
    [NSApp setActivationPolicy:NSApplicationActivationPolicyProhibited];
    replace([NSApplication class], @selector(activateIgnoringOtherApps:), (IMP)suppress_activation);
    replace([NSApplication class], @selector(activate), (IMP)suppress_key);
    replace([NSWindow class], @selector(makeKeyAndOrderFront:), (IMP)suppress_sender);
    replace([NSWindow class], @selector(orderFront:), (IMP)suppress_sender);
    replace([NSWindow class], @selector(orderWindow:relativeTo:), (IMP)suppress_order);
    replace([NSWindow class], @selector(makeKeyWindow), (IMP)suppress_key);
    [[NSNotificationCenter defaultCenter] addObserverForName:NSApplicationWillTerminateNotification
        object:NSApp queue:nil usingBlock:^(NSNotification *notification) {
            (void)notification;
            check_background();
            puts("REAL FLTK COCOA TERMINATED WITHOUT PRESENTATION"); fflush(stdout);
        }];
}
void pp_quit_background_request(int appleEvent) {
    check_background();
    printf("REAL DELEGATE %s; %lu HIDDEN NATIVE WINDOWS\n",
        class_getName([[NSApp delegate] class]), (unsigned long)[[NSApp windows] count]);
    fflush(stdout);
    if (!appleEvent) { [NSApp terminate:nil]; return; }
    if (appleEvent == 2) {
        // Negative control: reproduce the absent default handler in the actual
        // pre-created NSApp + FLTK startup path, without a mocked delegate.
        [[NSAppleEventManager sharedAppleEventManager] removeEventHandlerForEventClass:kCoreEventClass
            andEventID:kAEQuitApplication];
    }
    pid_t pid = getpid();
    NSAppleEventDescriptor *target = [NSAppleEventDescriptor descriptorWithDescriptorType:typeKernelProcessID
        bytes:&pid length:sizeof(pid)];
    NSAppleEventDescriptor *event = [NSAppleEventDescriptor appleEventWithEventClass:kCoreEventClass
        eventID:kAEQuitApplication targetDescriptor:target returnID:kAutoGenerateReturnID
        transactionID:kAnyTransactionID];
    NSError *error = nil;
    [event sendEventWithOptions:kAENoReply timeout:1 error:&error];
    if (error) {
        fprintf(stderr, "quit AppleEvent delivery: %s\n", [[error description] UTF8String]);
        exit(2);
    }
}
