#!/usr/bin/env python3
"""Verify the Codex native image bridge through a live Niffler endpoint.

The script never prints the API key or image Base64. It intentionally omits
X-OpenAI-Actor-Authorization and client-provided image tools so the server-side
bridge is the only mechanism that can make image generation available.
"""

from __future__ import annotations

import argparse
import base64
import json
import os
import struct
import sys
import urllib.error
import urllib.request
from dataclasses import dataclass
from pathlib import Path
from typing import Any


DEFAULT_POSITIVE_PROMPT = (
    "请把‘月光下划船的水獭’做成可直接查看的 1536×1024 PNG 成品，"
    "不要用文字描述。"
)
DEFAULT_NEGATIVE_PROMPT = (
    "解释 Responses API 的 image_generation 参数，并给出 Rust 示例；"
    "不要创建任何视觉成品。"
)
DEFAULT_TEXT_PROMPT = "计算 2+2，只回复结果。"


class VerificationError(RuntimeError):
    pass


@dataclass(frozen=True)
class SSEEvent:
    event: str
    data: dict[str, Any]


@dataclass(frozen=True)
class ResponseSummary:
    model: str
    response_id: str | None
    status: str
    text: str
    image_item: dict[str, Any] | None
    usage: dict[str, Any] | None
    event_types: tuple[str, ...]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="验证 Niffler 的 Codex 服务端原生生图桥接。",
    )
    parser.add_argument(
        "--base-url",
        default=os.environ.get("NIFFLER_TEST_BASE_URL"),
        help="测试网关地址，默认读取 NIFFLER_TEST_BASE_URL。",
    )
    parser.set_defaults(api_key=os.environ.get("NIFFLER_TEST_API_KEY"))
    parser.add_argument(
        "--models",
        default=os.environ.get("NIFFLER_TEST_MODELS", "gpt-5.5,gpt-5.6-sol"),
        help="逗号分隔的模型列表。",
    )
    parser.add_argument(
        "--output-dir",
        default=os.environ.get("NIFFLER_TEST_OUTPUT_DIR", "/tmp/niffler-image-bridge"),
        help="测试图片保存目录。",
    )
    parser.add_argument(
        "--timeout",
        type=int,
        default=int(os.environ.get("NIFFLER_TEST_TIMEOUT", "300")),
        help="单个请求超时秒数。",
    )
    parser.add_argument(
        "--include-follow-up",
        action="store_true",
        help="回放上一张图片并验证同会话续聊。",
    )
    return parser.parse_args()


def require_args(args: argparse.Namespace) -> tuple[str, str, list[str], Path]:
    if not args.base_url:
        raise VerificationError("缺少 --base-url 或 NIFFLER_TEST_BASE_URL。")
    if not args.api_key:
        raise VerificationError("缺少 --api-key 或 NIFFLER_TEST_API_KEY。")
    if args.timeout < 10 or args.timeout > 900:
        raise VerificationError("--timeout 必须在 10 到 900 秒之间。")
    models = [model.strip() for model in args.models.split(",") if model.strip()]
    if not models:
        raise VerificationError("至少需要一个测试模型。")
    output_dir = Path(args.output_dir).expanduser().resolve()
    output_dir.mkdir(parents=True, exist_ok=True)
    return args.base_url.rstrip("/"), args.api_key, models, output_dir


def iter_sse_events(response: Any) -> list[SSEEvent]:
    events: list[SSEEvent] = []
    event_name = "message"
    data_lines: list[str] = []

    def flush() -> None:
        nonlocal event_name, data_lines
        if not data_lines:
            event_name = "message"
            return
        data_text = "\n".join(data_lines).strip()
        if data_text.startswith("data:") or data_text.startswith("event:"):
            raise VerificationError("检测到重复 SSE 包裹。")
        if data_text != "[DONE]":
            try:
                payload = json.loads(data_text)
            except json.JSONDecodeError as exc:
                raise VerificationError(
                    f"SSE data 不是合法 JSON：{data_text[:160]}"
                ) from exc
            if not isinstance(payload, dict):
                raise VerificationError("SSE data JSON 不是对象。")
            payload_type = str(payload.get("type") or "").strip()
            if event_name != "message" and payload_type and event_name != payload_type:
                raise VerificationError(
                    f"SSE event 与 data.type 不一致：{event_name} != {payload_type}"
                )
            events.append(SSEEvent(event=event_name, data=payload))
        event_name = "message"
        data_lines = []

    for raw_line in response:
        line = raw_line.decode("utf-8", errors="strict").rstrip("\r\n")
        if not line:
            flush()
        elif line.startswith(":"):
            continue
        elif line.startswith("event:"):
            event_name = line[6:].strip()
        elif line.startswith("data:"):
            data_lines.append(line[5:].lstrip())
    flush()
    return events


