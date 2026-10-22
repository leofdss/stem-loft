"""Tests for the OpenSpec / bilingual / agent-agnostic / local-model scripts.

Run: python3 -m unittest discover -s scripts/tests
"""
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

import check_agent_agnostic  # noqa: E402
import check_bilingual_parity as parity  # noqa: E402
import check_frozen  # noqa: E402
import local_llm  # noqa: E402
import openspec_archive  # noqa: E402
import llm_task  # noqa: E402
import translate_md  # noqa: E402


class CounterpartTests(unittest.TestCase):
    def test_plain_doc_pairs_with_pt_br_suffix(self):
        self.assertEqual(parity.counterpart_path(Path("docs/a.md")), Path("docs/a.pt-BR.md"))
        self.assertEqual(parity.counterpart_path(Path("docs/a.pt-BR.md")), Path("docs/a.md"))

    def test_change_delta_spec_pairs_with_i18n_mirror(self):
        en = Path("openspec/changes/task-002-x/specs/project-persistence/spec.md")
        pt = Path("openspec/changes/task-002-x/i18n/pt-BR/specs/project-persistence/spec.md")
        self.assertEqual(parity.counterpart_path(en), pt)
        self.assertEqual(parity.counterpart_path(pt), en)
        self.assertTrue(parity.is_pt_br(pt))
        self.assertFalse(parity.is_pt_br(en))

    def test_main_spec_uses_the_plain_suffix(self):
        en = Path("openspec/specs/session-state/spec.md")
        self.assertEqual(parity.counterpart_path(en), Path("openspec/specs/session-state/spec.pt-BR.md"))

    def test_missing_openspec_translation_is_reported(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            change = root / "openspec" / "changes" / "c"
            change.mkdir(parents=True)
            (change / "proposal.md").write_text("# P\n")
            (change / "tasks.md").write_text("# T\n")
            (change / "tasks.pt-BR.md").write_text("# T\n")
            problems = parity.missing_translations(root)
            self.assertEqual(len(problems), 1)
            self.assertIn("proposal.md", problems[0])


class BoardEntryTests(unittest.TestCase):
    BOARD = (
        "# header\n\n"
        '[task-001-a]\nstatus = "review"\nbranch = "feat/a"\n\n'
        '[task-002-b]\nstatus = "backlog"\nbranch = "feat/b"\n'
    )

    def test_drops_only_the_named_table(self):
        out = openspec_archive.drop_board_entry(self.BOARD, "task-001-a")
        self.assertNotIn("task-001-a", out)
        self.assertNotIn("feat/a", out)
        self.assertIn('[task-002-b]\nstatus = "backlog"', out)
        self.assertTrue(out.startswith("# header"))

    def test_last_table_can_be_dropped(self):
        out = openspec_archive.drop_board_entry(self.BOARD, "task-002-b")
        self.assertNotIn("feat/b", out)
        self.assertIn("feat/a", out)


class LocalLlmTests(unittest.TestCase):
    def test_env_url_wins_and_is_normalised(self):
        self.assertEqual(local_llm.server_url({"STEMLOFT_OLLAMA_URL": "http://h:11434/"}, Path("/nonexistent")), "http://h:11434")

    def test_url_read_from_host_local(self):
        with tempfile.NamedTemporaryFile("w", suffix=".local", delete=False) as fh:
            fh.write("export STEMLOFT_REMOTE_HOST=me@box\nexport STEMLOFT_OLLAMA_URL=http://10.0.0.2:11434\n")
        self.assertEqual(local_llm.server_url({}, Path(fh.name)), "http://10.0.0.2:11434")

    def test_no_url_means_unavailable(self):
        self.assertIsNone(local_llm.server_url({}, Path("/nonexistent")))

    def test_request_uses_the_role_model_and_think_flag(self):
        req = local_llm.build_request({"model": "m:1b", "think": False}, "hi", "sys")
        self.assertEqual(req["model"], "m:1b")
        self.assertFalse(req["think"])
        self.assertEqual([m["role"] for m in req["messages"]], ["system", "user"])

    def test_every_role_in_models_toml_names_a_model(self):
        roles = local_llm.load_roles()
        self.assertTrue(roles)
        for name, role in roles.items():
            self.assertTrue(role.get("model"), name)
            self.assertTrue(role.get("use"), name)


class AgentAgnosticTests(unittest.TestCase):
    def make_repo(self, root: Path):
        (root / ".agents" / "skills" / "x").mkdir(parents=True)
        (root / ".agents" / "skills" / "x" / "SKILL.md").write_text("---\nname: x\n---\nRun the tests.\n")
        (root / "AGENTS.md").write_text("# Agents\n")
        (root / ".claude" / "agents").mkdir(parents=True)
        (root / ".claude" / "skills").symlink_to("../.agents/skills")

    def test_thin_adapters_pass(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self.make_repo(root)
            (root / "CLAUDE.md").write_text("@AGENTS.md\n")
            (root / ".claude" / "agents" / "r.md").write_text("Follow `.agents/skills/x/SKILL.md`.\n")
            self.assertEqual(check_agent_agnostic.check(root), [])

    def test_rules_living_only_in_a_tool_file_fail(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self.make_repo(root)
            (root / "CLAUDE.md").write_text("Rule\n" * 60)
            (root / ".claude" / "agents" / "r.md").write_text("Review everything carefully.\n")
            problems = check_agent_agnostic.check(root)
            self.assertTrue(any("CLAUDE.md: must point to AGENTS.md" in p for p in problems))
            self.assertTrue(any("lines" in p and "CLAUDE.md" in p for p in problems))
            self.assertTrue(any("r.md" in p for p in problems))

    def test_hook_with_inline_logic_fails(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self.make_repo(root)
            hooks = root / ".claude" / "hooks"
            hooks.mkdir()
            (hooks / "h.py").write_text("x = 1\n" * 5)
            self.assertTrue(any("delegate to scripts/" in p for p in check_agent_agnostic.check(root)))

    def test_tool_mechanics_in_a_canonical_playbook_fail(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self.make_repo(root)
            (root / ".agents" / "skills" / "x" / "SKILL.md").write_text("The PostToolUse hook runs it.\n")
            self.assertTrue(any("PostToolUse" in p for p in check_agent_agnostic.check(root)))


SPEC_EN = """## Purpose

Provides peaks for `waveform_ready`.

## ADDED Requirements

### Requirement: Peaks computed once
The core SHALL compute peaks once.

#### Scenario: No cache yet
- **WHEN** a project opens
- **THEN** peaks are computed
"""


class OpenSpecParityTests(unittest.TestCase):
    def test_translated_prose_with_english_structure_matches(self):
        pt = SPEC_EN.replace("Provides peaks", "Fornece picos").replace("Peaks computed once", "Picos uma vez") \
            .replace("The core SHALL compute peaks once.", "O núcleo SHALL calcular os picos uma vez.") \
            .replace("No cache yet", "Sem cache").replace("a project opens", "um projeto abre") \
            .replace("peaks are computed", "os picos são calculados")
        self.assertEqual(parity.openspec_signature(SPEC_EN), parity.openspec_signature(pt))

    def test_translated_section_heading_is_drift(self):
        pt = SPEC_EN.replace("## Purpose", "## Propósito")
        self.assertNotEqual(parity.openspec_signature(SPEC_EN), parity.openspec_signature(pt))

    def test_translated_requirement_prefix_is_drift(self):
        pt = SPEC_EN.replace("### Requirement:", "### Requisito:")
        self.assertNotEqual(parity.openspec_signature(SPEC_EN), parity.openspec_signature(pt))

    def test_translated_keyword_is_drift(self):
        pt = SPEC_EN.replace("SHALL", "DEVE")
        self.assertNotEqual(parity.openspec_signature(SPEC_EN), parity.openspec_signature(pt))

    def test_task_group_names_may_be_translated(self):
        en = "# Tasks\n\n## 1. Acceptance criteria\n\n- [ ] 1.1 Do it\n"
        pt = "# Tarefas\n\n## 1. Critérios de aceite\n\n- [ ] 1.1 Fazer\n"
        self.assertEqual(parity.openspec_signature(en), parity.openspec_signature(pt))


class TranslateMdTests(unittest.TestCase):
    def test_model_only_sees_prose(self):
        texts = [p.text for p in translate_md.split(SPEC_EN) if p.translate]
        self.assertEqual(texts, [
            "Provides peaks for `waveform_ready`.",
            "Peaks computed once",
            "The core SHALL compute peaks once.",
            "No cache yet",
            "a project opens",
            "peaks are computed",
        ])

    def test_structure_is_reassembled_verbatim(self):
        pieces = translate_md.split(SPEC_EN)
        fake = [f"<{i}>" for i, p in enumerate(p for p in pieces if p.translate)]
        out = translate_md.join(pieces, fake)
        self.assertIn("## Purpose\n", out)
        self.assertIn("### Requirement: <1>\n", out)
        self.assertIn("#### Scenario: <3>\n- **WHEN** <4>\n- **THEN** <5>", out)
        self.assertEqual(translate_md.join(pieces, [p.text for p in pieces if p.translate]), SPEC_EN)

    def test_code_blocks_and_checkbox_numbers_are_kept(self):
        md = "- [ ] 1.2 Run `cargo test`\n\n```bash\ncargo test\n```\n"
        pieces = translate_md.split(md)
        self.assertEqual([p.text for p in pieces if p.translate], ["Run `cargo test`"])
        self.assertEqual(pieces[0].prefix, "- [ ] 1.2 ")

    def test_segment_checks(self):
        self.assertIsNone(translate_md.segment_problem("The core SHALL use `x`.", "O núcleo SHALL usar `x`."))
        self.assertIn("code span", translate_md.segment_problem("Use `x`.", "Use x."))
        self.assertIn("SHALL/MUST", translate_md.segment_problem("It SHALL go.", "Ele DEVE ir."))

    def test_thinking_preamble_is_stripped(self):
        self.assertEqual(local_llm.strip_thinking("reasoning...</think>\n\nfeat: x"), "feat: x")
        self.assertEqual(local_llm.strip_thinking("feat: x"), "feat: x")


class LlmTaskTests(unittest.TestCase):
    FILE = "pub fn a() {}\n\n#[cfg(test)]\nmod tests { #[test] fn t() {} }\n"

    def test_split_freezes_everything_from_the_marker(self):
        section, frozen = llm_task.split(self.FILE, "#[cfg(test)]")
        self.assertEqual(section, "pub fn a() {}\n\n")
        self.assertTrue(frozen.startswith("#[cfg(test)]"))
        self.assertEqual(section + frozen, self.FILE)

    def test_missing_marker_is_an_error_not_a_silent_full_edit(self):
        with self.assertRaises(ValueError):
            llm_task.split(self.FILE, "#[cfg(nope)]")

    def test_reply_section_is_the_largest_fenced_block(self):
        reply = "Here:\n```rust\nuse x;\n```\nand the file:\n```rust\npub fn a() { 1 }\npub fn b() {}\n```\n"
        self.assertEqual(llm_task.extract_section(reply), "pub fn a() { 1 }\npub fn b() {}\n\n")

    def test_echoed_line_numbers_are_stripped(self):
        reply = "```rust\n 9 | fn a() {\n10 |     1\n11 | }\n```"
        self.assertEqual(llm_task.extract_section(reply), "fn a() {\n    1\n}\n\n")

    def test_code_that_merely_contains_pipes_is_left_alone(self):
        reply = "```rust\nlet x = a | b;\nfn f() {}\n```"
        self.assertEqual(llm_task.extract_section(reply), "let x = a | b;\nfn f() {}\n\n")

    def test_a_hung_check_is_killed_and_explained(self):
        ok, out, seconds = llm_task.run_check("sleep 30", 1)
        self.assertFalse(ok)
        self.assertIn("HUNG", out)
        self.assertLess(seconds, 10)

    def test_reply_without_a_block_is_rejected(self):
        self.assertIsNone(llm_task.extract_section("I would change the function a."))

    def test_feedback_keeps_errors_and_drops_warnings(self):
        out = (
            "   Compiling stemloft v0.1.0\n"
            "warning: unused variable: `x`\n --> src/a.rs:1:1\n\n"
            "error[E0425]: cannot find value `y`\n --> src/a.rs:2:5\n\n"
            "test result: FAILED. 1 passed; 1 failed\n"
        )
        fb = llm_task.trim_feedback(out)
        self.assertIn("error[E0425]", fb)
        self.assertIn("test result: FAILED", fb)
        self.assertNotIn("unused variable", fb)
        self.assertNotIn("Compiling", fb)

    def test_feedback_is_capped(self):
        fb = llm_task.trim_feedback("\n".join(f"error: e{i}" for i in range(500)))
        self.assertLessEqual(len(fb.splitlines()), llm_task.MAX_FEEDBACK_LINES + 1)
        self.assertIn("more lines trimmed", fb)

    def test_numbered_lines_match_file_line_numbers(self):
        self.assertEqual(llm_task.numbered("a\nb", 9), " 9 | a\n10 | b")

    def test_excerpts_say_whose_line_failed(self):
        section = "fn a() {\n    x?\n}\n\n"            # lines 1-4
        frozen = "#[cfg(test)]\nmod t {\n    assert!(msg.contains(\"exact\"));\n}\n"  # line 5 on
        out = (
            "error[E0277]: `?` couldn't convert\n  --> src/core/f.rs:2:5\n"
            "thread 't' panicked at src/core/f.rs:7:5:\nnope\n"
            "  --> src/other.rs:2:1\n"
        )
        ex = llm_task.excerpts(out, "src-tauri/src/core/f.rs", section, frozen)
        self.assertIn("Line 2 is in your section", ex)
        self.assertIn("2 |     x?", ex)
        self.assertIn("Line 7 is in FROZEN part", ex)
        self.assertIn('assert!(msg.contains(\"exact\"))'.replace('\\"', '"'), ex)
        self.assertEqual(ex.count("Line "), 2, "locations in other files are ignored")

    def test_stalled_note_reaches_the_prompt(self):
        prompt = llm_task.build_prompt("T", [], "f.rs", "", "fn a(){}", "error: same", 3, "Same as before.")
        self.assertIn("# STALLED\n\nSame as before.", prompt)
        self.assertIn("1 | fn a(){}", prompt)

    def test_prompt_shows_the_frozen_part_read_only_and_the_last_failure(self):
        prompt = llm_task.build_prompt("Do X.", [("ctx.md", "C")], "f.rs", "#[cfg(test)] frozen", "fn a(){}", "error: boom", 2)
        self.assertIn("# FROZEN part of f.rs", prompt)
        self.assertIn("#[cfg(test)] frozen", prompt)
        self.assertIn("after iteration 1 (FAILED)", prompt)
        self.assertIn("error: boom", prompt)
        self.assertLess(prompt.index("# Read-only context: ctx.md"), prompt.index("# YOUR SECTION"))


class CheckFrozenTests(unittest.TestCase):
    def repo_with_contract(self, root: Path) -> Path:
        import subprocess
        def git(*args):
            subprocess.run(["git", *args], cwd=root, check=True, capture_output=True)
        git("init", "-q")
        git("config", "user.email", "t@example.com")
        git("config", "user.name", "t")
        git("config", "commit.gpgsign", "false")
        git("config", "core.hooksPath", "/dev/null")
        f = root / "lib.rs"
        f.write_text("pub fn a() { todo!() }\n\n#[cfg(test)]\nmod tests { #[test] fn t() { assert_eq!(1, 1); } }\n")
        git("add", "lib.rs")
        git("commit", "-q", "-m", "test: contract")
        return f

    def test_implementing_the_section_keeps_the_frozen_part_ok(self):
        with tempfile.TemporaryDirectory() as tmp:
            f = self.repo_with_contract(Path(tmp))
            f.write_text(f.read_text().replace("todo!()", "1;"))
            code, _ = check_frozen.compare(f, "#[cfg(test)]", "HEAD")
            self.assertEqual(code, 0)

    def test_editing_a_test_is_caught_with_a_diff(self):
        with tempfile.TemporaryDirectory() as tmp:
            f = self.repo_with_contract(Path(tmp))
            f.write_text(f.read_text().replace("assert_eq!(1, 1)", "assert!(true)"))
            code, message = check_frozen.compare(f, "#[cfg(test)]", "HEAD")
            self.assertEqual(code, 1)
            self.assertIn("-mod tests { #[test] fn t() { assert_eq!(1, 1); } }", message)

    def test_deleting_the_marker_is_not_a_pass(self):
        with tempfile.TemporaryDirectory() as tmp:
            f = self.repo_with_contract(Path(tmp))
            f.write_text("pub fn a() {}\n")
            self.assertEqual(check_frozen.compare(f, "#[cfg(test)]", "HEAD")[0], 2)

    def test_an_uncommitted_contract_cant_be_checked(self):
        with tempfile.TemporaryDirectory() as tmp:
            self.repo_with_contract(Path(tmp))
            g = Path(tmp) / "new.rs"
            g.write_text("#[cfg(test)]\n")
            self.assertEqual(check_frozen.compare(g, "#[cfg(test)]", "HEAD")[0], 2)


if __name__ == "__main__":
    unittest.main()


class RemoteWorktreeTests(unittest.TestCase):
    """distrobox/remote/lib.sh and check_remote_image.py, run from a linked
    git worktree: host.local comes from the main checkout, and the remote
    copy gets its own directory."""

    def make_repo_with_worktree(self, root: Path) -> tuple[Path, Path]:
        import shutil
        import subprocess
        main = root / "main"
        (main / "distrobox" / "remote").mkdir(parents=True)
        shutil.copy(Path(__file__).resolve().parents[2] / "distrobox" / "remote" / "lib.sh",
                    main / "distrobox" / "remote" / "lib.sh")
        def git(*args, cwd=main):
            subprocess.run(["git", *args], cwd=cwd, check=True, capture_output=True)
        git("init", "-q")
        git("config", "user.email", "t@example.com")
        git("config", "user.name", "t")
        git("config", "commit.gpgsign", "false")
        git("config", "core.hooksPath", "/dev/null")
        git("add", ".")
        git("commit", "-q", "-m", "chore: init")
        (main / "distrobox" / "remote" / "host.local").write_text(
            "export STEMLOFT_REMOTE_HOST=me@box\nexport STEMLOFT_REMOTE_PATH=/srv/stem-loft/\n")
        worktree = root / "wt-cho"
        git("worktree", "add", "-q", str(worktree))
        return main, worktree

    def source_lib(self, checkout: Path) -> str:
        import subprocess
        return subprocess.run(
            ["bash", "-c", 'source "$1/distrobox/remote/lib.sh" && '
             'printf "%s|%s|%s" "$STEMLOFT_REMOTE_HOST" "$REMOTE_ROOT" "$STEMLOFT_IS_WORKTREE"',
             "_", str(checkout)],
            capture_output=True, text=True, check=True).stdout

    def test_main_checkout_uses_the_configured_path(self):
        with tempfile.TemporaryDirectory() as tmp:
            main, _ = self.make_repo_with_worktree(Path(tmp))
            self.assertEqual(self.source_lib(main), "me@box|/srv/stem-loft|0")

    def test_worktree_reads_main_host_local_and_gets_its_own_remote_dir(self):
        with tempfile.TemporaryDirectory() as tmp:
            _, worktree = self.make_repo_with_worktree(Path(tmp))
            self.assertEqual(self.source_lib(worktree),
                             "me@box|/srv/stem-loft-worktrees/wt-cho|1")

    def test_image_check_finds_host_local_from_a_worktree(self):
        import check_remote_image
        with tempfile.TemporaryDirectory() as tmp:
            main, worktree = self.make_repo_with_worktree(Path(tmp))
            saved = check_remote_image.REPO_ROOT, check_remote_image.HOST_LOCAL
            try:
                check_remote_image.REPO_ROOT = worktree
                check_remote_image.HOST_LOCAL = worktree / "distrobox" / "remote" / "host.local"
                self.assertEqual(check_remote_image.host_local_path().resolve(),
                                 (main / "distrobox" / "remote" / "host.local").resolve())
                self.assertEqual(check_remote_image.read_remote_host(), "me@box")
            finally:
                check_remote_image.REPO_ROOT, check_remote_image.HOST_LOCAL = saved


class CheckChangedRemoteTests(unittest.TestCase):
    """check_changed.py's post-edit cargo check goes to the remote build
    host only when one is configured and answers; otherwise it runs
    locally, silently."""

    def test_local_checks_env_forces_local(self):
        import os
        import check_changed
        os.environ["STEMLOFT_LOCAL_CHECKS"] = "1"
        try:
            self.assertIsNone(check_changed.remote_runner(Path(__file__).resolve().parents[2]))
        finally:
            del os.environ["STEMLOFT_LOCAL_CHECKS"]

    def test_no_host_configured_means_local(self):
        import shutil
        import check_changed
        with tempfile.TemporaryDirectory() as tmp:
            remote = Path(tmp) / "distrobox" / "remote"
            remote.mkdir(parents=True)
            shutil.copy(Path(__file__).resolve().parents[2] / "distrobox" / "remote" / "lib.sh", remote / "lib.sh")
            self.assertIsNone(check_changed.remote_runner(Path(tmp)))

    def test_no_remote_scripts_means_local(self):
        import check_changed
        with tempfile.TemporaryDirectory() as tmp:
            self.assertIsNone(check_changed.remote_runner(Path(tmp)))
