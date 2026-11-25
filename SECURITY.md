# Security Policy

## Supported Versions

Only the latest release of SARA receives security fixes. `sara-core`,
`sara-cli` and `sara-mcp` are released together, so this applies to all three
crates. Fixes ship in a new release: upgrade to it to get them.

| Version | Supported |
|---------|-----------|
| Latest release | :white_check_mark: |
| Older releases | :x: |

## Reporting a Vulnerability

Please do not report security vulnerabilities through public GitHub issues,
discussions or pull requests.

Report them privately instead, from the
[Security tab](https://github.com/cledouarec/sara/security) of the repository:
click **Report a vulnerability**, or open
[a new advisory](https://github.com/cledouarec/sara/security/advisories/new)
directly.

Include as much of the following as you can:

- The affected component (core library, CLI or MCP server) and version (output
  of `sara --version`)
- A description of the vulnerability and its impact
- The steps, files and configuration needed to reproduce it
- A proof of concept, if you have one
- A suggested fix, if you have one

## What to Expect

SARA is maintained on a best-effort basis, so there is no guaranteed response
time. Each report is handled in its private advisory, where you are kept
informed of its progress:

1. The maintainers confirm the vulnerability and assess its impact, and may ask
   you for more details. If the report is not accepted, they explain why.
2. A fix is prepared privately and shipped in a new release.
3. The advisory is then published, with a CVE when warranted, and you are
   credited unless you prefer to stay anonymous.

Please keep the vulnerability confidential until the advisory is published.
