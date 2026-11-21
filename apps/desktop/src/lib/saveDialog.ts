/**
 * Diálogo "Salvar como" do sistema (plugin de diálogo do Tauri). No app, link de Blob com
 * `download` não baixa nada (o WKWebView ignora): quem salva pergunta o caminho aqui e pede
 * ao núcleo para gravar o arquivo. Fora do app (navegador, modo de demonstração) não há
 * diálogo: use `isTauri()` antes e caia no download do navegador.
 */
export async function chooseSavePath(
  title: string,
  fileName: string,
  filters?: { name: string; extensions: string[] }[],
): Promise<string | null> {
  const { invoke } = await import('@tauri-apps/api/core');
  const path = await invoke<string | null>('plugin:dialog|save', {
    options: { title, defaultPath: fileName, ...(filters ? { filters } : {}) },
  });
  return typeof path === 'string' ? path : null;
}

/** Extensão do arquivo (sem o ponto), para o filtro do diálogo. */
export function extensionOf(fileName: string): string | null {
  const i = fileName.lastIndexOf('.');
  return i > 0 && i < fileName.length - 1 ? fileName.slice(i + 1).toLowerCase() : null;
}
