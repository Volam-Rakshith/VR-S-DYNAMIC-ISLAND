export type Theme = 'dark' | 'light' | 'black';
export type Settings = { theme: Theme; accent: string; showSeconds: boolean; animationMs: number };
export const defaults: Settings = { theme: 'dark', accent: '#a4f5c8', showSeconds: false, animationMs: 240 };
export function validateSettings(value: unknown): Settings {
  if (!value || typeof value !== 'object') throw new Error('Settings must be an object');
  const v = value as Record<string, unknown>;
  if (!['dark', 'light', 'black'].includes(String(v.theme)) || typeof v.accent !== 'string' || !/^#[\da-f]{6}$/i.test(v.accent) || typeof v.showSeconds !== 'boolean' || typeof v.animationMs !== 'number' || !Number.isInteger(v.animationMs) || v.animationMs < 0 || v.animationMs > 1000) throw new Error('Invalid settings file');
  return { theme: v.theme as Theme, accent: v.accent, showSeconds: v.showSeconds, animationMs: v.animationMs };
}
