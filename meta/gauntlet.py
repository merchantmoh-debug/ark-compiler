import os
import subprocess
import sys
import glob
import time

# ANSI Colors
RED = "\033[91m"
GREEN = "\033[92m"
YELLOW = "\033[93m"
RESET = "\033[0m"

def run_test(file_path):
    """
    Runs a single Ark test file.
    Returns: (success, output)
    """
    try:
        # We run the compiler via the meta/ark.py bootstrapper
        cmd = [sys.executable, "meta/ark.py", "run", file_path]
        
        # Capture output, inject safe input to prevent hangs on interactive scripts
        result = subprocess.run(
            cmd, 
            capture_output=True, 
            text=True, 
            input="!exit\n", # Send exit command just in case
            timeout=10 # 10 second timeout per test
        )
        
        return result
    except subprocess.TimeoutExpired:
        return None

def main():
    print(f"{YELLOW}=========================================={RESET}")
    print(f"{YELLOW}   THE GAUNTLET: ARK REGRESSION SUITE     {RESET}")
    print(f"{YELLOW}=========================================={RESET}")

    # 1. Discovery
    test_files = glob.glob("tests/**/*.ark", recursive=True) + \
                 glob.glob("apps/**/*.ark", recursive=True) + \
                 glob.glob("benchmarks/**/*.ark", recursive=True) + \
                 glob.glob("examples/**/*.ark", recursive=True)
    
    # Filter out specific files if needed
    test_files = [f for f in test_files if "manual" not in f]
    
    total = len(test_files)
    print(f"Loaded {total} candidates.\n")

    passed = 0
    failed = 0
    skipped = 0

    for f in sorted(test_files):
        is_expected_fail = "fail_" in f or "jailbreak" in f
        test_name = os.path.basename(f)
        
        print(f"Running {test_name.ljust(30)} ... ", end="", flush=True)
        
        start = time.time()
        result = run_test(f)
        duration = time.time() - start

        if result is None:
            print(f"{RED}[TIMEOUT]{RESET} ({duration:.2f}s)")
            failed += 1
            continue

        # Determine Status
        code = result.returncode
        
        # Logic:
        # - test_*: Expect 0
        # - fail_*: Expect != 0
        
        success = False
        if is_expected_fail:
            if code != 0:
                success = True
            else:
                success = False # It should have failed but didn't!
        else:
            if code == 0:
                success = True
            else:
                success = False

        if success:
            print(f"{GREEN}[PASS]{RESET} ({duration:.2f}s)")
            passed += 1
        else:
            print(f"{RED}[FAIL]{RESET} ({duration:.2f}s)")
            if is_expected_fail:
                print(f"  {RED}Expected Failure (Non-Zero Exit), got 0{RESET}")
            else:
                print(f"  {RED}Exit Code: {code}{RESET}")
                # Print last few lines of stderr for context
                lines = result.stderr.splitlines()
                for line in lines[-3:]:
                    print(f"  {RED}>> {line}{RESET}")
            failed += 1

    print(f"\n{YELLOW}=========================================={RESET}")
    print(f"RESULTS: {passed} Passed, {failed} Failed, {total} Total")
    
    if failed == 0:
        print(f"{GREEN}ALL SYSTEMS GREEN. RELEASE CANDIDATE VERIFIED.{RESET}")
        sys.exit(0)
    else:
        print(f"{RED}CRITICAL SYSTEM FAILURE. DO NOT RELEASE.{RESET}")
        sys.exit(1)

if __name__ == "__main__":
    main()
