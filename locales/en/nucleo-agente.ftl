# Export an agent conversation (Markdown and PDF).
nucleo-agente-exportar-voce = You
nucleo-agente-exportar-agente = Agent
nucleo-agente-exportar-agente-motor = Agent ({ $motor })
nucleo-agente-exportar-anexos = Attachments: { $nomes }
nucleo-agente-exportar-ferramentas = Tools used:
nucleo-agente-exportar-falhou = failed
nucleo-agente-exportar-propostas = { $n ->
    [one] 1 proposal created (approval in 13 · Agent)
   *[other] { $n } proposals created (approval in 13 · Agent)
}
nucleo-agente-exportar-iniciada = Conversation started on { $data }
nucleo-agente-exportar-mensagens = { $n ->
    [one] 1 message
   *[other] { $n } messages
}
nucleo-agente-exportar-gerada = Exported from Treasury on { $data }
nucleo-agente-exportar-aviso = AI-generated answers may contain errors and are not investment advice. Keys and passwords were removed from this file.
nucleo-agente-exportar-pagina = Treasury · page { $n } of { $total }
nucleo-agente-retirar-so-agente = Only the agent withdraws its own proposals; in the app, reject it in the queue.
nucleo-agente-retirar-de-outro = This proposal belongs to another client: you can only withdraw your own.
nucleo-agente-retirar-decidida = This proposal is no longer pending.
nucleo-agente-retirada-pelo-agente = Withdrawn by the agent. { $motivo }
