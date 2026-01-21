#!/usr/bin/env python3
"""
Master Test Runner for Slay the Spire RL Project.

This script runs all tests across the project:
1. Python tests with pytest (core, integration, unit)
2. React/TypeScript tests with bun test
3. Java mod compilation verification
4. Coverage report generation

Usage:
    python scripts/run_tests.py              # Run all tests
    python scripts/run_tests.py --python     # Python tests only
    python scripts/run_tests.py --frontend   # Frontend tests only
    python scripts/run_tests.py --java       # Java verification only
    python scripts/run_tests.py --fast       # Skip slow tests
    python scripts/run_tests.py --coverage   # Generate coverage report
    python scripts/run_tests.py -v           # Verbose output

Exit codes:
    0 - All tests passed
    1 - Some tests failed
    2 - Configuration error
"""

import argparse
import os
import subprocess
import sys
import time
from dataclasses import dataclass, field
from pathlib import Path
from typing import List, Optional


# =============================================================================
# Configuration
# =============================================================================

PROJECT_ROOT = Path(__file__).parent.parent.absolute()
TESTS_DIR = PROJECT_ROOT / "tests"
CORE_TESTS_DIR = PROJECT_ROOT / "core" / "tests"
FRONTEND_DIR = PROJECT_ROOT / "ui" / "frontend"
MOD_DIR = PROJECT_ROOT / "mod"
JDK_DIR = PROJECT_ROOT / "jdk8"


@dataclass
class TestResult:
    """Result of a test suite run."""
    name: str
    passed: bool = True
    duration: float = 0.0
    tests_run: int = 0
    tests_passed: int = 0
    tests_failed: int = 0
    tests_skipped: int = 0
    coverage: Optional[float] = None
    output: str = ""
    error: str = ""


@dataclass
class TestSummary:
    """Summary of all test results."""
    results: List[TestResult] = field(default_factory=list)
    total_duration: float = 0.0

    @property
    def all_passed(self) -> bool:
        return all(r.passed for r in self.results)

    @property
    def total_tests(self) -> int:
        return sum(r.tests_run for r in self.results)

    @property
    def total_passed(self) -> int:
        return sum(r.tests_passed for r in self.results)

    @property
    def total_failed(self) -> int:
        return sum(r.tests_failed for r in self.results)


# =============================================================================
# Test Runners
# =============================================================================


def run_python_tests(
    verbose: bool = False,
    fast: bool = False,
    coverage: bool = False,
    markers: Optional[List[str]] = None,
) -> TestResult:
    """Run Python tests with pytest."""
    result = TestResult(name="Python Tests")
    start_time = time.time()

    # Use uv to run pytest (handles dependencies properly)
    cmd = ["uv", "run", "pytest"]

    # Test directories
    test_paths = []
    if TESTS_DIR.exists():
        test_paths.append(str(TESTS_DIR))
    if CORE_TESTS_DIR.exists():
        test_paths.append(str(CORE_TESTS_DIR))

    if not test_paths:
        result.error = "No test directories found"
        result.passed = False
        return result

    cmd.extend(test_paths)

    # Options
    if verbose:
        cmd.append("-v")
    else:
        cmd.append("-q")

    if fast:
        cmd.extend(["-m", "not slow"])

    if markers:
        cmd.extend(["-m", " and ".join(markers)])

    if coverage:
        cmd.extend([
            "--cov=core",
            "--cov-report=term-missing",
            "--cov-report=html:coverage_html",
        ])

    # Add junit output for parsing
    cmd.extend(["--tb=short", "--no-header"])

    print(f"\n{'='*60}")
    print("Running Python Tests")
    print(f"{'='*60}")
    print(f"Command: {' '.join(cmd)}")

    try:
        proc = subprocess.run(
            cmd,
            cwd=PROJECT_ROOT,
            capture_output=True,
            text=True,
            timeout=600,
        )

        result.output = proc.stdout
        result.error = proc.stderr
        result.passed = proc.returncode == 0

        # Parse test counts from output
        for line in proc.stdout.split("\n"):
            if "passed" in line or "failed" in line:
                # Try to parse pytest summary line
                parts = line.split()
                for i, part in enumerate(parts):
                    if part == "passed" and i > 0:
                        try:
                            result.tests_passed = int(parts[i-1])
                        except ValueError:
                            pass
                    elif part == "failed" and i > 0:
                        try:
                            result.tests_failed = int(parts[i-1])
                        except ValueError:
                            pass
                    elif part == "skipped" and i > 0:
                        try:
                            result.tests_skipped = int(parts[i-1])
                        except ValueError:
                            pass

        result.tests_run = result.tests_passed + result.tests_failed

        # Parse coverage if present
        if coverage:
            for line in proc.stdout.split("\n"):
                if "TOTAL" in line and "%" in line:
                    try:
                        parts = line.split()
                        for part in parts:
                            if "%" in part:
                                result.coverage = float(part.rstrip("%"))
                                break
                    except (ValueError, IndexError):
                        pass

        if verbose or not result.passed:
            print(result.output)
            if result.error:
                print(result.error, file=sys.stderr)

    except subprocess.TimeoutExpired:
        result.passed = False
        result.error = "Test execution timed out (600s)"
    except Exception as e:
        result.passed = False
        result.error = f"Failed to run tests: {e}"

    result.duration = time.time() - start_time
    return result


