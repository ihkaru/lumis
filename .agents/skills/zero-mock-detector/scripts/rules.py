"""
Zero-Mock Detection Rules & Finding Models (October 2026 Standards)
Categorizes mock violations by architectural severity.
"""
from dataclasses import asdict, dataclass
from enum import Enum
from typing import Dict, List, Optional, Pattern, Tuple
import re

class Severity(str, Enum):
    CRITICAL = "CRITICAL"
    HIGH = "HIGH"
    MEDIUM = "MEDIUM"
    LOW = "LOW"

@dataclass
class Finding:
    rule_id: str
    severity: Severity
    file_path: str
    line_number: int
    symbol_name: Optional[str]
    message: str
    code_snippet: Optional[str] = None

    def to_dict(self) -> Dict:
        data = asdict(self)
        data["severity"] = self.severity.value
        return data

# Rule 1: Mocking framework imports in production source
MOCK_IMPORT_PATTERNS: List[Tuple[str, Pattern, str]] = [

    (
        "MOCK_IMPORT_PY",
        re.compile(r"^\s*(from|import)\s+(unittest\.mock|pytest_mock|faker|moto|responses|freezegun)\b"),
        "Python test mocking/faking framework imported in production source"
    ),
    (
        "MOCK_IMPORT_JS",
        re.compile(r"""(?:import|require)\s*\(?.*['"](@faker-js/faker|sinon|jest|vitest|nock|msw)['"]"""),
        "JavaScript/TypeScript test mocking framework imported in production source"
    ),
    (
        "MOCK_IMPORT_RS",
        re.compile(r"""^\s*(use\s+(mockall|wiremock|fake)::|#\[automock\])"""),
        "Rust test mocking framework referenced in production source"
    ),
    (
        "MOCK_IMPORT_GO",
        re.compile(r"""["']github\.com/(golang/mock|stretchr/testify/mock|DATA-DOG/go-sqlmock)["']"""),
        "Go test mock package imported in production source"
    ),
    (
        "MOCK_IMPORT_JVM",
        re.compile(r"""^\s*import\s+(org\.mockito|org\.powermock|com\.github\.tomakehurst\.wiremock)\b"""),
        "JVM Mockito/WireMock framework imported in production source"
    ),
]

# Rule 2: Mock symbol naming in production code
# Matches identifiers starting/ending with Mock, Stub, Fake, Dummy
RE_MOCK_SYMBOL = re.compile(
    r"""(?i)\b(mock|stub|fake|dummy|spy)[a-z0-9_]*\b|\b[a-z0-9_]*(mock|stub|fake|dummy|spy)\b"""
)

# Legitimate exclusions to avoid false positives
EXCLUDED_SYMBOLS = {
    "mocktail", "stubborn", "packet_stuffer", "stuffer", "hash_table",
    "stub", "stubs", "dummy"  # allow if specifically part of standard low-level OS/ABI placeholders
}

# Rule 3: Fake latency / Simulated network delay
RE_FAKE_LATENCY = re.compile(
    r"""(?:\b(tokio::time::sleep|std::thread::sleep|time\.sleep|setTimeout|setInterval)\s*\()"""
)

# Rule 4: Stubbed / Abandoned implementation
RE_STUBBED_CODE = re.compile(
    r"""(?:\btodo!\s*\(|\bunimplemented!\s*\(|throw\s+new\s+Error\s*\(\s*['"].*?(?:todo|stub|mock|not\s+implemented).*?['"]|raise\s+NotImplementedError\s*\(\s*['"].*?(?:todo|stub|mock).*?['"])""",
    re.IGNORECASE
)

# Rule 5: Poisoned environment mock bypass toggles
RE_MOCK_BYPASS_FLAG = re.compile(
    r"""(?:\bprocess\.env\.NODE_ENV\s*===?\s*['"]test['"]|\b(?:MOCK_ENABLED|USE_MOCK|IS_MOCK|MOCK_DATA|MOCK_MODE)\b)"""
)
