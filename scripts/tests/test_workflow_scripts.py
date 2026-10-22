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
import local_llm  # noqa: E402
import openspec_archive  # noqa: E402
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


if __name__ == "__main__":
    unittest.main()


class RemoteIniTests(unittest.TestCase):
    """The ini sent to the remote build host pins the image and drops the
    desktop-only init hooks (VS Code): the build host has no screen."""

    def test_pin_image_replaces_the_image_and_drops_init_hooks(self):
        import check_remote_image
        ini = "[stemloft]\nimage=ghcr.io/a:latest\npull=true\ninit_hooks=nohup x &\nentry=false\n"
        self.assertEqual(
            check_remote_image.pin_image(ini, "ghcr.io/a@sha256:1"),
            "[stemloft]\nimage=ghcr.io/a@sha256:1\npull=true\nentry=false\n",
        )

    def test_the_real_ini_keeps_working(self):
        import check_remote_image
        pinned = check_remote_image.pin_image(check_remote_image.DISTROBOX_INI.read_text(), "x@sha256:1")
        self.assertIn("image=x@sha256:1", pinned)
        self.assertNotIn("init_hooks=", pinned)
