# Reglas de categorización (pestaña Reglas de 04 · Movimientos): lista, diálogo de la regla,
# prueba y aplicación a los movimientos existentes (docs/i18n-design.md).
# Revisión: borrador por IA, sin revisión de un hablante nativo.

## Aplicar reglas

regras-n-alterados = { $n ->
        [one] { $n } movimiento modificado por las reglas
       *[other] { $n } movimientos modificados por las reglas
    }
regras-aplicar = Aplicar reglas
regras-aplicar-descricao = Primero mira lo que cambiaría; no se guarda nada hasta que confirmes. Todo queda en un solo elemento del historial.
regras-aplicacao-desfeita = Aplicación deshecha
regras-desfazer = Deshacer
regras-ver-previa = Ver vista previa
regras-aplicar-a = Aplicar a { $n }
regras-so-sem-categoria = Solo movimientos sin categoría
regras-so-sem-categoria-descricao = Si lo desactivas, las reglas pueden cambiar categorías ya elegidas.
regras-so-filtro = Solo los movimientos del filtro actual
regras-afetados = Movimientos afectados
regras-resultado-aplicado = Aplicado: { $casaram } coinciden · { $mudam } cambian
regras-resultado-previa = Vista previa: { $casaram } coinciden · { $mudam } cambian
regras-nada-mudar = Nada que cambiar.

## Lista de reglas

regras-titulo = Reglas de categorización
regras-busca = regla,automático
regras-tabela = Reglas
regras-col-prioridade = Prior.
regras-col-regra = Regla
regras-entao = Entonces
regras-col-acertos = Aciertos
regras-acoes = Acciones
regras-acao-categoria = categoría
regras-acao-tipo = tipo { $tipo }
regras-acao-renomear = renombrar a “{ $nome }”
regras-acao-membro = miembro
regras-ultimo = Último: { $data }
regras-erro-alterar = No se pudo cambiar
regras-desativar = Desactivar
regras-ativar = Activar
regras-editar = Editar { $nome }
regras-apagar-item = Eliminar { $nome }
regras-aplicar-reticencias = Aplicar reglas…
regras-nova = Nueva regla
regras-rodape-ativas = Activas
regras-rodape-ordem = Orden
regras-rodape-ordem-valor = menor prioridad primero; la primera que define un campo gana
regras-vazio-kicker = Ninguna regla
regras-vazio-titulo = Enseña a Treasury a categorizar
regras-vazio-descricao = Las reglas completan la categoría, el miembro y las etiquetas al importar. Crea una desde cero o a partir de un movimiento (abre el movimiento y usa “Crear regla a partir de este”).
regras-apagar = Eliminar regla
regras-apagar-descricao = La regla “{ $nome }” deja de valer en las próximas importaciones. Los movimientos ya categorizados no cambian.
regras-apagada = Regla eliminada

## Resumen de las condiciones (`Descripción contiene “uber” y Monto entre 10,00 y 20,00`)

regras-condicao = { $campo } { $op } { $valor }
regras-condicao-entre = { $campo } { $op } { $valor } y { $valor2 }
regras-conta-escolhida = cuenta elegida
regras-e = y
regras-ou = o

## Diálogo de la regla

regras-campo-condicao = Campo de la condición { $n }
regras-operacao-condicao = Operador de la condición { $n }
regras-valor-condicao = Monto de la condición { $n }
regras-segundo-valor-condicao = Segundo monto de la condición { $n }
regras-conta-condicao = Cuenta de la condición { $n }
regras-tipo-condicao = Tipo de la condición { $n }
regras-texto-condicao = Texto de la condición { $n }
regras-tirar-condicao = Quitar la condición { $n }
regras-erro-nome = Ponle un nombre a la regla.
regras-atualizada = Regla actualizada
regras-criada = Regla creada
regras-salva-descricao = Vale para las próximas importaciones; aplícala para categorizar lo que ya existe.
regras-editar-titulo = Editar regla
regras-testar = Probar
regras-salvar = Guardar regla
regras-criar = Crear regla
regras-nome = Nombre
regras-nome-exemplo = Viajes en aplicaciones de transporte
regras-prioridade = Prioridad
regras-prioridade-dica = Gana la menor.
regras-quando = Cuando
regras-combinar = Combinar condiciones
regras-todas = Todas las condiciones
regras-qualquer = Cualquiera
regras-adicionar-condicao = Agregar condición
regras-condicoes-nota = Los textos se comparan sin mayúsculas y sin acentos. Los montos tienen signo: los gastos son negativos.
regras-categoria = Categoría
regras-membro = Miembro
regras-nao-mudar = No cambiar
regras-renomear = Renombrar a
regras-renomear-dica = Opcional: la descripción original queda en las notas.
regras-mudar-tipo = Cambiar el tipo a
regras-tags = Etiquetas
regras-resultado-teste = Resultado de la prueba
regras-teste-nenhum = Ningún movimiento existente coincide con esta regla
regras-teste-n = { $n ->
        [one] { $n } movimiento coincide con esta regla
       *[other] { $n } movimientos coinciden con esta regla
    }
regras-teste-50 = 50 o más movimientos coinciden con esta regla
regras-sem-categoria = sin categoría

## Reglas sugeridas (rules.learned, docs/learned-rules.md)

regras-sugeridas-titulo = Reglas sugeridas
regras-sugeridas-explica = Ya le diste la misma categoría tres veces a estos comercios. Con la regla, los próximos movimientos entran con la categoría. No se crea nada sin que lo pidas.
regras-sugeridas-ja = { $n } con esa categoría
regras-sugeridas-faltam = { $n ->
    [one] 1 sin categoría
   *[other] { $n } sin categoría
}
regras-sugeridas-criar = Crear regla
regras-sugeridas-ignorar = No sugerir
regras-sugeridas-criada = Regla creada
regras-sugeridas-aplicada = { $n ->
    [0] Los próximos movimientos entran con la categoría.
    [one] 1 movimiento recibió la categoría.
   *[other] { $n } movimientos recibieron la categoría.
}
regras-sugeridas-erro = No se pudo crear la regla
