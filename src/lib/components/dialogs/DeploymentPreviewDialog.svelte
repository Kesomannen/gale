<script lang="ts">
	import Dialog from '$lib/components/ui/Dialog.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import type { RemoteDeploymentPreviewResult } from '$lib/types';
	import DeploymentStats from './DeploymentStats.svelte';
	import { m } from '$lib/paraglide/messages';
	import InfoBox from '$lib/components/ui/InfoBox.svelte';

	type Preview = Extract<RemoteDeploymentPreviewResult, { status: 'preview' }>;

	let open = $state(false);
	let preview = $state<Preview | null>(null);
	let resolveReview: ((accepted: boolean) => void) | null = null;

	export function openFor(value: Preview) {
		preview = value;
		open = true;
		return new Promise<boolean>((resolve) => {
			resolveReview = resolve;
		});
	}

	function close(accepted: boolean) {
		const resolve = resolveReview;
		resolveReview = null;
		resolve?.(accepted);
		open = false;
	}
</script>

<Dialog title={m.deploymentPreview_title()} bind:open large onclose={() => close(false)}>
	{#if preview}
		<p class="text-primary-600 dark:text-primary-300 mt-1">
			{m.deploymentPreview_content()}
		</p>

		<DeploymentStats
			uploaded={preview.uploadFiles.length}
			bytes={preview.uploadBytes}
			removed={preview.removeFiles.length}
			unchanged={preview.unchangedFiles}
		/>

		{#if preview.removeFiles.length > 0}
			<InfoBox type="warning" class="mt-4">{m.deploymentPreview_removalInfo()}</InfoBox>
		{/if}

		<details class="mt-4">
			<summary class="text-primary-700 dark:text-primary-300 cursor-pointer font-medium">
				{m.deploymentPreview_fileChanges({
					count: preview.uploadFiles.length + preview.removeFiles.length
				})}
			</summary>
			<div
				class="border-primary-300 dark:border-primary-600 bg-primary-50 dark:bg-primary-900 mt-2 max-h-64 overflow-auto rounded-lg border p-3 font-mono text-sm"
			>
				{#if preview.uploadFiles.length > 0}
					<div class="text-primary-500 mb-1 font-sans font-semibold">
						{m.deploymentPreview_uploads()}
					</div>
					{#each preview.uploadFiles as path}
						<div class="text-primary-700 dark:text-primary-300 flex gap-2 py-0.5">
							<span class="shrink-0 text-green-600 dark:text-green-400">+</span>
							<span class="wrap-anywhere">{path}</span>
						</div>
					{/each}
				{/if}
				{#if preview.removeFiles.length > 0}
					<div class="text-primary-500 mt-3 mb-1 font-sans font-semibold">
						{m.deploymentPreview_removals()}
					</div>
					{#each preview.removeFiles as path}
						<div class="text-primary-700 dark:text-primary-300 flex gap-2 py-0.5">
							<span class="shrink-0 text-red-600 dark:text-red-400">−</span>
							<span class="wrap-anywhere">{path}</span>
						</div>
					{/each}
				{/if}
			</div>
		</details>

		<div class="mt-5 flex justify-end gap-2">
			<Button color="primary" onclick={() => close(false)}>{m.deploymentPreview_cancel()}</Button>
			<Button icon="mdi:cloud-upload" onclick={() => close(true)}
				>{m.deploymentPreview_deploy()}</Button
			>
		</div>
	{/if}
</Dialog>
