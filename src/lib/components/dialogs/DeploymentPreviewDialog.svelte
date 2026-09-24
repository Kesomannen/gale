<script lang="ts">
	import Dialog from '$lib/components/ui/Dialog.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Checkbox from '$lib/components/ui/Checkbox.svelte';
	import type { RemoteDeploymentPreviewResult, RemoteDeploymentSelection } from '$lib/types';
	import DeploymentStats from './DeploymentStats.svelte';
	import { m } from '$lib/paraglide/messages';
	import InfoBox from '$lib/components/ui/InfoBox.svelte';

	type Preview = Extract<RemoteDeploymentPreviewResult, { status: 'preview' }>;
	type FileEntry = { path: string; name: string };
	type FileGroup = { directory: string; files: FileEntry[] };
	type FileOperation = 'upload' | 'remove';

	let open = $state(false);
	let preview = $state<Preview | null>(null);
	let selectedUploadFiles = $state<string[]>([]);
	let selectedRemoveFiles = $state<string[]>([]);
	let selectedUploads = $derived(new Set(selectedUploadFiles));
	let selectedRemovals = $derived(new Set(selectedRemoveFiles));
	let resolveReview: ((selection: RemoteDeploymentSelection | null) => void) | null = null;
	let hasChanges = $derived(
		preview !== null && (preview.uploadFiles.length > 0 || preview.removeFiles.length > 0)
	);
	let hasSelectedChanges = $derived(
		selectedUploadFiles.length > 0 || selectedRemoveFiles.length > 0
	);
	let selectedUploadBytes = $derived(
		selectedUploadFiles.reduce((total, path) => total + (preview?.uploadFileSizes[path] ?? 0), 0)
	);
	let uploadGroups = $derived(groupFiles(preview?.uploadFiles ?? []));
	let removalGroups = $derived(groupFiles(preview?.removeFiles ?? []));

	function groupFiles(paths: string[]): FileGroup[] {
		const groups = new Map<string, FileEntry[]>();

		for (const path of paths) {
			const normalized = path.endsWith('/') ? path.slice(0, -1) : path;
			const separator = normalized.lastIndexOf('/');
			const directory = separator === -1 ? '' : normalized.slice(0, separator);
			const name = normalized.slice(separator + 1) + (path.endsWith('/') ? '/' : '');
			const files = groups.get(directory) ?? [];
			files.push({ path, name });
			groups.set(directory, files);
		}

		return Array.from(groups, ([directory, files]) => ({ directory, files }));
	}

	export function openFor(value: Preview) {
		preview = value;
		selectedUploadFiles = [...value.uploadFiles];
		selectedRemoveFiles = [...value.removeFiles];
		open = true;
		return new Promise<RemoteDeploymentSelection | null>((resolve) => {
			resolveReview = resolve;
		});
	}

	function isSelected(operation: FileOperation, path: string) {
		return (operation === 'upload' ? selectedUploads : selectedRemovals).has(path);
	}

	function setSelected(operation: FileOperation, paths: string[], selected: boolean) {
		const current = operation === 'upload' ? selectedUploadFiles : selectedRemoveFiles;
		const changedPaths = new Set(paths);
		const changed = selected
			? [...new Set([...current, ...changedPaths])]
			: current.filter((path) => !changedPaths.has(path));
		if (operation === 'upload') selectedUploadFiles = changed;
		else selectedRemoveFiles = changed;
	}

	function allSelected(operation: FileOperation, paths: string[]) {
		return paths.every((path) => isSelected(operation, path));
	}

	function close(selection: RemoteDeploymentSelection | null) {
		const resolve = resolveReview;
		resolveReview = null;
		resolve?.(selection);
		open = false;
	}
</script>