def post_responses(
    base_url: str,
    api_key: str,
    model: str,
    input_value: Any,
    timeout: int,
) -> ResponseSummary:
    body = json.dumps(
        {
            "model": model,
            "input": input_value,
            "stream": True,
            "store": False,
        },
        ensure_ascii=False,
        separators=(",", ":"),
    ).encode("utf-8")
    responses_url = (
        f"{base_url}/responses" if base_url.endswith("/v1") else f"{base_url}/v1/responses"
    )
    request = urllib.request.Request(
        responses_url,
        data=body,
        method="POST",
        headers={
            "Authorization": f"Bearer {api_key}",
            "Content-Type": "application/json",
            "Accept": "text/event-stream",
            "User-Agent": "codex-tui/0.144.1 (niffler bridge verifier)",
            "originator": "codex-tui",
        },
    )
    try:
        with urllib.request.urlopen(request, timeout=timeout) as response:
            if response.status != 200:
                raise VerificationError(f"HTTP 状态异常：{response.status}")
            content_type = response.headers.get("Content-Type", "")
            if "text/event-stream" not in content_type.lower():
                raise VerificationError(f"响应不是 SSE：{content_type}")
            events = iter_sse_events(response)
    except urllib.error.HTTPError as exc:
        error_body = exc.read(1024).decode("utf-8", errors="replace")
        raise VerificationError(f"HTTP {exc.code}: {error_body}") from exc
    except urllib.error.URLError as exc:
        raise VerificationError(f"请求失败：{exc.reason}") from exc
    except (TimeoutError, ConnectionResetError) as exc:
        raise VerificationError(f"请求连接异常：{exc}") from exc

    image_item: dict[str, Any] | None = None
    text_parts: list[str] = []
    completed: dict[str, Any] | None = None
    for item in events:
        payload = item.data
        payload_type = payload.get("type")
        if payload_type == "response.output_text.delta":
            text_parts.append(str(payload.get("delta") or ""))
        elif payload_type == "response.output_text.done" and not text_parts:
            text_parts.append(str(payload.get("text") or ""))
        elif payload_type == "response.output_item.done":
            candidate = payload.get("item")
            if isinstance(candidate, dict) and candidate.get("type") == "image_generation_call":
                if candidate.get("result"):
                    image_item = candidate
        elif payload_type == "response.completed":
            response_value = payload.get("response")
            if isinstance(response_value, dict):
                completed = response_value

    if completed is None:
        raise VerificationError("缺少 response.completed。")
    if completed.get("status") != "completed":
        raise VerificationError(f"响应终态不是 completed：{completed.get('status')}")
    return ResponseSummary(
        model=model,
        response_id=str(completed.get("id") or "").strip() or None,
        status="completed",
        text="".join(text_parts).strip(),
        image_item=image_item,
        usage=completed.get("usage") if isinstance(completed.get("usage"), dict) else None,
        event_types=tuple(item.event for item in events),
    )


