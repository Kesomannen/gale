<script lang="ts">
	import Button from '$lib/components/ui/Button.svelte';
	import Dialog from '$lib/components/ui/Dialog.svelte';
	import FileTree from '$lib/components/ui/FileTree.svelte';
	import InfoBox from '$lib/components/ui/InfoBox.svelte';
	import { m } from '$lib/paraglide/messages';
	import type { RemoteDeploymentPreviewResult, RemoteDeploymentSelection } from '$lib/types';
	import DeploymentStats from './DeploymentStats.svelte';

	type Preview = Extract<RemoteDeploymentPreviewResult, { status: 'preview' }>;
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

	function close(selection: RemoteDeploymentSelection | null) {
		const resolve = resolveReview;
		resolveReview = null;
		resolve?.(selection);
		open = false;
	}
</script>

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
				<div class="mt-2 space-y-3">
					{#if preview.uploadFiles.length > 0}
						<FileTree
							title={m.deploymentPreview_uploads()}
							items={preview.uploadFiles}
							getPath={(path) => path}
							getSize={(path) => preview?.uploadFileSizes[path] ?? 0}
							get={(path) => isSelected('upload', path)}
							set={(path, selected) => setSelected('upload', [path], selected)}
							setMany={(paths, selected) => setSelected('upload', paths, selected)}
							maxHeight="sm"
						/>
					{/if}

					{#if preview.removeFiles.length > 0}
						<FileTree
							title={m.deploymentPreview_removals()}
							items={preview.removeFiles}
							getPath={(path) => path}
							get={(path) => isSelected('remove', path)}
							set={(path, selected) => setSelected('remove', [path], selected)}
							setMany={(paths, selected) => setSelected('remove', paths, selected)}
							maxHeight="sm"
						/>
					{/if}
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
