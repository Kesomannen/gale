import { invoke } from '$lib/invoke';
import type { ImportData, ModpackArgs, R2ImportData } from '$lib/types';

export const code = () => invoke<string>('export_code');
export const file = (dir: string) => invoke('export_file', { dir });
export const getPackArgs = () => invoke<ModpackArgs>('get_pack_args');
export const setPackArgs = (args: ModpackArgs) => invoke('set_pack_args', { args });
export const exportPack = (dir: string, args: ModpackArgs) => invoke('export_pack', { dir, args });
export const uploadPack = (args: ModpackArgs) => invoke('upload_pack', { args });
export const copyDependencyStrings = () => invoke('copy_dependency_strings');
export const copyDebugInfo = () => invoke('copy_debug_info');
export const generateChangelog = (args: ModpackArgs, all: boolean) =>
	invoke<string>('generate_changelog', { args, all });