def decode_image(
    image_item: dict[str, Any], output_path: Path
) -> tuple[int, str, int, int]:
    result = image_item.get("result")
    if not isinstance(result, str) or not result.strip():
        raise VerificationError("图片结果为空。")
    if image_item.get("status") != "completed":
        raise VerificationError(f"图片终态不是 completed：{image_item.get('status')}")
    try:
        image_bytes = base64.b64decode(result, validate=True)
    except ValueError as exc:
        raise VerificationError("图片结果不是合法 Base64。") from exc
    image_format = str(image_item.get("output_format") or "png").lower()
    signatures = {
        "png": b"\x89PNG\r\n\x1a\n",
        "jpeg": b"\xff\xd8\xff",
        "jpg": b"\xff\xd8\xff",
        "webp": b"RIFF",
    }
    signature = signatures.get(image_format)
    if signature is not None and not image_bytes.startswith(signature):
        raise VerificationError(f"图片内容与格式不符：{image_format}")
    if image_format != "png":
        raise VerificationError(f"测试要求 PNG，实际返回：{image_format}")
    if len(image_bytes) < 24 or image_bytes[12:16] != b"IHDR":
        raise VerificationError("PNG 缺少有效 IHDR 尺寸信息。")
    width, height = struct.unpack(">II", image_bytes[16:24])
    if width <= 0 or height <= 0:
        raise VerificationError(f"PNG 尺寸无效：{width}×{height}")
    output_path.write_bytes(image_bytes)
    return len(image_bytes), image_format, width, height


def replay_input(image_item: dict[str, Any]) -> list[dict[str, Any]]:
    replayed = {
        key: image_item[key]
        for key in ("type", "id", "status", "result")
        if key in image_item
    }
    return [
        replayed,
        {
            "type": "message",
            "role": "user",
            "content": "只用一句话描述刚才的视觉成品，不要再次创建。",
        },
    ]


def compact_usage(usage: dict[str, Any] | None) -> str:
    if not usage:
        return "usage=missing"
    return "usage=" + json.dumps(usage, ensure_ascii=False, separators=(",", ":"))


def main() -> int:
    args = parse_args()
    base_url, api_key, models, output_dir = require_args(args)
    print(f"base_url={base_url} models={','.join(models)} output_dir={output_dir}")
    print("custom_actor_authorization=absent")

    for model in models:
        text_summary = post_responses(
            base_url, api_key, model, DEFAULT_TEXT_PROMPT, args.timeout
        )
        if text_summary.image_item is not None or text_summary.text.strip() != "4":
            raise VerificationError(
                f"{model} 普通文本请求结果异常：text={text_summary.text!r}"
            )
        print(f"PASS model={model} case=text {compact_usage(text_summary.usage)}")

        negative_summary = post_responses(
            base_url, api_key, model, DEFAULT_NEGATIVE_PROMPT, args.timeout
        )
        if negative_summary.image_item is not None or not negative_summary.text:
            raise VerificationError(f"{model} 负向语料错误调用了图片工具。")
        print(f"PASS model={model} case=negative {compact_usage(negative_summary.usage)}")

        image_summary = post_responses(
            base_url, api_key, model, DEFAULT_POSITIVE_PROMPT, args.timeout
        )
        if image_summary.image_item is None:
            raise VerificationError(f"{model} 正向语料没有返回图片工具结果。")
        output_format = str(image_summary.image_item.get("output_format") or "png").lower()
        suffix = "jpg" if output_format in {"jpeg", "jpg"} else output_format
        output_path = output_dir / f"{model.replace('/', '_')}.{suffix}"
        image_size, image_format, width, height = decode_image(
            image_summary.image_item, output_path
        )
        if (width, height) != (1536, 1024):
            raise VerificationError(
                f"{model} 图片尺寸异常：{width}×{height}，期望 1536×1024。"
            )
        print(
            f"PASS model={model} case=image format={image_format} "
            f"dimensions={width}x{height} "
            f"bytes={image_size} path={output_path} {compact_usage(image_summary.usage)}"
        )

        if args.include_follow_up:
            follow_up = post_responses(
                base_url,
                api_key,
                model,
                replay_input(image_summary.image_item),
                args.timeout,
            )
            if follow_up.image_item is not None or not follow_up.text:
                raise VerificationError(f"{model} 图片续聊结果异常。")
            print(
                f"PASS model={model} case=follow_up "
                f"{compact_usage(follow_up.usage)}"
            )

    print("ALL PASS")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except VerificationError as exc:
        print(f"FAIL {exc}", file=sys.stderr)
        raise SystemExit(1) from exc
