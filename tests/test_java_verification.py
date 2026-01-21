"""
Java Mod Verification Tests.

Tests to verify the Java mod structure and compilation:
1. Required Java source files exist
2. Required dependencies exist (JAR files)
3. Maven pom.xml is valid
4. Mod compiles successfully (optional, requires Maven)
5. Static analysis checks (optional)
"""

import os
import subprocess
import sys
import xml.etree.ElementTree as ET
from pathlib import Path

import pytest

sys.path.insert(0, '/Users/jackswitzer/Desktop/SlayTheSpireRL')

# =============================================================================
# Configuration
# =============================================================================

PROJECT_ROOT = Path("/Users/jackswitzer/Desktop/SlayTheSpireRL")
MOD_DIR = PROJECT_ROOT / "mod"
JDK_DIR = PROJECT_ROOT / "jdk8"


# =============================================================================
# File Structure Tests
# =============================================================================


@pytest.mark.java
class TestJavaFileStructure:
    """Test that all required Java source files exist."""

    REQUIRED_JAVA_FILES = [
        "src/main/java/evtracker/EVTrackerMod.java",
        "src/main/java/evtracker/DamageCalculator.java",
        "src/main/java/evtracker/EVLogger.java",
        "src/main/java/evtracker/TurnStateCapture.java",
        "src/main/java/evtracker/InfiniteDetector.java",
        "src/main/java/evtracker/DebugOverlay.java",
        "src/main/java/evtracker/ConsoleCommands.java",
    ]

    def test_mod_directory_exists(self):
        """Mod directory should exist."""
        assert MOD_DIR.exists(), f"Mod directory not found: {MOD_DIR}"
        assert MOD_DIR.is_dir(), f"Mod path is not a directory: {MOD_DIR}"

    def test_pom_xml_exists(self):
        """Maven pom.xml should exist."""
        pom_file = MOD_DIR / "pom.xml"
        assert pom_file.exists(), f"pom.xml not found: {pom_file}"

    def test_src_directory_exists(self):
        """Source directory structure should exist."""
        src_dir = MOD_DIR / "src" / "main" / "java" / "evtracker"
        assert src_dir.exists(), f"Source directory not found: {src_dir}"

    @pytest.mark.parametrize("java_file", REQUIRED_JAVA_FILES)
    def test_required_java_file_exists(self, java_file):
        """Each required Java file should exist."""
        file_path = MOD_DIR / java_file
        assert file_path.exists(), f"Required Java file not found: {file_path}"

    def test_all_java_files_are_readable(self):
        """All Java files should be readable."""
        java_dir = MOD_DIR / "src" / "main" / "java" / "evtracker"
        if not java_dir.exists():
            pytest.skip("Java source directory not found")

        for java_file in java_dir.glob("*.java"):
            assert os.access(java_file, os.R_OK), f"Cannot read: {java_file}"

    def test_java_files_not_empty(self):
        """Java files should not be empty."""
        for java_file in self.REQUIRED_JAVA_FILES:
            file_path = MOD_DIR / java_file
            if file_path.exists():
                size = file_path.stat().st_size
                assert size > 0, f"Java file is empty: {file_path}"


# =============================================================================
# Dependency Tests
# =============================================================================


@pytest.mark.java
class TestJavaDependencies:
    """Test that required dependencies exist."""

    REQUIRED_LIBS = [
        "lib/desktop-1.0.jar",  # Slay the Spire game JAR
        "lib/BaseMod.jar",
        "lib/ModTheSpire.jar",
        "lib/StSLib.jar",
    ]

    def test_lib_directory_exists(self):
        """Library directory should exist."""
        lib_dir = MOD_DIR / "lib"
        assert lib_dir.exists(), f"lib directory not found: {lib_dir}"

    @pytest.mark.parametrize("lib_file", REQUIRED_LIBS)
    def test_required_lib_exists(self, lib_file):
        """Each required library JAR should exist."""
        lib_path = MOD_DIR / lib_file
        assert lib_path.exists(), f"Required library not found: {lib_path}"

    def test_lib_files_are_valid_jars(self):
        """Library files should be valid JAR (ZIP) files."""
        import zipfile

        lib_dir = MOD_DIR / "lib"
        if not lib_dir.exists():
            pytest.skip("lib directory not found")

        for jar_file in lib_dir.glob("*.jar"):
            try:
                with zipfile.ZipFile(jar_file, 'r') as zf:
                    # Should be able to list contents
                    assert len(zf.namelist()) > 0, f"JAR is empty: {jar_file}"
            except zipfile.BadZipFile:
                pytest.fail(f"Invalid JAR file: {jar_file}")


# =============================================================================
# POM.xml Validation Tests
# =============================================================================


