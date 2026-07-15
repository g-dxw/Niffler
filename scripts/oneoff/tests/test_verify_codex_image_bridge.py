from __future__ import annotations

import base64
import contextlib
import importlib.util
import io
import json
import struct
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock


SCRIPT_PATH = Path(__file__).resolve().parents[1] / "verify_codex_image_bridge.py"
SPEC = importlib.util.spec_from_file_location("verify_codex_image_bridge", SCRIPT_PATH)
assert SPEC is not None and SPEC.loader is not None
bridge = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = bridge
SPEC.loader.exec_module(bridge)


def png_bytes(width: int, height: int) -> bytes:
    return b"\x89PNG\r\n\x1a\n" + struct.pack(">I4sII", 13, b"IHDR", width, height)


class FakeResponse:
    status = 200
    headers = {"Content-Type": "text/event-stream"}

    def __enter__(self) -> "FakeResponse":
        return self

    def __exit__(self, *_args: object) -> None:
        return None

    def __iter__(self):
        payload = {
            "type": "response.completed",
            "response": {"id": "resp_test", "status": "completed"},
        }
        yield f"data: {json.dumps(payload)}\n\n".encode()


class VerifyCodexImageBridgeTests(unittest.TestCase):
    def test_api_key_is_read_from_environment_not_command_line(self) -> None:
        with mock.patch.dict(bridge.os.environ, {"NIFFLER_TEST_API_KEY": "env-key"}):
            with mock.patch.object(sys, "argv", ["verify_codex_image_bridge.py"]):
                args = bridge.parse_args()
        self.assertEqual(args.api_key, "env-key")

        with mock.patch.object(
            sys,
            "argv",
            ["verify_codex_image_bridge.py", "--api-key", "command-line-key"],
        ):
            with contextlib.redirect_stderr(io.StringIO()):
                with self.assertRaises(SystemExit):
                    bridge.parse_args()

    def test_decode_image_returns_png_dimensions(self) -> None:
        raw = png_bytes(1536, 1024)
        item = {
            "type": "image_generation_call",
            "status": "completed",
            "output_format": "png",
            "result": base64.b64encode(raw).decode(),
        }
        with tempfile.TemporaryDirectory() as directory:
            result = bridge.decode_image(item, Path(directory) / "image.png")

        self.assertEqual(result, (len(raw), "png", 1536, 1024))

    def test_post_responses_accepts_root_or_v1_base_url(self) -> None:
        requested_urls: list[str] = []

        def fake_urlopen(request, timeout):
            self.assertEqual(timeout, 30)
            requested_urls.append(request.full_url)
            return FakeResponse()

        with mock.patch.object(bridge.urllib.request, "urlopen", side_effect=fake_urlopen):
            bridge.post_responses("https://niffler.org", "test-key", "gpt-test", "hi", 30)
            bridge.post_responses("https://niffler.org/v1", "test-key", "gpt-test", "hi", 30)

        self.assertEqual(
            requested_urls,
            [
                "https://niffler.org/v1/responses",
                "https://niffler.org/v1/responses",
            ],
        )


if __name__ == "__main__":
    unittest.main()