def run_frontend_tests(verbose: bool = False) -> TestResult:
    """Run frontend tests with bun test."""
    result = TestResult(name="Frontend Tests")
    start_time = time.time()

    if not FRONTEND_DIR.exists():
        result.error = f"Frontend directory not found: {FRONTEND_DIR}"
        result.passed = True  # Skip, don't fail
        result.output = "Skipped: No frontend directory"
        return result

    package_json = FRONTEND_DIR / "package.json"
    if not package_json.exists():
        result.error = "No package.json found in frontend"
        result.passed = True  # Skip
        result.output = "Skipped: No package.json"
        return result

    # Check if bun is available
    try:
        subprocess.run(["bun", "--version"], capture_output=True, check=True)
    except (subprocess.CalledProcessError, FileNotFoundError):
        result.error = "bun not found, skipping frontend tests"
        result.passed = True
        result.output = "Skipped: bun not available"
        return result

    # Check if test script exists in package.json
    import json
    with open(package_json) as f:
        pkg = json.load(f)
        if "test" not in pkg.get("scripts", {}):
            result.output = "Skipped: No test script in package.json"
            result.passed = True
            return result

    print(f"\n{'='*60}")
    print("Running Frontend Tests")
    print(f"{'='*60}")

    cmd = ["bun", "run", "test:run"]

    try:
        proc = subprocess.run(
            cmd,
            cwd=FRONTEND_DIR,
            capture_output=True,
            text=True,
            timeout=300,
        )

        result.output = proc.stdout
        result.error = proc.stderr
        result.passed = proc.returncode == 0

        if verbose or not result.passed:
            print(result.output)
            if result.error:
                print(result.error, file=sys.stderr)

    except subprocess.TimeoutExpired:
        result.passed = False
        result.error = "Frontend test execution timed out"
    except Exception as e:
        result.passed = False
        result.error = f"Failed to run frontend tests: {e}"

    result.duration = time.time() - start_time
    return result


def run_java_verification(verbose: bool = False) -> TestResult:
    """Verify Java mod compiles successfully."""
    result = TestResult(name="Java Mod Verification")
    start_time = time.time()

    if not MOD_DIR.exists():
        result.error = f"Mod directory not found: {MOD_DIR}"
        result.passed = False
        return result

    pom_file = MOD_DIR / "pom.xml"
    if not pom_file.exists():
        result.error = "No pom.xml found in mod directory"
        result.passed = False
        return result

    print(f"\n{'='*60}")
    print("Running Java Mod Verification")
    print(f"{'='*60}")

    # Check required Java files exist
    expected_files = [
        "src/main/java/evtracker/EVTrackerMod.java",
        "src/main/java/evtracker/DamageCalculator.java",
        "src/main/java/evtracker/EVLogger.java",
        "src/main/java/evtracker/TurnStateCapture.java",
        "src/main/java/evtracker/InfiniteDetector.java",
        "src/main/java/evtracker/DebugOverlay.java",
        "src/main/java/evtracker/ConsoleCommands.java",
    ]

    missing_files = []
    for f in expected_files:
        if not (MOD_DIR / f).exists():
            missing_files.append(f)

    if missing_files:
        result.error = f"Missing Java files: {', '.join(missing_files)}"
        result.passed = False
        print(f"Missing files: {missing_files}")
        return result

    result.tests_run = len(expected_files)
    result.tests_passed = len(expected_files) - len(missing_files)

    # Try to compile with Maven
    # First check if mvn is available
    try:
        subprocess.run(["mvn", "--version"], capture_output=True, check=True)
    except (subprocess.CalledProcessError, FileNotFoundError):
        result.output = "Maven not available, skipping compilation (file check passed)"
        result.passed = True
        print(result.output)
        result.duration = time.time() - start_time
        return result

    # Set JAVA_HOME if jdk8 exists
    env = os.environ.copy()
    if JDK_DIR.exists():
        env["JAVA_HOME"] = str(JDK_DIR)
        env["PATH"] = f"{JDK_DIR}/bin:{env.get('PATH', '')}"

    cmd = ["mvn", "compile", "-q"]

    try:
        proc = subprocess.run(
            cmd,
            cwd=MOD_DIR,
            capture_output=True,
            text=True,
            timeout=120,
            env=env,
        )

        result.output = proc.stdout
        result.error = proc.stderr
        result.passed = proc.returncode == 0

        if result.passed:
            print("Java compilation successful")
            result.tests_run += 1
            result.tests_passed += 1
        else:
            print("Java compilation failed")
            if verbose or not result.passed:
                print(result.output)
                print(result.error, file=sys.stderr)

    except subprocess.TimeoutExpired:
        result.passed = False
        result.error = "Maven compilation timed out"
    except Exception as e:
        result.passed = False
        result.error = f"Failed to run Maven: {e}"

    result.duration = time.time() - start_time
    return result