{#snippet selectionButton(operation: FileOperation, paths: string[])}
	<button
		type="button"
		class="text-accent-600 hover:text-accent-700 dark:text-accent-400 dark:hover:text-accent-300 shrink-0 text-xs hover:underline"
		onclick={() => setSelected(operation, paths, !allSelected(operation, paths))}
	>
		{allSelected(operation, paths)
			? m.deploymentPreview_selectNone()
			: m.deploymentPreview_selectAll()}
	</button>
{/snippet}

{#snippet fileGroups(groups: FileGroup[], operation: FileOperation)}
	{#if groups.length > 0}
		{@const paths = groups.flatMap((group) => group.files.map((file) => file.path))}
		<div class={operation === 'remove' ? 'mt-4' : ''}>
			<div class="border-primary-200 dark:border-primary-700 flex items-center border-b pb-2">
				<div
					class="text-primary-800 dark:text-primary-200 flex min-w-0 items-center gap-2 font-semibold"
				>
					<span
						class={[
							'text-base',
							operation === 'upload'
								? 'text-green-600 dark:text-green-400'
								: 'text-red-600 dark:text-red-400'
						]}
					>
						{operation === 'upload' ? '+' : '−'}
					</span>
					<span>
						{operation === 'upload'
							? m.deploymentPreview_uploads()
							: m.deploymentPreview_removals()}
					</span>
					<span class="text-primary-500 dark:text-primary-400 text-xs font-normal">
						{paths.filter((path) => isSelected(operation, path)).length}/{paths.length}
					</span>
				</div>
				<div class="ml-auto">{@render selectionButton(operation, paths)}</div>
			</div>

			{#each groups as group}
				{@const paths = group.files.map((file) => file.path)}
				<div class="py-2.5 last:pb-0">
					<div class="flex min-w-0 items-center gap-3">
						<span
							class="text-primary-600 dark:text-primary-300 truncate text-sm font-medium"
							title={group.directory || m.deploymentPreview_serverRoot()}
						>
							{group.directory || m.deploymentPreview_serverRoot()}
						</span>
						<span class="text-primary-500 dark:text-primary-400 ml-auto shrink-0 text-xs">
							{group.files.filter((file) => isSelected(operation, file.path)).length}/{group.files
								.length}
						</span>
						{@render selectionButton(operation, paths)}
					</div>
					<div class="mt-1 space-y-0.5">
						{#each group.files as file}
							<div
								class="hover:bg-primary-100 dark:hover:bg-primary-800 text-primary-700 dark:text-primary-200 flex min-w-0 items-center rounded px-2 py-1 text-sm"
							>
								<Checkbox
									size="sm"
									class="mr-2.5 shrink-0"
									checked={isSelected(operation, file.path)}
									onCheckedChange={(selected) => setSelected(operation, [file.path], selected)}
								/>
								<span class="wrap-anywhere">{file.name}</span>
							</div>
						{/each}
					</div>
				</div>
			{/each}
		</div>
	{/if}
{/snippet}

<Dialog title={m.deploymentPreview_title()} bind:open large onclose={() => close(null)}>
	{#if preview}
		<p class="text-primary-600 dark:text-primary-300 mt-1">
			{hasChanges ? m.deploymentPreview_content() : m.deploymentPreview_noChanges()}
		</p>

		<DeploymentStats
			uploaded={selectedUploadFiles.length}
			bytes={selectedUploadBytes}
			removed={selectedRemoveFiles.length}
			unchanged={preview.unchangedFiles}
		/>

		{#if preview.skippedClientOnlyMods.length > 0}
			<InfoBox icon="mdi:monitor" class="mt-4">
				<div class="min-w-0 grow">
					<div class="font-medium">{m.deploymentPreview_clientOnlyTitle()}</div>
					<p class="mt-1 text-sm">
						{m.deploymentPreview_clientOnlyContent({ count: preview.skippedClientOnlyMods.length })}
					</p>
					<details class="mt-2 text-sm">
						<summary class="cursor-pointer font-medium">{m.deploymentPreview_showMods()}</summary>
						<ul class="mt-1 max-h-32 overflow-auto pl-5">
							{#each preview.skippedClientOnlyMods as mod}
								<li class="list-disc py-0.5 wrap-anywhere">{mod}</li>
							{/each}
						</ul>
					</details>
				</div>
			</InfoBox>
		{/if}

		{#if preview.preservedFiles > 0}
			<InfoBox icon="mdi:shield-check" class="mt-4">
				<div class="min-w-0 grow">
					<div class="font-medium">{m.deploymentPreview_preservedTitle()}</div>
					<p class="mt-1 text-sm">
						{m.deploymentPreview_preservedContent({ count: preview.preservedFiles })}
					</p>
				</div>
			</InfoBox>
		{/if}

		{#if hasChanges}
			<details class="mt-4">
				<summary class="text-primary-700 dark:text-primary-300 cursor-pointer font-medium">
					{m.deploymentPreview_fileChanges({
						count: preview.uploadFiles.length + preview.removeFiles.length
					})}
				</summary>
				<div
					class="bg-primary-50 dark:bg-primary-900/40 mt-2 max-h-72 overflow-auto rounded-lg p-4"
				>
					{@render fileGroups(uploadGroups, 'upload')}
					{@render fileGroups(removalGroups, 'remove')}
				</div>
			</details>
		{/if}

		<div class="mt-5 flex justify-end gap-2">
			{#if hasSelectedChanges}
				<Button color="primary" onclick={() => close(null)}>{m.deploymentPreview_cancel()}</Button>
				<Button
					icon="mdi:cloud-upload"
					onclick={() =>
						close({ uploadFiles: selectedUploadFiles, removeFiles: selectedRemoveFiles })}
					>{m.deploymentPreview_deploy()}</Button
				>
			{:else}
				<Button onclick={() => close(null)}>{m.deploymentResult_done()}</Button>
			{/if}
		</div>
	{/if}
</Dialog>
