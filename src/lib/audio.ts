import { readFile } from "@tauri-apps/plugin-fs";

const blobUrlCache = new Map<string, string>();

/**
 * Convert a local file path to a blob URL that the webview can play.
 * Results are cached so the same file isn't read twice.
 */
export async function getAudioUrl(filePath: string): Promise<string> {
  const cached = blobUrlCache.get(filePath);
  if (cached) return cached;

  const bytes = await readFile(filePath);
  const blob = new Blob([bytes], { type: "audio/wav" });
  const url = URL.createObjectURL(blob);
  blobUrlCache.set(filePath, url);
  return url;
}
