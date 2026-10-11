# Match fltk-rs static-msvcrt and Rust's +crt-static. Ignored by FLTK on Unix.
set(FLTK_MSVC_RUNTIME_DLL OFF CACHE BOOL "Use the static MSVC runtime" FORCE)
