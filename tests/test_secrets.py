"""Preserve the credential guard's observable rejection and redaction check."""

import io
import tempfile
import unittest
from contextlib import redirect_stderr
from pathlib import Path

from hooks.check_secrets import scan


class CredentialGuardTests(unittest.TestCase):
    def test_secret_guard_does_not_echo_credentials(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "fixture.txt"
            for fake in (
                "gh" + "p_" + "a" * 36,
                "gh" + "s_123456_" + "eyJhbGciOiJSUzI1NiJ9.eyJleHAiOjF9.c3ludGhldGlj",
            ):
                path.write_text(fake)
                errors = io.StringIO()
                with redirect_stderr(errors):
                    self.assertEqual(scan([str(path)]), 1)
                self.assertNotIn(fake, errors.getvalue())
            path.write_text("ordinary source")
            self.assertEqual(scan([str(path)]), 0)


if __name__ == "__main__":
    unittest.main()