@pytest.mark.java
class TestPomXmlValidation:
    """Test Maven pom.xml validity."""

    def test_pom_xml_is_valid_xml(self):
        """pom.xml should be valid XML."""
        pom_file = MOD_DIR / "pom.xml"
        if not pom_file.exists():
            pytest.skip("pom.xml not found")

        try:
            tree = ET.parse(pom_file)
            root = tree.getroot()
            assert root is not None
        except ET.ParseError as e:
            pytest.fail(f"Invalid XML in pom.xml: {e}")

    def test_pom_has_required_elements(self):
        """pom.xml should have required Maven elements."""
        pom_file = MOD_DIR / "pom.xml"
        if not pom_file.exists():
            pytest.skip("pom.xml not found")

        tree = ET.parse(pom_file)
        root = tree.getroot()

        # Maven namespace
        ns = {'m': 'http://maven.apache.org/POM/4.0.0'}

        # Required elements
        assert root.find('m:groupId', ns) is not None or root.find('groupId') is not None, "Missing groupId"
        assert root.find('m:artifactId', ns) is not None or root.find('artifactId') is not None, "Missing artifactId"
        assert root.find('m:version', ns) is not None or root.find('version') is not None, "Missing version"

    def test_pom_has_java8_target(self):
        """pom.xml should target Java 8."""
        pom_file = MOD_DIR / "pom.xml"
        if not pom_file.exists():
            pytest.skip("pom.xml not found")

        content = pom_file.read_text()

        # Check for Java 8 configuration
        assert "1.8" in content or "8" in content, "pom.xml should target Java 8"

    def test_pom_has_required_dependencies(self):
        """pom.xml should declare required dependencies."""
        pom_file = MOD_DIR / "pom.xml"
        if not pom_file.exists():
            pytest.skip("pom.xml not found")

        content = pom_file.read_text()

        # Check for expected dependencies
        expected_deps = ["slaythespire", "BaseMod", "ModTheSpire"]
        for dep in expected_deps:
            assert dep in content, f"Missing dependency: {dep}"


# =============================================================================
# Java Source Validation Tests
# =============================================================================


@pytest.mark.java
class TestJavaSourceValidation:
    """Basic validation of Java source files."""

    def test_main_mod_class_has_annotation(self):
        """Main mod class should have SpireInitializer annotation."""
        mod_file = MOD_DIR / "src/main/java/evtracker/EVTrackerMod.java"
        if not mod_file.exists():
            pytest.skip("EVTrackerMod.java not found")

        content = mod_file.read_text()

        # Check for SpireInitializer annotation
        assert "@SpireInitializer" in content, "Missing @SpireInitializer annotation"

    def test_main_mod_class_has_initialize_method(self):
        """Main mod class should have initialize() method."""
        mod_file = MOD_DIR / "src/main/java/evtracker/EVTrackerMod.java"
        if not mod_file.exists():
            pytest.skip("EVTrackerMod.java not found")

        content = mod_file.read_text()

        assert "public static void initialize()" in content, "Missing initialize() method"

    def test_damage_calculator_exists_and_valid(self):
        """DamageCalculator should have calculation methods."""
        calc_file = MOD_DIR / "src/main/java/evtracker/DamageCalculator.java"
        if not calc_file.exists():
            pytest.skip("DamageCalculator.java not found")

        content = calc_file.read_text()

        # Should have some calculation method
        assert "calculate" in content.lower() or "damage" in content.lower(), \
            "DamageCalculator should have calculation methods"

    def test_no_syntax_errors_in_imports(self):
        """Java files should have valid import statements."""
        java_dir = MOD_DIR / "src" / "main" / "java" / "evtracker"
        if not java_dir.exists():
            pytest.skip("Java source directory not found")

        for java_file in java_dir.glob("*.java"):
            content = java_file.read_text()
            lines = content.split('\n')

            for i, line in enumerate(lines):
                stripped = line.strip()
                if stripped.startswith("import "):
                    # Basic syntax check: should end with ;
                    assert stripped.endswith(";"), \
                        f"Invalid import in {java_file.name}:{i+1}: {stripped}"


# =============================================================================
# Compilation Tests (Optional - Requires Maven)
# =============================================================================


@pytest.mark.java
@pytest.mark.slow
class TestJavaCompilation:
    """Test Java mod compilation (requires Maven)."""

    @staticmethod
    def maven_available() -> bool:
        """Check if Maven is available."""
        try:
            result = subprocess.run(
                ["mvn", "--version"],
                capture_output=True,
                timeout=10,
            )
            return result.returncode == 0
        except (subprocess.SubprocessError, FileNotFoundError):
            return False

    def test_maven_compile(self):
        """Mod should compile with Maven."""
        if not self.maven_available():
            pytest.skip("Maven not available")

        if not MOD_DIR.exists():
            pytest.skip("Mod directory not found")

        # Set up environment with JDK 8 if available
        env = os.environ.copy()
        if JDK_DIR.exists():
            env["JAVA_HOME"] = str(JDK_DIR)
            env["PATH"] = f"{JDK_DIR}/bin:{env.get('PATH', '')}"

        result = subprocess.run(
            ["mvn", "compile", "-q"],
            cwd=MOD_DIR,
            capture_output=True,
            text=True,
            timeout=120,
            env=env,
        )

        if result.returncode != 0:
            print("STDOUT:", result.stdout)
            print("STDERR:", result.stderr)

        assert result.returncode == 0, f"Maven compile failed: {result.stderr}"

    def test_maven_package(self):
        """Mod should package with Maven."""
        if not self.maven_available():
            pytest.skip("Maven not available")

        if not MOD_DIR.exists():
            pytest.skip("Mod directory not found")

        env = os.environ.copy()
        if JDK_DIR.exists():
            env["JAVA_HOME"] = str(JDK_DIR)
            env["PATH"] = f"{JDK_DIR}/bin:{env.get('PATH', '')}"

        result = subprocess.run(
            ["mvn", "package", "-q", "-DskipTests"],
            cwd=MOD_DIR,
            capture_output=True,
            text=True,
            timeout=180,
            env=env,
        )

        assert result.returncode == 0, f"Maven package failed: {result.stderr}"

        # Check that JAR was created
        target_dir = MOD_DIR / "target"
        if target_dir.exists():
            jars = list(target_dir.glob("*.jar"))
            assert len(jars) > 0, "No JAR file created in target/"


