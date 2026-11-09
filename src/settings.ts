export const themes = ['dark','light','black','midnight','ocean','forest','lavender','rose','sunset','slate','sand','mint','contrast','glass','neo'] as const;
export const animations = ['fade','scale','slide-down','slide-up','slide-left','slide-right','bounce','soft-pop','flip','blur','zoom-out','rotate','swing','stretch','none'] as const;
export const positions = ['top-center','top-left','top-right','bottom-center','bottom-left','bottom-right'] as const;
export type Theme = typeof themes[number];
export type Settings = { theme: Theme; accent: string; showSeconds: boolean; animationMs: number; animation: typeof animations[number]; position: typeof positions[number]; compactWidth: number; compactHeight: number };
export const defaults: Settings = { theme: 'dark', accent: '#a4f5c8', showSeconds: false, animationMs: 240, animation:'scale',position:'top-center',compactWidth:260,compactHeight:52 };
export function validateSettings(value: unknown): Settings {
  if (!value || typeof value !== 'object' || Array.isArray(value)) throw new Error('Settings must be an object');
  const v = {...defaults, ...(value as Record<string, unknown>)};
  if (!themes.includes(v.theme as Theme) || typeof v.accent !== 'string' || !/^#[\da-f]{6}$/i.test(v.accent) || typeof v.showSeconds !== 'boolean' || typeof v.animationMs !== 'number' || !Number.isInteger(v.animationMs) || v.animationMs < 0 || v.animationMs > 1000 || !animations.includes(v.animation as Settings['animation']) || !positions.includes(v.position as Settings['position']) || !Number.isInteger(v.compactWidth) || v.compactWidth < 240 || v.compactWidth > 400 || !Number.isInteger(v.compactHeight) || v.compactHeight < 48 || v.compactHeight > 80) throw new Error('Invalid settings file');
  return { theme: v.theme as Theme, accent: v.accent, showSeconds: v.showSeconds, animationMs: v.animationMs, animation:v.animation as Settings['animation'],position:v.position as Settings['position'],compactWidth:v.compactWidth,compactHeight:v.compactHeight };
}
