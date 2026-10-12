include("${CMAKE_CURRENT_LIST_DIR}/static-crt.cmake")

# No app path uses FLTK's file chooser, printing or image decoders.
set(FLTK_OPTION_PRINT_SUPPORT OFF CACHE BOOL "No printing" FORCE)
set(FLTK_OPTION_FILESYSTEM_SUPPORT OFF CACHE BOOL "No FLTK file chooser" FORCE)
set(FLTK_OPTION_SVG OFF CACHE BOOL "No SVG decoder" FORCE)

# Split C++ sections as well as Rust sections so the native linker can discard
# unused FLTK methods. Preserve debug flags and the static Windows CRT.
if(CMAKE_HOST_WIN32)
  set(CMAKE_C_FLAGS_MINSIZEREL "/O1 /DNDEBUG /Gy /Gw" CACHE STRING "Native size flags" FORCE)
  set(CMAKE_CXX_FLAGS_MINSIZEREL "/O1 /DNDEBUG /Gy /Gw" CACHE STRING "Native size flags" FORCE)
else()
  if(CMAKE_HOST_APPLE)
    # Apple ld can optimize Clang bitcode from the static FLTK archives together.
    # Rust LTO alone cannot inline/eliminate code across this native boundary.
    set(PP_NATIVE_SIZE_FLAG "-Oz -flto=thin")
  else()
    set(PP_NATIVE_SIZE_FLAG "-Os")
  endif()
  set(CMAKE_C_FLAGS_MINSIZEREL "${PP_NATIVE_SIZE_FLAG} -DNDEBUG -ffunction-sections -fdata-sections" CACHE STRING "Native size flags" FORCE)
  set(CMAKE_CXX_FLAGS_MINSIZEREL "${PP_NATIVE_SIZE_FLAG} -DNDEBUG -ffunction-sections -fdata-sections" CACHE STRING "Native size flags" FORCE)
endif()
