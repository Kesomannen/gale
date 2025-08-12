import { invoke } from '$lib/invoke';
import type { LegacyImportData, R2ImportData } from '$lib/types';

export const profile = (data: LegacyImportData, importAll: boolean) =>
	invoke('import_profile', { data, importAll });
export const readCode = (key: string) => invoke<LegacyImportData>('read_profile_code', { key });
export const readFile = (path: string) => invoke<LegacyImportData>('read_profile_file', { path });
export const readBase64 = (base64: string) =>
	invoke<LegacyImportData>('read_profile_base64', { base64 });
export const localMod = (path: string) => invoke('import_local_mod', { path });
export const localModBase64 = (base64: string) => invoke('import_local_mod_base64', { base64 });
export const getR2modmanInfo = (path: string | null) =>
	invoke<R2ImportData | null>('get_r2modman_info', { path });
export const r2modman = (path: string, include: boolean[]) =>
	invoke('import_r2modman', { path, include });
