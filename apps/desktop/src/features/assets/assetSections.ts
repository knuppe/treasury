/**
 * Que abas e botões a página do ativo (e o menu de contexto dele) mostra conforme o tipo:
 *
 * | Tipo                                           | Proventos | Eventos corporativos |
 * | ---------------------------------------------- | --------- | -------------------- |
 * | Ações, FIIs, ETFs, BDRs, REITs (BR, EUA, mundo) | sim       | sim                  |
 * | Fundos de investimento                          | sim       | sim (incorporação)   |
 * | Renda fixa e Tesouro Direto                     | sim (cupom, juros, amortização) | não |
 * | Previdência                                     | não (o rendimento fica na cota) | não |
 * | Cripto                                          | não       | não                  |
 * | Outro                                           | sim       | sim                  |
 *
 * Fundamentos seguem `hasFundamentals`/Fundamentus e a aba Previdência só aparece em
 * previdência. Resumo, Extrato, Operações, Gráfico, Saúde, Estudo e Cadastro valem para
 * todos. Se o ativo já tem proventos ou eventos registrados, a aba aparece mesmo assim,
 * para nenhum dado ficar escondido.
 */
import type { AssetClass } from '@/api/generated/AssetClass';

const WITHOUT_INCOME: readonly AssetClass[] = ['pension', 'crypto'];
const WITHOUT_ACTIONS: readonly AssetClass[] = ['pension', 'crypto', 'fixed_income', 'treasury_br'];

export interface AssetSections {
  /** Aba Proventos, botão Provento e "Registrar provento" no menu. */
  income: boolean;
  /** Aba Eventos corporativos, botão Evento e "Evento corporativo" no menu. */
  actions: boolean;
}

export function assetSections(
  cls: AssetClass,
  recorded: { income?: boolean; actions?: boolean } = {},
): AssetSections {
  return {
    income: !WITHOUT_INCOME.includes(cls) || Boolean(recorded.income),
    actions: !WITHOUT_ACTIONS.includes(cls) || Boolean(recorded.actions),
  };
}
