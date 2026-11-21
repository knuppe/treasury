# Score de saúde por ativo: nota, faixas, dimensões, métricas, red flags, eventos, histórico e
# avaliação (docs/i18n-design.md). As explicações e notas que vêm do núcleo não passam por aqui.
# pt-BR é o idioma de origem.

## Faixas, situação e severidade

score-faixa-robusto = Robusto
score-faixa-saudavel = Saudável
score-faixa-atencao = Atenção
score-faixa-risco = Risco
score-dados-insuficientes = Dados insuficientes
score-status-calculado = Calculado
score-status-parcial = Parcial
score-status-nao-suportada = Classe não suportada
score-severidade-informativa = Informativa
score-severidade-baixa = Baixa
score-severidade-media = Média
score-severidade-alta = Alta
score-severidade-critica = Crítica

## Valores das métricas

score-dias = { $n ->
    [1] { $valor } dia
   *[other] { $valor } dias
}
score-anos = { $n ->
    [1] { $valor } ano
   *[other] { $valor } anos
}
score-percentil-pares = { $n ->
    [1] { $p } de { $n } par
   *[other] { $p } de { $n } pares
}
score-teto = teto { $nota }
score-penalidade = { $n ->
   *[other] { $pontos } pontos
}

## Peças: selo, barra e régua

score-chip-parcial = {" "}· parcial
score-barra-sem-nota = { $rotulo }: sem nota
score-barra-nota = { $rotulo }: { $nota } de 100
score-faixas-regua = Faixas de 0 a 100: { $faixas }

## Aba Saúde: cabeçalho

score-segmento = Segmento
score-segmento-metodologia = Segmento da metodologia
score-segmento-automatico = Segmento automático
score-segmento-trocado = Segmento trocado
score-segmento-refeito = { $ticker }: o score foi refeito com os pesos do novo segmento.
score-segmento-auto-setor = Automático (pelo setor)
score-escolhido-por-voce = Escolhido por você ·{" "}
score-titulo = Score de saúde do ativo
score-metodologia = Metodologia
score-avaliado-em = Avaliado em
score-dados-de = Dados de
score-defasado = defasado
score-defasado-dica = Os dados mais recentes têm mais de 2 trimestres
score-nota = Score
score-nota-parcial = Score (parcial)
score-pontos = { $delta } pontos
score-mudanca = { $pontos } desde { $data } (era { $nota })
score-mudanca-faixa = { $pontos } desde { $data } (era { $nota }, { $faixa })
score-confianca = Confiança
score-saude = Saúde
score-saude-dica = sem valuation
score-atratividade = Atratividade
score-atratividade-dica = preço
score-pares = Pares
score-pares-setor = { $setor } no setor { $nome } · { $segmento } no segmento · { $classe } na classe
score-pares-resumo = { $setor } no setor · { $segmento } no segmento · { $classe } na classe
score-alertas = Alertas do score

## Aba Saúde: dado crítico ausente e motivos

score-nao-calculado = Score não calculado: falta um dado crítico
score-criticos-ausentes = Dados críticos ausentes
score-critica = crítica
score-tambem-sem-dado = Também sem dado: { $lista }.
score-sem-metricas = Sem essas métricas a nota não é calculada (nada é inventado). O caso foi registrado para o desenvolvedor, em 15 · Desenvolvedor, para revisão da coleta. As dimensões abaixo mostram o que já foi medido.
score-por-que = Por que essa nota
score-a-favor = A favor
score-contra = Contra
score-red-flags = Red flags
score-gravidade = gravidade { $nivel }
score-antes-flags = Antes das red flags o score era { $nota }.
score-sem-dado-redistribuido = Sem dado (o peso foi redistribuído na dimensão): { $lista }.

## Aba Saúde: modelos, conferência e avisos

score-piotroski = { $valor } de { $total }
score-itens = Itens · { $modelo }
score-atende = atende
score-nao-atende = não atende
score-metrica = Métrica
score-fonte = Fonte
score-diferenca = Diferença
score-conferido-com = Conferido com
score-calculando = Calculando o score
score-notas-calculo = Notas do cálculo
score-cripto-fontes = Fontes dos dados de criptoativos
score-cripto-fontes-texto = Dados fornecidos por:
score-indisponivel = Score de saúde ainda não disponível para { $classe }
score-cripto-titulo = Robustez dentro da classe
score-cripto-texto = Cripto não se compara diretamente com ações: o score mede a robustez dentro da classe de criptoativos.
score-parcial-titulo = Score parcial: sem faixa definitiva
score-parcial-texto = { $pct }% do peso veio de dimensões imputadas (mediana dos pares × 0,9) por falta de dados. O número é mostrado, mas a faixa fica como “Dados insuficientes” até a cobertura melhorar.
score-dimensoes-titulo = Dimensões e métricas
score-dimensoes-ajuda = Clique numa dimensão para ver as métricas. A Saúde soma as dimensões sem o selo “atratividade”; a Atratividade, só as de preço. Percentis já estão na direção “maior é melhor”.
score-modelos = Modelos clássicos
score-conferencia = Conferência com outras fontes
score-conferencia-ajuda = Só validação cruzada: o score usa os números calculados pelo Treasury com as demonstrações da CVM. Os proventos são conferidos com a B3 (oficial); se a agenda diverge, o DY usa a B3. Diferenças acima de 10% ficam em destaque.

## Dimensões e métricas

