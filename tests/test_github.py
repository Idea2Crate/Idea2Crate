import unittest
from unittest.mock import patch

from standards.bundle import PolicyError
from standards.github import GitHub, NoRedirect


class GitHubTests(unittest.TestCase):
    def test_truncated_tree_never_passes_validation(self):
        api = GitHub(token="synthetic-test-token")
        with patch.object(api, "request", return_value={"tree": [], "truncated": True}):
            with self.assertRaises(PolicyError):
                api.tree("Idea2Crate/Example", "a" * 40)

    def test_pagination_reads_all_pages(self):
        api = GitHub(token="synthetic-test-token")
        with patch.object(
            api, "request", side_effect=[list(range(100)), [100, 101]]
        ) as request:
            self.assertEqual(list(api.pages("/test?state=open")), list(range(102)))
            self.assertIn("&per_page=100&page=2", request.call_args.args[1])

    def test_pagination_is_bounded(self):
        api = GitHub(token="synthetic-test-token")
        with patch.object(api, "request", return_value=[{}] * 100):
            with self.assertRaises(PolicyError):
                list(api.pages("/test"))

    def test_redirect_does_not_forward_app_token(self):
        with self.assertRaises(PolicyError):
            NoRedirect().redirect_request(
                None, None, 302, "redirect", {}, "https://other.invalid"
            )

    def test_malformed_blob_is_rejected(self):
        api = GitHub(token="synthetic-test-token")
        for blob in (
            {"encoding": "utf-8", "content": "data"},
            {"encoding": "base64", "content": "?"},
        ):
            with (
                self.subTest(blob=blob),
                patch.object(api, "request", return_value=blob),
            ):
                with self.assertRaises(PolicyError):
                    api.blob("Idea2Crate/Example", "a" * 40)


if __name__ == "__main__":
    unittest.main()
