"""Tests for the tool-neutral policy scripts (guard_command, check_commit_msg).

Run: python3 -m unittest discover -s scripts/tests
"""
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

import check_commit_msg  # noqa: E402
import guard_command  # noqa: E402

BAD_SUBJECT = "oo" + "ps"


class CommitMessageTests(unittest.TestCase):
    def test_conventional_subject_passes(self):
        self.assertIsNone(check_commit_msg.problem("feat(core): add session\n\nbody"))

    def test_breaking_change_marker_passes(self):
        self.assertIsNone(check_commit_msg.problem("refactor!: drop old api"))

    def test_free_form_subject_fails(self):
        self.assertIn(BAD_SUBJECT, check_commit_msg.problem(BAD_SUBJECT))

    def test_git_generated_merge_subject_is_exempt(self):
        self.assertIsNone(check_commit_msg.problem("Merge pull request #4 from x/y"))

    def test_comment_lines_from_the_editor_are_ignored(self):
        self.assertIsNone(check_commit_msg.problem("# Please enter...\nfix: thing"))


class GuardCommandTests(unittest.TestCase):
    def test_npm_install_with_package_is_blocked(self):
        self.assertIsNotNone(guard_command.violation("npm install lodash"))

    def test_npm_ci_with_redirection_is_allowed(self):
        self.assertIsNone(guard_command.violation("npm ci 2>&1"))

    def test_plain_npm_install_is_allowed(self):
        self.assertIsNone(guard_command.violation("cd ui && npm install"))

    def test_bad_inline_commit_message_is_blocked(self):
        self.assertIsNotNone(guard_command.violation(f"git commit -m '{BAD_SUBJECT}'"))

    def test_good_inline_commit_message_is_allowed(self):
        self.assertIsNone(guard_command.violation("git add . && git commit -m 'docs: x'"))

    def test_bad_heredoc_commit_message_is_blocked(self):
        cmd = f"git commit -F- <<EOF\n{BAD_SUBJECT}\nEOF"
        self.assertIsNotNone(guard_command.violation(cmd))

    def test_words_git_commit_inside_a_heredoc_body_are_not_a_commit(self):
        cmd = "cat > notes.txt <<'EOF'\n#!/usr/bin/env python3\ngit commit\nEOF"
        self.assertIsNone(guard_command.violation(cmd))


if __name__ == "__main__":
    unittest.main()
