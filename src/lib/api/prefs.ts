import { invoke } from '$lib/invoke';
import type { Prefs, Zoom } from '$lib/types';

export const get = () => invoke<Prefs>('get_prefs');
export const set = (value: Prefs) => invoke('set_prefs', { value });
export const zoomWindow = (value: Zoom) => invoke('zoom_window', { value });
export const getSystemFonts = () => invoke<string[]>('get_system_fonts');
