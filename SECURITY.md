# Security Policy

## Reporting a Vulnerability

Please report security vulnerabilities **privately** through GitHub's private
vulnerability reporting, not through public issues or pull requests:

- Open a report at:
  https://github.com/ancient0328/arcRTC/security/advisories/new

When reporting, include where possible:

- the affected component (`Kernel/` or `implementations/`) and version/commit,
- a description of the issue and its impact,
- reproduction steps or a proof of concept,
- any suggested remediation.

Please do **not** disclose the issue publicly until a fix has been released
and coordinated through the advisory.

## Scope

In scope:

- the published source under `Kernel/` and `implementations/`.

Out of scope:

- third-party dependencies (please report those to their upstream projects),
- issues that require a non-default, intentionally unsafe configuration.

## Response

Reports are triaged through the GitHub Security Advisory linked above.
Confirmed vulnerabilities are addressed by a coordinated fix and advisory.

---

## セキュリティ報告（日本語）

脆弱性は公開 issue / pull request ではなく、GitHub の **private vulnerability
reporting** で**非公開**にご報告ください。

- 報告先: https://github.com/ancient0328/arcRTC/security/advisories/new

可能な範囲で、対象コンポーネント（`Kernel/` または `implementations/`）・バージョン
／コミット・影響・再現手順（PoC）・想定される修正案を添えてください。修正が公開され、
アドバイザリで調整されるまでは公開を控えてください。

対象は公開ソース（`Kernel/` / `implementations/`）です。第三者依存は各上流へご報告ください。
