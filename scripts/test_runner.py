"""Cargo runner: isolate inherited client overrides before test threads start."""
import os
from pathlib import Path
import re
import subprocess
import sys


CLIENT_OVERRIDES = (
    "ANTHROPIC_BASE_URL", "ANTHROPIC_AUTH_TOKEN", "ANTHROPIC_API_KEY",
    "CLAUDE_CODE_USE_BEDROCK", "CLAUDE_CODE_USE_VERTEX", "CLAUDE_CODE_USE_FOUNDRY",
    "ANTHROPIC_MODEL", "ANTHROPIC_DEFAULT_HAIKU_MODEL",
    "ANTHROPIC_DEFAULT_SONNET_MODEL", "ANTHROPIC_DEFAULT_OPUS_MODEL",
    "ANTHROPIC_SMALL_FAST_MODEL", "OPENAI_API_KEY", "OPENAI_BASE_URL",
    "GEMINI_API_KEY", "GOOGLE_GEMINI_BASE_URL", "GEMINI_BASE_URL",
    "GOOGLE_API_KEY", "GOOGLE_GENAI_USE_VERTEXAI", "GEMINI_MODEL",
    "DEEPSEEK_BASE_URL", "DEEPSEEK_API_KEY",
)


def environment(binary, inherited):
    env = inherited.copy()
    # cargo test executes hashed harnesses in deps; cargo run executes the app
    # or examples elsewhere and must keep the user's real environment.
    binary = Path(binary)
    if binary.parent.name == "deps" and re.search(r"-[0-9a-f]{16}$", binary.stem):
        for name in CLIENT_OVERRIDES:
            env.pop(name, None)
    return env


def main():
    return subprocess.call(sys.argv[1:], env=environment(sys.argv[1], os.environ))


if __name__ == "__main__":
    raise SystemExit(main())
