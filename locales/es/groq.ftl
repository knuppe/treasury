# Groq como motor gratuito del agente (docs/ai-agent.md › Groq): la guía de la clave al
# registrar el motor, el atajo cuando no hay motor y el aviso de espera del límite por minuto.
# Fuentes revisadas el 20/10/2026. Revisión: borrador por IA, aún sin revisar por un hablante nativo.

groq-usar-gratuito = Usar Groq (plan gratuito)
groq-sem-motor-dica = Sin costo: Groq tiene un plan gratuito. En Ajustes › IA y agente, toca "Usar Groq" para ver cómo obtener la clave.
groq-guia-titulo = Cómo obtener la clave gratuita de Groq
groq-guia-passo-1 = Abre console.groq.com y crea una cuenta (con correo, Google o GitHub).
groq-guia-passo-2 = En el menú, abre "API Keys" y toca "Create API Key". Ponle un nombre (p. ej., Treasury) y copia la clave, que empieza con "gsk_". Se muestra una sola vez.
groq-guia-passo-3 = Pega la clave en el campo "Clave de API" de abajo y guarda. Treasury guarda la clave cifrada en la bóveda.
groq-guia-abrir = Abrir la consola de Groq
groq-guia-limites-titulo = Límites del plan gratuito
groq-guia-limites = El plan gratuito limita las solicitudes y los tokens por minuto y por día, por modelo. Tus límites aparecen en console.groq.com/settings/limits. Cuando se acaba el límite del minuto, Treasury espera y continúa.
groq-guia-enxuto-titulo = Agente reducido
groq-guia-enxuto = Para caber en el límite por minuto, el agente con Groq usa instrucciones cortas y solo las herramientas del día a día: resumen, cuentas, movimientos, gastos, presupuesto, metas, ingresos, categorizar y registrar. Importar extractos e inversiones y lo demás requieren un motor completo.
groq-guia-privacidade-titulo = Privacidad (revisado el { $data })
groq-guia-privacidade = Los datos que consulta el agente (movimientos, saldos, nombres) salen de tu computadora y van a Groq. Según la página de datos de Groq, no guarda las solicitudes por defecto, pero puede guardar registros hasta 30 días para resolver errores o investigar abusos. Puedes activar la retención cero (Zero Data Retention) en Data Controls, en la configuración de tu cuenta de Groq. Los datos se almacenan en servidores en Estados Unidos.
groq-guia-fonte = Política de datos de Groq
groq-aguardando-limite = { $motor }: se alcanzó el límite del plan gratuito; esperando { $segundos } s para continuar.
