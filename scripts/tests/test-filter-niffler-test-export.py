#!/usr/bin/env python3

import json
import subprocess
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
FILTER = ROOT / "scripts/filter-niffler-test-export.py"


def run(records):
    with tempfile.TemporaryDirectory() as tmp:
        source = Path(tmp) / "source.jsonl"
        target = Path(tmp) / "target.jsonl"
        source.write_text("".join(json.dumps(row) + "\n" for row in records))
        result = subprocess.run(
            [str(FILTER), str(source), str(target)], capture_output=True, text=True
        )
        output = []
        if target.exists():
            output = [json.loads(line) for line in target.read_text().splitlines()]
        return result, output


manifest = {
    "record_type": "manifest",
    "manifest": {"format_version": 1, "created_at_unix_secs": 1, "source_driver": "sqlite", "domains": ["users", "api_keys"]},
}
result, output = run(
    [
        manifest,
        {"record_type": "row", "domain": "users", "id": "u1", "payload": {"id": "u1"}},
        {"record_type": "row", "domain": "api_keys", "id": "k1", "payload": {"id": "k1", "key_encrypted": "secret"}},
        {"record_type": "row", "domain": "system_configs", "id": "c1", "payload": {"key": "site_name", "value": "Test"}},
        {"record_type": "row", "domain": "system_configs", "id": "c2", "payload": {"key": "admin_cleanup_run_history", "value": "history"}},
    ]
)
assert result.returncode == 0, result.stderr
assert [row.get("domain") for row in output if row["record_type"] == "row"] == ["users", "system_configs"]
assert "api_keys" not in output[0]["manifest"]["domains"]
assert output[0]["manifest"]["domains"] == [
    "users",
    "providers",
    "endpoints",
    "global_models",
    "models",
    "user_groups",
    "user_group_members",
    "system_configs",
]

result, _ = run(
    [
        manifest,
        {"record_type": "row", "domain": "endpoints", "id": "e1", "payload": {"base_url": "https://user:pass@example.test/v1"}},
    ]
)
assert result.returncode != 0
assert "contains credentials" in result.stderr

print("niffler-test export filter tests passed")
