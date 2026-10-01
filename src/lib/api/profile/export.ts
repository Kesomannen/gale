import { invoke } from '$lib/invoke';
import type {
	ExportCodeResult,
	ExportFile,
	ModpackArgs,
	ModpackInfo,
	UploadSubmissionResult
} from '$lib/types';

export const code = () => invoke<ExportCodeResult>('export_code');
export const file = (dir: string) => invoke('export_file', { dir });
export const getPackArgs = () => invoke<ModpackInfo>('get_pack_args');
export const setPackArgs = (args: ModpackArgs) => invoke('set_pack_args', { args });
export const exportPack = (dir: string, args: ModpackArgs) => invoke('export_pack', { dir, args });
export const uploadPack = (args: ModpackArgs) =>
	invoke<UploadSubmissionResult>('upload_pack', { args });
export const getExportedFiles = () => invoke<ExportFile[]>('get_exported_files');
export const copyDependencyStrings = () => invoke('copy_dependency_strings');
export const exportDependencyStrings = (directory: string) =>
	invoke('export_dependency_strings', { directory });
export const copyDebugInfo = () => invoke('copy_debug_info');
export const generateChangelog = (args: ModpackArgs, all: boolean) =>
	invoke<string>('generate_changelog', { args, all });
export const listFiles = () => invoke<ExportFile[]>('list_export_files');
export const setExcludedFiles = (excludedFiles: string[]) =>
	invoke('set_excluded_export_files', { excludedFiles });
