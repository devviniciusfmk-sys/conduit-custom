# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

We take security vulnerabilities seriously. If you discover a security issue, please report it responsibly.

### How to Report

**Do NOT open a public GitHub issue for security vulnerabilities.**

Instead, please report security issues by emailing the maintainers directly or using GitHub's private vulnerability reporting feature.

### What to Include

When reporting a vulnerability, please include:

1. **Description** - A clear description of the vulnerability
2. **Steps to reproduce** - Detailed steps to reproduce the issue
3. **Impact** - What an attacker could achieve by exploiting this
4. **Affected versions** - Which versions are affected
5. **Suggested fix** - If you have ideas on how to fix it (optional)

### Response Timeline

- **Initial response**: Within 48 hours
- **Status update**: Within 7 days
- **Resolution target**: Depends on severity

### What to Expect

1. We will acknowledge receipt of your report
2. We will investigate and validate the vulnerability
3. We will work on a fix and coordinate disclosure timing with you
4. We will credit you in the release notes (unless you prefer to remain anonymous)

## Security Best Practices

When using Conduit:

- Keep Conduit and its dependencies up to date
- Review agent outputs before executing suggested commands
- Be cautious with agents that have shell access
- Don't share session files that may contain sensitive information
- Use environment variables for API keys rather than hardcoding them

## Scope

This security policy covers:

- The Conduit TUI application
- The Conduit website (getconduit.sh)
- Official documentation

Third-party integrations (Claude Code, Codex CLI) have their own security policies.

Thank you for helping keep Conduit secure!