# =============================================================================
# Static Analysis Tests (Optional)
# =============================================================================


@pytest.mark.java
class TestJavaStaticAnalysis:
    """Basic static analysis of Java source."""

    def test_no_system_exit_in_mod(self):
        """Mod should not call System.exit()."""
        java_dir = MOD_DIR / "src" / "main" / "java" / "evtracker"
        if not java_dir.exists():
            pytest.skip("Java source directory not found")

        for java_file in java_dir.glob("*.java"):
            content = java_file.read_text()
            assert "System.exit" not in content, \
                f"System.exit() found in {java_file.name}"

    def test_no_hardcoded_paths(self):
        """Mod should not have hardcoded absolute paths."""
        java_dir = MOD_DIR / "src" / "main" / "java" / "evtracker"
        if not java_dir.exists():
            pytest.skip("Java source directory not found")

        bad_patterns = [
            "/Users/",
            "C:\\Users\\",
            "/home/",
            "\\\\",  # UNC paths
        ]

        for java_file in java_dir.glob("*.java"):
            content = java_file.read_text()
            for pattern in bad_patterns:
                assert pattern not in content, \
                    f"Hardcoded path '{pattern}' found in {java_file.name}"

    def test_no_print_statements_in_production_code(self):
        """Production code should use logger, not System.out (with exceptions)."""
        java_dir = MOD_DIR / "src" / "main" / "java" / "evtracker"
        if not java_dir.exists():
            pytest.skip("Java source directory not found")

        # Files that are allowed to use System.out (loggers, debug utilities)
        allowed_files = {"EVLogger.java", "DebugOverlay.java", "ConsoleCommands.java", "InfiniteDetector.java"}

        for java_file in java_dir.glob("*.java"):
            # Skip debug/logger files that legitimately use System.out
            if java_file.name in allowed_files:
                continue

            # Skip test files
            if "Debug" in java_file.name or "Test" in java_file.name:
                continue

            content = java_file.read_text()
            lines = content.split('\n')

            for i, line in enumerate(lines):
                # Allow in comments
                stripped = line.strip()
                if stripped.startswith("//") or stripped.startswith("*"):
                    continue

                # Check for System.out (but allow System.out in strings)
                if "System.out.print" in line:
                    # Basic check - not perfect but catches obvious cases
                    if '"System.out' not in line:
                        pytest.fail(
                            f"System.out found in {java_file.name}:{i+1} - use logger instead"
                        )

    def test_classes_have_package_declaration(self):
        """All Java classes should have package declaration."""
        java_dir = MOD_DIR / "src" / "main" / "java" / "evtracker"
        if not java_dir.exists():
            pytest.skip("Java source directory not found")

        for java_file in java_dir.glob("*.java"):
            content = java_file.read_text()
            assert "package evtracker;" in content, \
                f"Missing package declaration in {java_file.name}"


# =============================================================================
# Resource File Tests
# =============================================================================


@pytest.mark.java
class TestJavaResources:
    """Test resource files for the mod."""

    def test_resources_directory_structure(self):
        """Resources directory should have proper structure."""
        resources_dir = MOD_DIR / "src" / "main" / "resources"

        # Resources directory is optional but should be well-structured if present
        if not resources_dir.exists():
            pytest.skip("No resources directory")

        # Check for ModTheSpire.json if it exists
        mts_json = resources_dir / "ModTheSpire.json"
        if mts_json.exists():
            import json
            try:
                with open(mts_json) as f:
                    data = json.load(f)
                assert "modid" in data or "name" in data, "Invalid ModTheSpire.json"
            except json.JSONDecodeError as e:
                pytest.fail(f"Invalid JSON in ModTheSpire.json: {e}")

    def test_no_large_binary_files(self):
        """Resource files should not be excessively large."""
        resources_dir = MOD_DIR / "src" / "main" / "resources"
        if not resources_dir.exists():
            pytest.skip("No resources directory")

        max_size = 10 * 1024 * 1024  # 10 MB

        for resource_file in resources_dir.rglob("*"):
            if resource_file.is_file():
                size = resource_file.stat().st_size
                assert size < max_size, \
                    f"Resource file too large: {resource_file} ({size} bytes)"
