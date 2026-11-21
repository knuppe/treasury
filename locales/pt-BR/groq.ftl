# Groq como motor gratuito do agente (docs/ai-agent.md › Groq): o guia da chave no cadastro
# do motor, o atalho quando não há motor e o aviso de espera do limite por minuto.
# Fontes conferidas em 20/10/2026: console.groq.com/docs/your-data, /docs/rate-limits,
# /docs/tool-use, /docs/models. pt-BR é a origem.

groq-usar-gratuito = Usar o Groq (plano gratuito)
groq-sem-motor-dica = Sem custo: o Groq tem plano gratuito. Em Ajustes › IA e agente, toque em "Usar o Groq" para ver como pegar a chave.
groq-guia-titulo = Como pegar a chave gratuita do Groq
groq-guia-passo-1 = Abra console.groq.com e crie uma conta (com e-mail, Google ou GitHub).
groq-guia-passo-2 = No menu, abra "API Keys" e toque em "Create API Key". Dê um nome (ex.: Treasury) e copie a chave, que começa com "gsk_". Ela aparece uma vez só.
groq-guia-passo-3 = Cole a chave no campo "API key" abaixo e cadastre. O Treasury guarda a chave cifrada no cofre.
groq-guia-abrir = Abrir o console do Groq
groq-guia-limites-titulo = Limites do plano gratuito
groq-guia-limites = O plano gratuito limita pedidos e tokens por minuto e por dia, por modelo. Os seus limites aparecem em console.groq.com/settings/limits. Quando o limite do minuto acaba, o Treasury espera e continua.
groq-guia-enxuto-titulo = Agente enxuto
groq-guia-enxuto = Para caber no limite por minuto, o agente com o Groq usa instruções curtas e só as ferramentas do dia a dia: resumo, contas, lançamentos, gastos, orçamento, metas, renda, categorizar e lançar. Importar extratos e investimentos e o resto pedem um motor completo.
groq-guia-privacidade-titulo = Privacidade (conferido em { $data })
groq-guia-privacidade = Os dados que o agente consulta (lançamentos, saldos, nomes) saem do computador e vão para o Groq. Pela página de dados do Groq, ele não guarda os pedidos por padrão, mas pode guardar registros por até 30 dias para resolver erros ou investigar abuso. Dá para ligar a retenção zero (Zero Data Retention) em Data Controls, nas configurações da conta do Groq. Os dados ficam em servidores nos Estados Unidos.
groq-guia-fonte = Política de dados do Groq
groq-aguardando-limite = { $motor }: limite do plano gratuito atingido; esperando { $segundos } s para continuar.
