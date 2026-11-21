# Términos de uso (docs/terms.md): la pantalla "Antes de empezar", la tarjeta en Ajustes y los
# avisos de las pantallas. El texto completo está en conteudo/<idioma>/termos.md.
# Revisión: borrador por IA, sin revisión nativa.

termos-titulo = Antes de empezar
termos-intro = Treasury es un programa **gratuito y de código abierto**, hecho por una sola persona, con la ayuda de quien quiera contribuir. Organiza las finanzas de tu familia **en tu computadora**. Lee estos puntos con calma:
termos-ponto-dados = Tus datos se quedan contigo.
termos-ponto-dados-texto = Todo se guarda cifrado en esta computadora. Treasury no me envía nada a mí ni a nadie, y no tiene rastreo.
termos-ponto-senha = Si olvidas la contraseña de la bóveda, los datos no se pueden recuperar.
termos-ponto-senha-texto = Ni siquiera yo puedo abrirla. Guarda la contraseña en un lugar seguro y haz copias de seguridad.
termos-ponto-impostos = Los impuestos son solo estimaciones.
termos-ponto-impostos-texto = Treasury no reemplaza a la autoridad fiscal de tu país, a los programas oficiales de declaración ni a un contador. Consulta siempre los canales oficiales del gobierno antes de declarar o pagar.
termos-ponto-investimentos = Inversiones: no es una recomendación.
termos-ponto-investimentos-texto = Notas, cartera, simulaciones y "¿y si…?" son herramientas de estudio. No son consejos para comprar o vender nada. Las decisiones de inversión son tuyas.
termos-ponto-erros = Puede haber errores.
termos-ponto-erros-texto = Los datos de fuentes externas pueden retrasarse o venir equivocados, las importaciones pueden leer algo mal y la inteligencia artificial puede equivocarse. Verifica los números importantes.
termos-ponto-nuvem = Si usas IA en la nube,
termos-ponto-nuvem-texto = lo que le envíes va a la empresa que elegiste (por ejemplo, Anthropic, OpenAI o Google), según sus reglas.
termos-ponto-risco = Uso bajo tu propio riesgo.
termos-ponto-risco-texto = El programa se ofrece tal cual, sin garantía. Me esfuerzo para que funcione bien, pero no me hago responsable de pérdidas causadas por su uso, por información equivocada ni por mal uso.
termos-ler-completos = Leer los términos completos
termos-ler-licenca = Leer la licencia AGPL
termos-marcar = Leí y entendí. Sé que Treasury no es una herramienta oficial de impuestos, no hace recomendaciones de inversión y que uso el programa bajo mi propio riesgo.
termos-sair = Salir
termos-o-que-mudou = Qué cambió

## Aviso de versión nueva y tarjeta en Ajustes

termos-de-uso = Términos de uso
termos-atualizados = Los términos de uso se actualizaron (versión { $versao }).
termos-ler = Leer
termos-entendi = Entendido
termos-versao = Versión { $versao }. La aceptación se guarda solo en esta computadora; no se envía nada.
termos-versao-aceita = Versión { $versao } · aceptada en esta computadora el { $data } (versión { $aceita }). La aceptación se guarda solo en esta computadora; no se envía nada.
termos-historico = Historial

## Avisos de las pantallas ($pais: BR, US, PT u outro)

termos-aviso-estudo = Herramienta de estudio. No es una recomendación de inversión.
termos-aviso-imposto = Estimación basada en { $fonte } de { $data }. No reemplaza las herramientas oficiales { $pais ->
        [BR] de la Receita Federal de Brasil
        [US] del IRS
        [PT] de la Autoridad Tributaria y Aduanera de Portugal
       *[outro] de la autoridad fiscal de tu país
    }.
# Divergencias conocidas de la estimación (docs/tax-validation-countries.md), junto al aviso.
termos-aviso-imposto-limites-us = Todavía no considera: la parte reembolsable del crédito por hijo y el EITC, ni el impuesto estatal. El estado civil sale del registro de la familia, como en el formulario W-4: sin cónyuge y con dependientes, cabeza de familia (head of household); con cónyuge sin ingresos registrados, declaración conjunta; con ingresos de los dos, cada uno por la tabla individual.
termos-aviso-imposto-limites-pt = Todavía no considera: el aumento por dependiente de hasta 6 años, el IRS Jovem ni la tributación conjunta. Cuenta con la deducción de gastos generales familiares en su límite legal (250 €, o 335 € en la familia monoparental), como la propia regla del mínimo de existencia.
