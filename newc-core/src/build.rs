//! Shared build-target logic for both build systems.
//!
//! Make and CMake projects expose the same logical targets (`all`, `run`,
//! `debug`, `release`, `strict`, `valgrind`, `analyse`, `test`, `clean`,
//! `help`). For Make these map 1:1 onto Makefile targets. For CMake there is
//! no single-command equivalent, so callers (the GUI build runner and the
//! CLI) use these helpers to drive `cmake -S . -B build` (configure) and
//! `cmake --build build` (build) themselves.

/// Help text for the `help` target, shared between Make and CMake projects
/// since both generate the same logical target set.
pub const HELP_LINES: &[&str] = &[
    "Targets:",
    "  all       Build the project (default)",
    "  run       Build and run the executable",
    "  debug     Build with debug symbols and sanitizers (ASan, UBSan)",
    "  release   Build with optimisations and hardening flags",
    "  strict    Clean rebuild with -Werror and release optimisations",
    "  valgrind  Build with -g and run under Valgrind memory checker",
    "  analyse   Run clang static analysis (syntax check + extra warnings)",
    "  cppcheck  Run cppcheck static analysis (requires cppcheck installed)",
    "  coverage  Build with gcov instrumentation, run, and report line coverage",
    "  test      Build and run the test executable (if test_utils was scaffolded)",
    "  clean     Remove build artefacts and executable",
];

/// CMake configure (`-D...`) flags for a given logical target.
pub fn cmake_configure_args(target: &str) -> Vec<&'static str> {
    match target {
        "debug" => vec!["-DCMAKE_BUILD_TYPE=Debug", "-DSTRICT=OFF", "-DCOVERAGE=OFF"],
        "strict" => vec![
            "-DCMAKE_BUILD_TYPE=Release",
            "-DSTRICT=ON",
            "-DCOVERAGE=OFF",
        ],
        "coverage" => vec!["-DCMAKE_BUILD_TYPE=Debug", "-DSTRICT=OFF", "-DCOVERAGE=ON"],
        _ => vec![
            "-DCMAKE_BUILD_TYPE=Release",
            "-DSTRICT=OFF",
            "-DCOVERAGE=OFF",
        ],
    }
}

/// Map a logical target to the `CMakePresets.json` configure preset name
/// written by [`crate::templates::CMAKE_PRESETS`].
pub fn cmake_preset_name(target: &str) -> &'static str {
    match target {
        "debug" => "debug",
        "strict" => "strict",
        "coverage" => "coverage",
        _ => "release",
    }
}

/// Returns `true` if switching to this target requires reconfiguring
/// (changes `CMAKE_BUILD_TYPE`/`STRICT`), beyond the one-time configure
/// needed when `build/CMakeCache.txt` doesn't exist yet.
pub fn cmake_needs_reconfigure(target: &str) -> bool {
    matches!(target, "debug" | "release" | "strict" | "coverage")
}

/// Map a Makefile-style target name to the CMake `--build --target` to pass
/// (CMake has no implicit default for these synthetic Make targets).
pub fn cmake_build_target(target: &str) -> Option<&str> {
    match target {
        "all" | "debug" | "release" | "strict" => None, // default target
        _ => Some(target),
    }
}
