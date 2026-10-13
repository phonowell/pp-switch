"""Quit bridge and real Cocoa lifecycle regressions without windows or activation.

The self-targeted quit AppleEvent uses the same event class/id as Dock Quit.
Native Dock pointer acceptance remains a separate dedicated-desktop check.
"""
from pathlib import Path
import os
import platform
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
SOURCE = r'''
#import <Cocoa/Cocoa.h>
#import <CoreServices/CoreServices.h>
#include <assert.h>
#include <unistd.h>
void pp_listen_quit(void (*callback)(void));
@interface FLAppDelegate : NSObject <NSApplicationDelegate>
- (NSApplicationTerminateReply)applicationShouldTerminate:(nullable NSApplication *)sender;
- (int)untouched;
- (void)request:(NSTimer *)timer;
@end
@implementation FLAppDelegate
- (NSApplicationTerminateReply)applicationShouldTerminate:(NSApplication *)sender {
    (void)sender; return NSTerminateCancel;
}
- (void)applicationWillTerminate:(NSNotification *)notification {
    (void)notification;
    assert(![NSApp isActive] && [[NSApp windows] count] == 0);
    puts("Cocoa termination accepted; no windows or activation"); fflush(stdout);
}
- (int)untouched { return 42; }
- (void)request:(NSTimer *)timer {
    if ([[timer userInfo] isEqualToString:@"appleevent"]) {
        NSAppleEventDescriptor *target = [NSAppleEventDescriptor descriptorWithDescriptorType:typeKernelProcessID
            bytes:&(pid_t){getpid()} length:sizeof(pid_t)];
        NSAppleEventDescriptor *event = [NSAppleEventDescriptor
            appleEventWithEventClass:kCoreEventClass eventID:kAEQuitApplication
            targetDescriptor:target returnID:kAutoGenerateReturnID transactionID:kAnyTransactionID];
        NSError *error = nil;
        [event sendEventWithOptions:kAENoReply timeout:1 error:&error];
        assert(error == nil);
    } else {
        [NSApp terminate:nil];
    }
}
@end
static int calls;
static void requested(void) { calls++; puts("PPS quit callback"); fflush(stdout); }
int main(int argc, const char **argv) {
    @autoreleasepool {
        FLAppDelegate *delegate = [FLAppDelegate new];
        if (argc == 1) {
            assert(NSApp == nil);
            pp_listen_quit(requested);
            assert([delegate applicationShouldTerminate:nil] == NSTerminateNow);
            assert(calls == 1 && [delegate untouched] == 42);
            pp_listen_quit(requested);
            assert([delegate applicationShouldTerminate:nil] == NSTerminateNow);
            assert(calls == 2 && NSApp == nil);
            puts("delegate preserved; no NSApplication");
            return 0;
        }
        [NSApplication sharedApplication];
        [NSApp setActivationPolicy:NSApplicationActivationPolicyProhibited];
        [NSApp setDelegate:delegate];
        [NSApp finishLaunching];
        assert(![NSApp isActive] && [[NSApp windows] count] == 0);
        pp_listen_quit(requested);
        [NSTimer scheduledTimerWithTimeInterval:0.05 target:delegate
            selector:@selector(request:) userInfo:[NSString stringWithUTF8String:argv[1]] repeats:NO];
        // Model FLTK\'s outstanding native wait, not NSApplication.run.
        // NSTerminateCancel leaves this wait blocked even after the callback.
        [NSApp nextEventMatchingMask:NSEventMaskAny untilDate:[NSDate distantFuture]
            inMode:NSDefaultRunLoopMode dequeue:YES];
        fputs("native wait returned without Cocoa termination\n", stderr);
        return 2;
    }
}
'''


@unittest.skipUnless(platform.system() == "Darwin", "macOS Objective-C bridge")
class NativeQuitBridge(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.directory = tempfile.TemporaryDirectory()
        source = Path(cls.directory.name) / "quit.m"
        cls.binary = Path(cls.directory.name) / "quit"
        source.write_text(SOURCE)
        subprocess.run(["clang", "-Oz", "-fobjc-arc", "-Wall", "-Wextra", "-Werror", str(source), str(ROOT / "src/macos.m"), "-framework", "Cocoa", "-framework", "CoreServices", "-framework", "QuartzCore", "-o", str(cls.binary)], check=True, capture_output=True, text=True, timeout=60)

    @classmethod
    def tearDownClass(cls):
        cls.directory.cleanup()

    def test_delegate_registration_preserves_other_methods(self):
        result = subprocess.run([str(self.binary)], check=True, capture_output=True, text=True, timeout=10)
        self.assertIn("no NSApplication", result.stdout)

    def check_lifecycle(self, route):
        result = subprocess.run([str(self.binary), route], check=True, capture_output=True, text=True, timeout=10)
        self.assertEqual(result.stdout.count("PPS quit callback"), 1)
        self.assertIn("Cocoa termination accepted; no windows or activation", result.stdout)

    def test_application_menu_terminates_inside_native_wait(self):
        self.check_lifecycle("menu")

    def test_dock_quit_appleevent_terminates_inside_native_wait(self):
        self.check_lifecycle("appleevent")


@unittest.skipUnless(platform.system() == "Darwin", "macOS real FLTK lifecycle")
class RealFltkQuit(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cargo = str(Path.home() / ".cargo/bin/cargo")
        subprocess.run([cargo, "build", "--locked", "--example", "mac_quit_background"],
                       cwd=ROOT, check=True, capture_output=True, text=True, timeout=120)
        target = Path(os.environ.get("CARGO_TARGET_DIR", ROOT / "target"))
        cls.binary = target / "debug/examples/mac_quit_background"

    def test_real_widgets_quit_from_each_page_without_activation(self):
        for route in ["menu", "dock"]:
            for page in ["main", "settings", "editor"]:
                with self.subTest(route=route, page=page):
                    result = subprocess.run([str(self.binary), f"{route}-{page}"], cwd=ROOT,
                                            capture_output=True, text=True, timeout=10)
                    self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
                    self.assertIn("REAL DELEGATE FLAppDelegate", result.stdout)
                    self.assertIn("QUIT FIXTURES UNCHANGED", result.stdout)
                    self.assertIn("REAL FLTK COCOA TERMINATED WITHOUT PRESENTATION", result.stdout)
                    self.assertIn(f"{2 if page == 'editor' else 1} HIDDEN NATIVE WINDOWS", result.stdout)

    def test_missing_handler_negative_control_reproduces_dock_failure(self):
        result = subprocess.run([str(self.binary), "dock-unhandled-main"], cwd=ROOT,
                                capture_output=True, text=True, timeout=10)
        self.assertEqual(result.returncode, 2)
        self.assertIn("errAEEventNotHandled", result.stderr)
        self.assertNotIn("REAL FLTK COCOA TERMINATED", result.stdout)

    def test_model_prefill_editor_and_import_callbacks_preserve_client_files(self):
        result = subprocess.run([str(self.binary), "model-prefill"], cwd=ROOT,
                                check=True, capture_output=True, text=True, timeout=10)
        self.assertIn("MODEL PREFILL EDIT SWITCH IMPORT SAVE CANCEL PASSED", result.stdout)
        self.assertIn("CLIENT FIXTURE UNCHANGED", result.stdout)
        self.assertIn("REAL FLTK COCOA TERMINATED WITHOUT PRESENTATION", result.stdout)
