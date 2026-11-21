# Extra MCPs

MCP servers from other programs that the person registers for the agent **of one vault**, in Settings › AI and agent › Extra MCPs. Example: an MCP that controls the phone, turned on only in the personal vault.

## What lives where

- The configuration (label, name, command and arguments or URL, names of the variables and headers, switches) lives in the `ai_mcp_server` table of the vault's encrypted database (migration V154). Another vault does not see it or run it.
- The secret values of variables and headers are stored encrypted in the `secret` table (XChaCha20-Poly1305 with a subkey of the database key, like the engine keys), with ids `mcp_extra:<id>:env:<CHAVE>` and `mcp_extra:<id>:header:<nome>`. The interface only sees whether they are registered and the last 4 characters; the values never appear in API responses, logs, errors or the audit. They go through the re-encryption on a password change like the other secrets.
- Only the app's interface registers, tests, deletes and confirms (`ai.mcp_extra.*` refuses the agent and outside MCP clients).

## How the agent uses them

- **API engines** (including local models through an endpoint): on each response, Treasury connects to the servers that are turned on (child process over stdio or Streamable HTTP), offers their tools to the model as `<nome>-<ferramenta>` (up to 40 per server; no `__` in the name, which Grok cannot call) and adds one line per server to the instructions. The lean mode (Groq) has no extra MCPs.
- **Confirmation**: Treasury does not know whether an external tool only reads or also acts. Before each call, the conversation shows a card with the server, the tool and the summarized parameters, and the agent waits (up to 5 minutes; canceling the conversation declines). The card has "Allow", "Allow in this conversation" (the same server goes through until the end of the conversation; the permission stays only in memory and disappears when the vault locks) and "Decline". Per server, "Allow without asking" skips the card in all conversations. Allowed or not, each call goes to the audit.
- **CLI engines** (Claude Code, Codex, Gemini CLI, Grok, OpenCode and custom ones): at the start of the conversation, Treasury opens the sessions with the servers that are turned on and Treasury's own MCP offers their tools to the CLI (the job's token), with the exact name of each in the instructions (`treasury__<nome>-<ferramenta>` in Grok, `mcp__treasury__<nome>-<ferramenta>` in Claude Code), because in Grok tool search is blocked together with file reading. The call goes through the app, with the same confirmation card and the same audit; no secret value goes into the CLI's configuration files. The sessions close when the response ends.
- The result of an external tool goes to the model marked as data, not instruction, capped at 20 thousand characters (images and audio become a notice).
- Each call goes to the agent's audit (Agent › Audit screen) with the client `agente → <rótulo>`, the tool, the summarized parameters, the result and the duration; declines appear as `declined`.
- Treasury's MCP server only offers the extra MCPs to the agent token of a conversation; outside MCP clients never see them.

## MCPs you already have in the CLIs

The CLIs load MCPs on their own: Grok imports those from Claude (`~/.claude.json`), Cursor and plugins; Codex, Gemini and OpenCode have those from their own configuration. By default, **none** enters the agent's conversations: only Treasury and the vault's extra MCPs. In Settings › AI and agent › MCPs from your CLIs, the person allows them one by one (vault setting `ai.cli_user_mcps`, list in `ai.cli_mcps.list`).

- Grok: those not allowed are turned off in the configuration of the conversation folder (`[mcp_servers.<nome>] enabled = false`, which overrides the user's, Claude's, Cursor's and the plugins': the process does not even start) and denied (`--deny MCPTool(<nome>__*)`); those allowed get `--allow`.
- Claude Code (`--strict-mcp-config`): those allowed are copied from `~/.claude.json` into the conversation's MCP file and go into `--allowedTools`.
- Gemini: `--allowed-mcp-server-names` with Treasury and the allowed ones, even with the template edited.
- Codex and OpenCode: those not allowed are turned off in this run.
- An allowed MCP is called by the CLI directly, without Treasury's confirmation or audit; the agent's instructions say so.

## Limitations

- At most 20 servers per vault. HTTP URL only with `https://` (or `http://localhost` for a local server).
- A local command runs on this computer with your user's permissions: register only servers you trust.
- No OAuth login on an HTTP server (only a fixed token through a header), and no MCP resources and prompts (only tools).
