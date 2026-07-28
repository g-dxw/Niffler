#!/usr/bin/env python3

import argparse
import json
import sys
from urllib.parse import urlsplit

ALLOWED_DOMAINS = [
    "users",
    "providers",
    "endpoints",
    "global_models",
    "models",
    "user_groups",
    "user_group_members",
    "system_configs",
]
ALLOWED_DOMAIN_SET = set(ALLOWED_DOMAINS)
ALLOWED_SYSTEM_CONFIG_KEYS = {
    "contact_us_content",
    "contact_us_format",
    "default_user_group_id",
    "provider_priority_mode",
    "scheduling_mode",
    "site_name",
    "site_subtitle",
}
ALLOWED_PROVIDER_CONFIG_KEYS = {"pool_advanced"}
ALLOWED_ENDPOINT_CONFIG_KEYS = {"_aether_fixed_provider_template", "upstream_stream_policy"}


def reject(message: str) -> None:
    raise ValueError(message)


def validate_url(value: object, label: str) -> None:
    if value in (None, ""):
        return
    if not isinstance(value, str):
        reject(f"{label} must be a string")
    parsed = urlsplit(value)
    if parsed.username or parsed.password or parsed.query or parsed.fragment:
        reject(f"{label} contains credentials, query parameters, or a fragment")


def validate_json_keys(value: object, allowed: set[str], label: str) -> None:
    if value in (None, ""):
        return
    if not isinstance(value, str):
        reject(f"{label} must be JSON text")
    parsed = json.loads(value)
    if not isinstance(parsed, dict):
        reject(f"{label} must contain a JSON object")
    unexpected = set(parsed) - allowed
    if unexpected:
        reject(f"{label} contains unexpected keys: {sorted(unexpected)}")


def keep_row(record: dict) -> bool:
    domain = record.get("domain")
    if domain not in ALLOWED_DOMAIN_SET:
        return False
    payload = record.get("payload")
    if not isinstance(payload, dict):
        reject(f"{domain} payload is not an object")

    if domain == "system_configs":
        return payload.get("key") in ALLOWED_SYSTEM_CONFIG_KEYS
    if domain == "providers":
        validate_url(payload.get("website"), "provider website")
        validate_url(payload.get("proxy"), "provider proxy")
        validate_json_keys(payload.get("config"), ALLOWED_PROVIDER_CONFIG_KEYS, "provider config")
    if domain == "endpoints":
        validate_url(payload.get("base_url"), "endpoint base_url")
        validate_url(payload.get("proxy"), "endpoint proxy")
        for field in ("header_rules", "body_rules", "metadata"):
            if payload.get(field) not in (None, "", "{}", "[]"):
                reject(f"endpoint {field} is not empty")
        validate_json_keys(payload.get("config"), ALLOWED_ENDPOINT_CONFIG_KEYS, "endpoint config")
    return True


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("input")
    parser.add_argument("output")
    args = parser.parse_args()

    counts: dict[str, int] = {}
    manifest_written = False
    with open(args.input, encoding="utf-8") as source, open(
        args.output, "x", encoding="utf-8"
    ) as target:
        for line_number, line in enumerate(source, 1):
            if not line.strip():
                continue
            record = json.loads(line)
            if record.get("record_type") == "manifest":
                if manifest_written:
                    reject("duplicate manifest")
                manifest = record.get("manifest")
                if not isinstance(manifest, dict):
                    reject("invalid manifest")
                # The importer processes domains in manifest order. Keep parent
                # rows ahead of children so foreign keys remain valid.
                manifest["domains"] = ALLOWED_DOMAINS
                target.write(json.dumps(record, separators=(",", ":")) + "\n")
                manifest_written = True
                continue
            if record.get("record_type") != "row":
                reject(f"unknown record type at line {line_number}")
            if keep_row(record):
                domain = record["domain"]
                counts[domain] = counts.get(domain, 0) + 1
                target.write(json.dumps(record, separators=(",", ":")) + "\n")

    if not manifest_written:
        reject("manifest is missing")
    print(json.dumps({"migrated_counts": counts}, sort_keys=True), file=sys.stderr)
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as exc:
        print(f"migration export rejected: {exc}", file=sys.stderr)
        raise SystemExit(1)
