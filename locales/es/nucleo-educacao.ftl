# Meta de universidad de los hijos (núcleo: `goals::education` y `goals::holdings`;
# docs/design/college-goal.md): errores, notas del cálculo y motivos del historial.

nucleo-educacao-plano = plan de educación de la meta { $id }
nucleo-educacao-conta = cuenta { $id }
nucleo-educacao-ativo = activo { $id }
nucleo-educacao-erro-sem-filho = elige al hijo o indica la fecha prevista de nacimiento
nucleo-educacao-erro-sem-nascimento = registra la fecha de nacimiento del hijo (Familia) o indica la fecha de inicio de la carrera
nucleo-educacao-erro-inicio = la edad o el mes de inicio de la carrera no es válido
nucleo-educacao-erro-duracao = la duración de la carrera va de 1 a 180 meses
nucleo-educacao-erro-parcelas = las cuotas por año son 1, 2, 4 o 12
nucleo-educacao-erro-valor-negativo = la mensualidad, los extras y el costo de vida no pueden ser negativos
nucleo-educacao-erro-desconto = el descuento va de 0% a 100%
nucleo-educacao-erro-retorno = el retorno real supuesto va de −5% a 15% al año
nucleo-educacao-erro-diferenca = indica la diferencia sobre la inflación, de −10% a 20% al año
nucleo-educacao-erro-serie = la serie de inflación de la educación tiene que ser una serie mensual del catálogo
nucleo-educacao-erro-posicao-repetida = la misma cuenta (o el mismo activo en la cuenta) aparece dos veces
nucleo-educacao-erro-fracao = la fracción va de 1% a 100%
nucleo-educacao-erro-fracao-soma = las fracciones de { $nome } entre las metas pasarían de 100% (libre: { $livre })
nucleo-educacao-nota-sem-serie = No hay años completos de la serie de inflación de la educación: el cálculo usa solo la inflación general. Sincroniza los índices o indica una diferencia fija.
nucleo-educacao-nota-inflacao-suposta = No hay serie de inflación general con años completos: los valores en dinero de la época suponen { $taxa } al año.
nucleo-educacao-nota-publica = En una universidad pública, el costo suele ser el de vida (vivienda, transporte, comida en otra ciudad). Conviene llenar el costo de vida por mes.
nucleo-educacao-nota-exterior = Carrera en el exterior: el cálculo usa la moneda de la meta y no incluye la variación del tipo de cambio, que es un riesgo aparte.
nucleo-educacao-nota-sem-mensalidade = Falta la mensualidad de hoy: pide una cotización a la universidad e indícala en el plan.
nucleo-educacao-nota-terminou = La carrera terminó. La meta se puede archivar, y lo que sobró queda libre.
nucleo-educacao-meta-nome = Universidad de { $nome }
nucleo-educacao-meta-nome-previsto = Universidad (nacimiento previsto el { $data })
nucleo-educacao-meta-nome-sem-filho = Universidad
nucleo-educacao-motivo-meta-criada = Meta de universidad creada
nucleo-educacao-motivo-posicoes = Posiciones vinculadas a la meta: { $nome }
