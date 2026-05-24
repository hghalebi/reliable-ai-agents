import os
from pathlib import Path

ROOT = Path("/Users/hamzeghalebi/selfhost/windmill")
BOOK_SRC = ROOT / "books" / "postgres-rig-agent-jobs" / "src"
FURTHER_READING_NAME = "31-credible-resources-further-reading.md"

POINTER_TEXT = """## Further Reading and Sources

- [Appendix A: Credible Resources and Further Reading](./31-credible-resources-further-reading.md) contains the complete list of academic papers and industry standards used to build the reliability model in this chapter.
"""

for path in BOOK_SRC.glob("*.md"):
    if path.name == FURTHER_READING_NAME:
        continue
    if path.name == "SUMMARY.md":
        continue
    
    content = path.read_text()
    # Find the start of the reading section. Support both old and new headers.
    match = None
    for header in ["## Further Reading & Credible References", "## Further Reading and Sources"]:
        if header in content:
            parts = content.split(header)
            # Take everything before the header and append the pointer.
            # We assume the reading section is at the end of the file.
            new_content = parts[0] + POINTER_TEXT
            path.write_text(new_content)
            print(f"Refactored {path.name}")
            break

print("Chapter refactoring complete.")
