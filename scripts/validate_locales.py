#!/usr/bin/env python3
"""
Validation script for locale files.
Ensures all locale files have the same message keys and proper syntax.
"""

import os
import re
import sys
from pathlib import Path
from typing import Dict, Set, List, Tuple

def extract_message_keys(file_path: Path) -> Set[str]:
    """Extract all message keys from a Fluent (.ftl) file."""
    keys = set()
    
    with open(file_path, 'r', encoding='utf-8') as f:
        for line_num, line in enumerate(f, 1):
            line = line.strip()
            
            # Skip comments and empty lines
            if not line or line.startswith('#'):
                continue
            
            # Match message definitions (key = value)
            match = re.match(r'^([a-zA-Z][a-zA-Z0-9_-]*)\s*=', line)
            if match:
                key = match.group(1)
                keys.add(key)
    
    return keys

def validate_fluent_syntax(file_path: Path) -> List[str]:
    """Basic validation of Fluent syntax."""
    errors = []

    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()
        lines = content.split('\n')

        # Track multiline constructs like pluralization
        in_multiline = False
        multiline_start = 0
        brace_stack = []

        for line_num, line in enumerate(lines, 1):
            line = line.strip()

            # Skip comments and empty lines
            if not line or line.startswith('#'):
                continue

            # Check for basic syntax issues
            if '=' in line and not in_multiline:
                if not re.match(r'^[a-zA-Z][a-zA-Z0-9_-]*\s*=', line):
                    errors.append(f"Line {line_num}: Invalid message key format")

            # Handle pluralization and multiline constructs
            if '{$' in line and '->' in line:
                in_multiline = True
                multiline_start = line_num
                brace_stack.append('{')
            elif in_multiline:
                if line.startswith('[') and ']' in line:
                    # This is a pluralization option line
                    continue
                elif line == '}':
                    if brace_stack:
                        brace_stack.pop()
                        if not brace_stack:
                            in_multiline = False
                    else:
                        errors.append(f"Line {line_num}: Unexpected closing brace")
            else:
                # For single-line messages, check brace matching
                open_braces = line.count('{')
                close_braces = line.count('}')
                if open_braces != close_braces:
                    # Allow for simple parameter substitution like {$variable}
                    if not re.search(r'\{\$[a-zA-Z][a-zA-Z0-9_-]*\}', line):
                        errors.append(f"Line {line_num}: Unmatched braces")

        # Check if we ended with unclosed multiline constructs
        if in_multiline and brace_stack:
            errors.append(f"Line {multiline_start}: Unclosed multiline construct")

    return errors

def main():
    """Main validation function."""
    script_dir = Path(__file__).parent
    locales_dir = script_dir.parent / 'locales'
    
    if not locales_dir.exists():
        print(f"Error: Locales directory not found: {locales_dir}")
        sys.exit(1)
    
    # Find all locale files
    locale_files = {}
    for locale_dir in locales_dir.iterdir():
        if locale_dir.is_dir():
            main_ftl = locale_dir / 'main.ftl'
            if main_ftl.exists():
                locale_files[locale_dir.name] = main_ftl
    
    if not locale_files:
        print("Error: No locale files found")
        sys.exit(1)
    
    print(f"Found {len(locale_files)} locale files:")
    for locale, path in locale_files.items():
        print(f"  {locale}: {path}")
    
    print("\n" + "="*50)
    
    # Extract keys from all files
    all_keys = {}
    syntax_errors = {}
    
    for locale, file_path in locale_files.items():
        print(f"\nValidating {locale}...")
        
        # Extract keys
        keys = extract_message_keys(file_path)
        all_keys[locale] = keys
        print(f"  Found {len(keys)} message keys")
        
        # Check syntax
        errors = validate_fluent_syntax(file_path)
        if errors:
            syntax_errors[locale] = errors
            print(f"  ❌ {len(errors)} syntax errors found")
            for error in errors:
                print(f"    {error}")
        else:
            print(f"  ✅ Syntax validation passed")
    
    # Compare keys across locales
    print(f"\n" + "="*50)
    print("Comparing message keys across locales...")
    
    if len(all_keys) < 2:
        print("Need at least 2 locales to compare")
        return
    
    # Use the first locale as reference
    reference_locale = list(all_keys.keys())[0]
    reference_keys = all_keys[reference_locale]
    
    print(f"Using '{reference_locale}' as reference ({len(reference_keys)} keys)")
    
    all_consistent = True
    
    for locale, keys in all_keys.items():
        if locale == reference_locale:
            continue
        
        missing_keys = reference_keys - keys
        extra_keys = keys - reference_keys
        
        if missing_keys or extra_keys:
            all_consistent = False
            print(f"\n❌ {locale} has inconsistencies:")
            
            if missing_keys:
                print(f"  Missing keys ({len(missing_keys)}):")
                for key in sorted(missing_keys):
                    print(f"    - {key}")
            
            if extra_keys:
                print(f"  Extra keys ({len(extra_keys)}):")
                for key in sorted(extra_keys):
                    print(f"    + {key}")
        else:
            print(f"✅ {locale} is consistent with reference")
    
    # Summary
    print(f"\n" + "="*50)
    print("VALIDATION SUMMARY")
    print(f"="*50)
    
    if syntax_errors:
        print(f"❌ Syntax errors found in {len(syntax_errors)} locales")
        for locale, errors in syntax_errors.items():
            print(f"  {locale}: {len(errors)} errors")
    else:
        print("✅ No syntax errors found")
    
    if all_consistent:
        print("✅ All locales have consistent message keys")
    else:
        print("❌ Some locales have inconsistent message keys")
    
    # Exit with error code if there are issues
    if syntax_errors or not all_consistent:
        sys.exit(1)
    else:
        print("\n🎉 All validations passed!")

if __name__ == '__main__':
    main()
