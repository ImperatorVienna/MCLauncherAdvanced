#!/usr/bin/env python3
"""Keep application source English; Chinese belongs in locale catalogs.
Unicode test data should use explicit escapes. Historical docs and imported
assets are not source code and are intentionally outside this check.
"""
from pathlib import Path
import re
import sys

root = Path(__file__).resolve().parents[1]
roots = ['apps/desktop/src', 'apps/desktop/src-tauri/src', 'crates/launcher-core/src', 'crates/launcher-install/src', 'crates/launcher-platform/src', 'scripts']
failures = []
for directory in roots:
    for path in (root / directory).rglob('*'):
        if path.suffix not in {'.ts', '.tsx', '.rs', '.py', '.css'}:
            continue
        for number, line in enumerate(path.read_text().splitlines(), 1):
            if re.search(r'[\u3400-\u9fff]', line):
                failures.append(f'{path.relative_to(root)}:{number}: localized text outside locale resources')
if failures:
    print('\n'.join(failures), file=sys.stderr)
    sys.exit(1)
print('English source check passed')
