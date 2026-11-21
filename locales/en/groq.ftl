# Groq as the agent's free engine (docs/ai-agent.md › Groq): the key guide when adding the
# engine, the shortcut when there is no engine and the per-minute limit wait notice.
# Sources checked on 2026-10-20. Revision: AI draft, not yet reviewed by a native speaker.

groq-usar-gratuito = Use Groq (free plan)
groq-sem-motor-dica = No cost: Groq has a free plan. In Settings › AI and agent, tap "Use Groq" to see how to get the key.
groq-guia-titulo = How to get a free Groq key
groq-guia-passo-1 = Open console.groq.com and create an account (with email, Google or GitHub).
groq-guia-passo-2 = In the menu, open "API Keys" and tap "Create API Key". Give it a name (e.g. Treasury) and copy the key, which starts with "gsk_". It is shown only once.
groq-guia-passo-3 = Paste the key in the "API key" field below and save. Treasury keeps the key encrypted in the vault.
groq-guia-abrir = Open the Groq console
groq-guia-limites-titulo = Free plan limits
groq-guia-limites = The free plan limits requests and tokens per minute and per day, per model. Your limits are shown at console.groq.com/settings/limits. When the per-minute limit runs out, Treasury waits and continues.
groq-guia-enxuto-titulo = Lean agent
groq-guia-enxuto = To fit the per-minute limit, the agent with Groq uses short instructions and only the everyday tools: overview, accounts, transactions, spending, budget, goals, income, categorizing and recording. Importing statements and investments and the rest need a full engine.
groq-guia-privacidade-titulo = Privacy (checked on { $data })
groq-guia-privacidade = The data the agent looks up (transactions, balances, names) leaves your computer and goes to Groq. According to Groq's data page, it does not keep requests by default, but it may keep logs for up to 30 days to troubleshoot errors or investigate abuse. You can turn on Zero Data Retention in Data Controls, in your Groq account settings. The data is stored on servers in the United States.
groq-guia-fonte = Groq's data policy
groq-aguardando-limite = { $motor }: free plan limit reached; waiting { $segundos } s to continue.
