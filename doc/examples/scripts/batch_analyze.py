#!/usr/bin/env python3
# batch_analyze.py

import subprocess
import json
import glob

def analyze_file(filepath):
    """Run SwissArmyHammer analysis on a file."""
    result = subprocess.run([
        'swissarmyhammer', 'test', 'review/code',
        '--file_path', filepath,
        '--context', 'batch analysis'
    ], capture_output=True, text=True)
    
    return {
        'file': filepath,
        'output': result.stdout,
        'errors': result.stderr
    }

# Analyze all Python files
files = glob.glob('**/*.py', recursive=True)
results = [analyze_file(f) for f in files]

# Save results
with open('analysis_results.json', 'w') as f:
    json.dump(results, f, indent=2)

print(f"Analyzed {len(files)} files. Results saved to analysis_results.json")