score-fonte-de = Fonte: { $fonte }
score-critica-dica = Sem esta métrica não há score
score-sem-dado = Sem dado: { $motivo }
score-valor = Valor
score-col-nota = Nota
score-pior = {" "}(pior)
score-entre-pares = Entre os pares
score-absoluta = Absoluta
score-peso = Peso
score-periodo = Período
score-imputada = imputada
score-imputada-dica = Cobertura abaixo do mínimo: nota da mediana dos pares × 0,9
score-selo-atratividade = atratividade
score-selo-atratividade-dica = Entra no sub-score Atratividade (preço), não na Saúde
score-peso-cobertura = peso { $peso } · cobertura { $cobertura }
score-metricas-de = Métricas · { $dimensao }
score-dimensoes = Dimensões do score

## Histórico do score

score-data = Data
score-faixa = Faixa
score-historico = Histórico do score
score-historico-fonte = Snapshots de { $de } a { $ate }
score-historico-aria = Score de { $primeira } em { $de } a { $ultima } em { $ate }

## Sua opinião ("Discordo")

score-discordo = Discordo
score-concordo = Concordo
score-sem-opiniao = Não sei / sem opinião
score-opiniao-registrada = Opinião registrada
score-opiniao-registrada-dica = Ela entra na revisão da metodologia do score.
score-opiniao-form = Opinião sobre o score
score-veredito = Veredito
score-faixa-esperada = Faixa que você esperaria
score-opcional = Opcional.
score-motivo = Motivo
score-motivo-dica = O que a nota deixou de ver ou pesou demais (ex.: dívida já renegociada, lucro não recorrente).
score-motivo-erro = Conte o motivo.
score-registrar-opiniao = Registrar opinião
score-sua-opiniao-sobre = Sua opinião sobre o score
score-sua-opiniao = Sua opinião
score-sua-opiniao-dica = A nota não bate com o que você sabe da empresa? Registre; os casos entram na calibração da metodologia.
score-opinioes-registradas = Opiniões registradas
score-opiniao-nota = · nota { $nota } ({ $faixa }) · metodologia { $versao }
score-esperava = · esperava { $faixa }
score-discordo-titulo = Discordo do score · { $nome }
score-discordo-descricao = Nota { $nota } ({ $faixa }) na metodologia { $versao }. Sua opinião fica registrada com a nota do dia.

## E-mail de contato da SEC

score-sec-baixadas = Demonstrações da SEC baixadas
score-sec-texto = A SEC (órgão regulador dos EUA) só entrega dados a quem se identifica com um e-mail de contato. Ele vai só no cabeçalho dos pedidos à SEC, fica no cofre e pode ser trocado em Ajustes › Fontes de dados.
score-sec-email = Seu e-mail de contato para a SEC
score-sec-baixando = Baixando da SEC…
score-sec-salvar = Salvar e baixar

## Eventos qualitativos

score-evento-aguardando = Aguardando você
score-evento-aceito = Aceito (vale na nota)
score-evento-rejeitado = Rejeitado
score-evento-resolvido = Resolvido
score-evidencia = evidência ↗
score-evento-hoje = · hoje: { $efeito }
score-aceitar = Aceitar
score-rejeitar = Rejeitar
score-marcar-resolvido = Marcar como resolvido
score-voltar-valer = Voltar a valer
score-apagar-evento = Apagar o evento { $titulo }
score-evento-aplicado = Evento aplicado ao score
score-casou = · casou “{ $termo }”
score-aplicar = Aplicar ao score
score-evento-registrado = Evento registrado
score-nota-refeita = A nota foi refeita.
score-novo-evento = Novo evento
score-evento-tipo = Tipo
score-evento-tipo-dica = O teto ou a penalidade de cada tipo estão na metodologia.
score-evento-o-que = O que aconteceu
score-evento-descreva = Descreva o evento.
score-informe-data = Informe a data.
score-evidencia-link = Evidência (link)
score-evidencia-dica = Fato relevante, notícia ou documento. Recomendado.
score-observacao = Observação
score-registrar-aplicar = Registrar e aplicar
score-eventos-titulo = Eventos qualitativos
score-eventos-ajuda = Recuperação judicial, fraude, litígios, desastres, governança e regulação. Só os eventos aceitos por você mexem na nota, e o efeito diminui com o tempo até você marcar como resolvido.
score-eventos-registrados = Eventos registrados
score-nenhum-evento = Nenhum evento registrado.
score-sugestoes-cvm = Sugestões dos fatos relevantes da CVM
score-sugestoes = Sugestões de eventos
score-novo-evento-de = Novo evento · { $nome }
score-novo-evento-descricao = O evento entra aceito e a nota é refeita na hora.

## Carteira › Saúde dos ativos

score-classe-nao-suportada = classe não suportada
score-falta-critico = Falta dado crítico
score-parcial-dica = Mais de 30% do peso imputado: sem faixa definitiva
score-ativo = Ativo
score-variacao = Variação
score-flags = Flags
score-calculando-ativos = Calculando o score dos ativos
score-sem-ativos = Sem ativos
score-carteira-vazia = Nenhum ativo na carteira ou na watchlist
score-carteira-vazia-descricao = O score de saúde avalia os fundamentos (rentabilidade, dívida, qualidade dos lucros, valuation…) de cada ação ou FII que você tem ou acompanha. Registre uma operação ou acompanhe um ativo em 09 · Ativos.
score-carteira-ajuda = Score de saúde de 0 a 100 de cada ativo com posição ou na watchlist, pelos fundamentos públicos (CVM, B3). Robusto ≥ 75, Saudável 60–74, Atenção 40–59, Risco abaixo de 40. Não é recomendação de investimento.
score-carteira-tabela = Saúde dos ativos
score-sec-email-exemplo = nome@email.com
