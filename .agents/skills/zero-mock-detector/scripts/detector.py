"""
Zero-Mock Static Analysis Engine leveraging AST Code-Graph.
Combines Code-Graph index queries with deep file-level AST heuristics.
"""
import os
import re
import sqlite3
import subprocess
from pathlib import Path
from typing import Dict, List, Optional, Set

from rules import (
    EXCLUDED_SYMBOLS,
    Finding,
    MOCK_IMPORT_PATTERNS,
    RE_FAKE_LATENCY,
    RE_MOCK_BYPASS_FLAG,
    RE_MOCK_SYMBOL,
    RE_STUBBED_CODE,
    Severity,
)

class ZeroMockDetector:
    def __init__(self, root_dir: str, use_codegraph: bool = True):
        self.root_dir = Path(root_dir).resolve()
        self.use_codegraph = use_codegraph
        self.index_db_path = self.root_dir / ".code-graph" / "index.db"

    def is_production_file(self, rel_path: str) -> bool:
        normalized = rel_path.replace("\\", "/")
        lower = normalized.lower()

        # Discard documentation, build outputs, and caches
        if any(lower.startswith(p) or f"/{p}" in lower for p in [
            "target/", "node_modules/", "dist/", "build/", ".git/", ".agents/", "docs/"
        ]):
            return False

        # Exclude recognized test directories and test suffixes
        test_patterns = [
            "/tests/", "/test/", "/spec/", "/__tests__/", "/testing/",
            "/benches/", "/evals/", "/examples/"
        ]
        if any(tp in f"/{lower}" for tp in test_patterns):
            return False

        test_file_suffixes = [
            "_test.rs", ".test.ts", ".test.js", ".spec.ts", ".spec.js",
            "_test.go", "_test.py"
        ]
        if any(lower.endswith(s) for s in test_file_suffixes) or Path(lower).name.startswith("test_"):
            return False

        # Must be valid source code
        valid_exts = {".rs", ".ts", ".js", ".tsx", ".jsx", ".py", ".go", ".java", ".kt"}
        return Path(lower).suffix in valid_exts

    def ensure_codegraph_index(self) -> bool:
        if self.index_db_path.exists():
            return True
        if not self.use_codegraph:
            return False

        # Attempt to run code-graph incremental-index
        try:
            res = subprocess.run(
                ["code-graph", "incremental-index"],
                cwd=str(self.root_dir),
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                timeout=30,
                check=False
            )
            return self.index_db_path.exists()
        except Exception:
            return False

    def scan_codegraph_nodes(self) -> List[Finding]:
        findings: List[Finding] = []
        if not self.index_db_path.exists():
            return findings

        try:
            conn = sqlite3.connect(str(self.index_db_path))
            cur = conn.cursor()
            query = """
                SELECT n.id, f.path, n.type, n.name, n.start_line, n.code_content, n.is_test
                FROM nodes n
                JOIN files f ON n.file_id = f.id
                WHERE n.is_test = 0
            """
            cur.execute(query)
            rows = cur.fetchall()
            conn.close()

            for _, rel_path, ntype, name, start_line, code_content, is_test in rows:
                if not self.is_production_file(rel_path):
                    continue

                # Check 1: Mock Symbol Name in Production
                clean_name = name.lower()
                if clean_name not in EXCLUDED_SYMBOLS and RE_MOCK_SYMBOL.search(name):
                    findings.append(Finding(
                        rule_id="RULE_MOCK_SYMBOL_NAME",
                        severity=Severity.HIGH,
                        file_path=rel_path,
                        line_number=start_line,
                        symbol_name=name,
                        message=f"Production {ntype} '{name}' contains forbidden mock/fake/stub naming",
                        code_snippet=code_content.splitlines()[0] if code_content else None
                    ))

                # Check 2: Stubbed Code in Production
                if code_content and RE_STUBBED_CODE.search(code_content):
                    findings.append(Finding(
                        rule_id="RULE_STUBBED_CODE",
                        severity=Severity.HIGH,
                        file_path=rel_path,
                        line_number=start_line,
                        symbol_name=name,
                        message=f"Production symbol '{name}' contains stubbed todo!() / unimplemented!() logic",
                        code_snippet=code_content.splitlines()[0] if code_content else None
                    ))

                # Check 3: Fake Latency in Repository/Service symbols
                is_service = any(k in rel_path.lower() for k in ["repo", "service", "client", "gateway", "driver"])
                if is_service and code_content and RE_FAKE_LATENCY.search(code_content):
                    findings.append(Finding(
                        rule_id="RULE_FAKE_LATENCY",
                        severity=Severity.HIGH,
                        file_path=rel_path,
                        line_number=start_line,
                        symbol_name=name,
                        message=f"Simulated latency/sleep found in production service/driver '{name}'",
                        code_snippet=code_content.splitlines()[0] if code_content else None
                    ))

        except Exception as e:
            # Fallback continues gracefully
            pass

        return findings

    def scan_files_directly(self) -> List[Finding]:
        findings: List[Finding] = []
        for root, _, files in os.walk(self.root_dir):
            for file in files:
                full_path = Path(root) / file
                rel_path = str(full_path.relative_to(self.root_dir))
                if not self.is_production_file(rel_path):
                    continue

                try:
                    with open(full_path, "r", encoding="utf-8", errors="ignore") as f:
                        lines = f.readlines()
                except Exception:
                    continue

                for idx, line in enumerate(lines, 1):
                    # Check 1: Mock Framework Imports
                    for rule_id, pattern, desc in MOCK_IMPORT_PATTERNS:
                        if pattern.search(line):
                            findings.append(Finding(
                                rule_id=rule_id,
                                severity=Severity.CRITICAL,
                                file_path=rel_path,
                                line_number=idx,
                                symbol_name=None,
                                message=desc,
                                code_snippet=line.strip()
                            ))

                    # Check 2: Mock Bypass Environment Flags
                    if RE_MOCK_BYPASS_FLAG.search(line):
                        findings.append(Finding(
                            rule_id="RULE_MOCK_BYPASS_FLAG",
                            severity=Severity.MEDIUM,
                            file_path=rel_path,
                            line_number=idx,
                            symbol_name=None,
                            message="Production code contains mock bypass flag (e.g. NODE_ENV === 'test' or MOCK_ENABLED)",
                            code_snippet=line.strip()
                        ))

                    # Check 3: Mock Symbol Definition in Production File
                    mock_def_match = re.search(r"""^\s*(?:pub\s+)?(?:class|struct|fn|def|interface|type)\s+([A-Za-z0-9_]*mock[A-Za-z0-9_]*)""", line, re.IGNORECASE)
                    if mock_def_match:
                        sym = mock_def_match.group(1)
                        if sym.lower() not in EXCLUDED_SYMBOLS:
                            findings.append(Finding(
                                rule_id="RULE_MOCK_SYMBOL_NAME",
                                severity=Severity.HIGH,
                                file_path=rel_path,
                                line_number=idx,
                                symbol_name=sym,
                                message=f"Production symbol '{sym}' contains forbidden mock/fake naming",
                                code_snippet=line.strip()
                            ))

                    # Check 4: Stubbed Code in Production
                    if RE_STUBBED_CODE.search(line):
                        findings.append(Finding(
                            rule_id="RULE_STUBBED_CODE",
                            severity=Severity.HIGH,
                            file_path=rel_path,
                            line_number=idx,
                            symbol_name=None,
                            message="Production code contains stubbed todo!() / unimplemented!() logic",
                            code_snippet=line.strip()
                        ))

                    # Check 5: Fake Latency in Repository/Service Files
                    is_service = any(k in rel_path.lower() for k in ["repo", "service", "client", "gateway", "driver"])
                    if is_service and RE_FAKE_LATENCY.search(line):
                        findings.append(Finding(
                            rule_id="RULE_FAKE_LATENCY",
                            severity=Severity.HIGH,
                            file_path=rel_path,
                            line_number=idx,
                            symbol_name=None,
                            message="Simulated latency/sleep found in production service/driver",
                            code_snippet=line.strip()
                        ))

        return findings


    def scan(self) -> List[Finding]:
        self.ensure_codegraph_index()
        all_findings = self.scan_codegraph_nodes() + self.scan_files_directly()

        # Deduplicate findings by (rule_id, file_path, line_number)
        seen: Set[str] = set()
        deduped: List[Finding] = []
        for f in all_findings:
            key = f"{f.rule_id}:{f.file_path}:{f.line_number}"
            if key not in seen:
                seen.add(key)
                deduped.append(f)

        # Sort by severity (CRITICAL first), then file, line
        severity_order = {
            Severity.CRITICAL: 0,
            Severity.HIGH: 1,
            Severity.MEDIUM: 2,
            Severity.LOW: 3,
        }
        deduped.sort(key=lambda x: (severity_order.get(x.severity, 99), x.file_path, x.line_number))
        return deduped
