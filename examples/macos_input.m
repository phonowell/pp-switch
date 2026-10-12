// Linked only by the native input example; never drives another process.
#import <Cocoa/Cocoa.h>
#import <ApplicationServices/ApplicationServices.h>
#import <QuartzCore/QuartzCore.h>

static NSPoint savedMouse;
static BOOL saved;
static NSPoint pressedAt;
static BOOL pressed;

static NSWindow *target(void *handle) {
    if (handle) return (__bridge NSWindow *)handle;
    return [NSApp keyWindow] ?: [NSApp mainWindow];
}

int pp_smoke_available(void) { return [[NSScreen screens] count] > 0; }

int pp_smoke_title_icon(void *handle) {
    return [[target(handle) standardWindowButton:NSWindowDocumentIconButton] image] != nil;
}

int pp_smoke_has_shadow(void *handle) { return [target(handle) hasShadow]; }

int pp_smoke_rounded_menu(void *handle) {
    NSWindow *window = target(handle);
    CALayer *layer = [[window contentView] layer];
    return ![window isOpaque] && layer.cornerRadius == 7 && layer.masksToBounds;
}

int pp_smoke_capture_chrome(void *handle, const char *path) {
    NSWindow *window = target(handle);
    NSView *frame = [[window contentView] superview];
    [frame displayIfNeeded];
    if (!([window styleMask] & NSWindowStyleMaskTitled)) {
        NSView *view = [window contentView];
        NSBitmapImageRep *pixels = [[NSBitmapImageRep alloc]
            initWithBitmapDataPlanes:NULL pixelsWide:(NSInteger)view.bounds.size.width
            pixelsHigh:(NSInteger)view.bounds.size.height bitsPerSample:8 samplesPerPixel:4
            hasAlpha:YES isPlanar:NO colorSpaceName:NSDeviceRGBColorSpace bytesPerRow:0 bitsPerPixel:0];
        if (!pixels) return 0;
        NSGraphicsContext *context = [NSGraphicsContext graphicsContextWithBitmapImageRep:pixels];
        [[view layer] renderInContext:[context CGContext]];
        if ([[pixels colorAtX:0 y:0] alphaComponent] > 0.05 ||
            [[pixels colorAtX:pixels.pixelsWide/2 y:pixels.pixelsHigh/2] alphaComponent] < 0.99)
            return 0; // prove native clipping changes the rendered corner alpha
        NSData *png = [pixels representationUsingType:NSBitmapImageFileTypePNG properties:@{}];
        return [png writeToFile:[NSString stringWithUTF8String:path] atomically:YES];
    }
    NSBitmapImageRep *pixels = [frame bitmapImageRepForCachingDisplayInRect:[frame bounds]];
    if (!pixels) return 0;
    [frame cacheDisplayInRect:[frame bounds] toBitmapImageRep:pixels];
    NSData *png = [pixels representationUsingType:NSBitmapImageFileTypePNG properties:@{}];
    return [png writeToFile:[NSString stringWithUTF8String:path] atomically:YES];
}

void pp_smoke_mouse(void *handle, int x, int y, int down) {
    NSWindow *window = target(handle);
    NSPoint local = NSMakePoint(x, [[window contentView] bounds].size.height - y);
    NSPoint screen = [window convertPointToScreen:local];
    if (!saved) { savedMouse = [NSEvent mouseLocation]; saved = YES; }
    // FLTK reads mouseLocationOutsideOfEventStream for native hit testing.
    NSScreen *mainScreen = [[NSScreen screens] firstObject];
    if (!mainScreen) return;
    CGWarpMouseCursorPosition(CGPointMake(screen.x, NSMaxY([mainScreen frame]) - screen.y));
    [window makeKeyAndOrderFront:nil];
    // A physical pointer leaving a pressed button delivers a drag before up.
    // FLTK updates the pressed value on drag, not on the release coordinate.
    if (!down && pressed && !NSEqualPoints(pressedAt, local)) {
        NSEvent *drag = [NSEvent mouseEventWithType:NSEventTypeLeftMouseDragged
            location:local modifierFlags:0 timestamp:[[NSProcessInfo processInfo] systemUptime]
            windowNumber:[window windowNumber] context:nil eventNumber:0 clickCount:1 pressure:1];
        [NSApp postEvent:drag atStart:NO];
    }
    pressed = down;
    if (down) pressedAt = local;
    NSEvent *event = [NSEvent mouseEventWithType:down ? NSEventTypeLeftMouseDown : NSEventTypeLeftMouseUp
        location:local modifierFlags:0 timestamp:[[NSProcessInfo processInfo] systemUptime]
        windowNumber:[window windowNumber] context:nil eventNumber:0 clickCount:1 pressure:down ? 1 : 0];
    [NSApp postEvent:event atStart:NO];
}

void pp_smoke_key(void *handle, unsigned short key, unsigned int modifiers, const char *text) {
    NSWindow *window = target(handle);
    NSString *characters = [NSString stringWithUTF8String:text];
    for (int down = 1; down >= 0; down--) {
        NSEvent *event = [NSEvent keyEventWithType:down ? NSEventTypeKeyDown : NSEventTypeKeyUp
            location:NSZeroPoint modifierFlags:modifiers timestamp:[[NSProcessInfo processInfo] systemUptime]
            windowNumber:[window windowNumber] context:nil characters:characters
            charactersIgnoringModifiers:characters isARepeat:NO keyCode:key];
        [NSApp postEvent:event atStart:NO];
    }
}

void pp_smoke_close(void *handle) { [target(handle) performClose:nil]; }

// Only this fixture's application appearance changes; no global defaults.
void pp_smoke_appearance(int dark) {
    [NSApplication sharedApplication].appearance = dark < 0 ? nil : [NSAppearance appearanceNamed:
        dark ? NSAppearanceNameDarkAqua : NSAppearanceNameAqua];
}

int pp_smoke_windows(void) {
    int count = 0;
    for (NSWindow *window in [NSApp windows]) if ([window isVisible]) count++;
    return count;
}

void pp_smoke_restore_mouse(void) {
    NSScreen *mainScreen = [[NSScreen screens] firstObject];
    if (saved && mainScreen) CGWarpMouseCursorPosition(CGPointMake(savedMouse.x,
        NSMaxY([mainScreen frame]) - savedMouse.y));
}
