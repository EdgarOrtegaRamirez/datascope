# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.1.x   | ✅        |

## Reporting a Vulnerability

If you discover a security vulnerability in DataScope, please report it responsibly:

1. **Do not** open a public GitHub issue
2. Email the maintainer with details of the vulnerability
3. Include steps to reproduce if possible

You will receive a response within 48 hours.

## Security Measures

DataScope is designed with security in mind:

- **No network access** — All processing happens locally. No data is transmitted.
- **Input size limit** — Files larger than 500 MB are rejected to prevent memory exhaustion.
- **No code execution** — DataScope parses data files only; it does not evaluate or execute any code.
- **No secrets stored** — No credentials, tokens, or secrets are stored or transmitted.
- **Safe file operations** — File paths are validated; no path traversal is possible (paths are opened directly via the OS API).
