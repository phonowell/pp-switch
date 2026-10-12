// Only Cocoa URL events and Launch Services; no resident helper or extra runtime.
#import <Cocoa/Cocoa.h>
#import <CoreServices/CoreServices.h>

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
static BOOL has_handler(NSString *scheme) {
    NSURL *url = [NSURL URLWithString:[scheme stringByAppendingString:@"://v1/import"]];
    return [[NSWorkspace sharedWorkspace] URLForApplicationToOpenURL:url] != nil;
}

int pp_register_urls(void) {
    NSBundle *bundle = [NSBundle mainBundle];
    if (![[bundle bundlePath] hasSuffix:@".app"] || ![bundle bundleIdentifier]) return 0;
    // Capture existing associations before LS learns about this bundle.
    BOOL ccs = has_handler(@"ccswitch"), pps = has_handler(@"ppswitch");
    if (LSRegisterURL((__bridge CFURLRef)[bundle bundleURL], true) != noErr) return 0;
    CFStringRef ident = (__bridge CFStringRef)[bundle bundleIdentifier];
    if (!pps && LSSetDefaultHandlerForURLScheme(CFSTR("ppswitch"), ident) != noErr) return 0;
    if (!ccs && LSSetDefaultHandlerForURLScheme(CFSTR("ccswitch"), ident) != noErr) return 0;
    return 1;
}