def run_python_integration_tests(verbose: bool = False) -> TestResult:
    """Run only integration tests."""
    return run_python_tests(
        verbose=verbose,
        markers=["integration"],
    )


def run_python_unit_tests(verbose: bool = False) -> TestResult:
    """Run only unit tests."""
    return run_python_tests(
        verbose=verbose,
        markers=["unit"],
    )


# =============================================================================
# Output Formatting
# =============================================================================


def print_summary(summary: TestSummary) -> None:
    """Print a summary of all test results."""
    print(f"\n{'='*60}")
    print("TEST SUMMARY")
    print(f"{'='*60}")

    for result in summary.results:
        status = "PASS" if result.passed else "FAIL"
        status_color = "\033[92m" if result.passed else "\033[91m"
        reset_color = "\033[0m"

        print(f"\n{result.name}:")
        print(f"  Status: {status_color}{status}{reset_color}")
        print(f"  Duration: {result.duration:.2f}s")

        if result.tests_run > 0:
            print(f"  Tests: {result.tests_passed}/{result.tests_run} passed", end="")
            if result.tests_failed > 0:
                print(f", {result.tests_failed} failed", end="")
            if result.tests_skipped > 0:
                print(f", {result.tests_skipped} skipped", end="")
            print()

        if result.coverage is not None:
            print(f"  Coverage: {result.coverage:.1f}%")

        if result.error and not result.passed:
            print(f"  Error: {result.error[:100]}")

    print(f"\n{'='*60}")
    print(f"TOTAL: {summary.total_passed}/{summary.total_tests} tests passed in {summary.total_duration:.2f}s")

    if summary.all_passed:
        print("\033[92mAll tests passed!\033[0m")
    else:
        print("\033[91mSome tests failed!\033[0m")

    print(f"{'='*60}\n")


def print_coverage_report(result: TestResult) -> None:
    """Print coverage report location."""
    if result.coverage is not None:
        print(f"\nCoverage Report: {result.coverage:.1f}%")
        coverage_html = PROJECT_ROOT / "coverage_html" / "index.html"
        if coverage_html.exists():
            print(f"HTML Report: file://{coverage_html}")


# =============================================================================
# Main
# =============================================================================


def main() -> int:
    """Main entry point."""
    parser = argparse.ArgumentParser(
        description="Run all tests for Slay the Spire RL project",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=__doc__,
    )

    parser.add_argument(
        "-v", "--verbose",
        action="store_true",
        help="Verbose output",
    )
    parser.add_argument(
        "--fast",
        action="store_true",
        help="Skip slow tests",
    )
    parser.add_argument(
        "--coverage",
        action="store_true",
        help="Generate coverage report",
    )
    parser.add_argument(
        "--python",
        action="store_true",
        help="Run only Python tests",
    )
    parser.add_argument(
        "--frontend",
        action="store_true",
        help="Run only frontend tests",
    )
    parser.add_argument(
        "--java",
        action="store_true",
        help="Run only Java verification",
    )
    parser.add_argument(
        "--integration",
        action="store_true",
        help="Run only integration tests",
    )
    parser.add_argument(
        "--unit",
        action="store_true",
        help="Run only unit tests",
    )

    args = parser.parse_args()

    summary = TestSummary()
    start_time = time.time()

    # Determine which tests to run
    run_all = not (args.python or args.frontend or args.java or args.integration or args.unit)

    try:
        if run_all or args.python:
            result = run_python_tests(
                verbose=args.verbose,
                fast=args.fast,
                coverage=args.coverage,
            )
            summary.results.append(result)

            if args.coverage:
                print_coverage_report(result)

        if run_all or args.frontend:
            result = run_frontend_tests(verbose=args.verbose)
            summary.results.append(result)

        if run_all or args.java:
            result = run_java_verification(verbose=args.verbose)
            summary.results.append(result)

        if args.integration:
            result = run_python_integration_tests(verbose=args.verbose)
            summary.results.append(result)

        if args.unit:
            result = run_python_unit_tests(verbose=args.verbose)
            summary.results.append(result)

    except KeyboardInterrupt:
        print("\nTest run interrupted!")
        return 1

    summary.total_duration = time.time() - start_time
    print_summary(summary)

    return 0 if summary.all_passed else 1


if __name__ == "__main__":
    sys.exit(main())
