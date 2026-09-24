<script lang="ts">
	import Dialog from '$lib/components/ui/Dialog.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Icon from '@iconify/svelte';
	import type { RemoteDeploymentResult } from '$lib/types';
	import DeploymentStats from './DeploymentStats.svelte';
	import { m } from '$lib/paraglide/messages';
	import InfoBox from '$lib/components/ui/InfoBox.svelte';

	type Result = Extract<RemoteDeploymentResult, { status: 'deployed' }>;

	let open = $state(false);
	let result = $state<Result | null>(null);

	export function openFor(value: Result) {
		result = value;
		open = true;
	}
</script>

<Dialog title={m.deploymentResult_title()} bind:open large>
	{#if result}
		<div class="mt-2 flex items-center gap-2 text-green-700 dark:text-green-400">
			<Icon icon="mdi:check-circle" class="text-2xl" />
			<span class="font-medium">{m.deploymentResult_success()}</span>
		</div>

		<DeploymentStats
			uploaded={result.uploadedFiles}
			bytes={result.uploadedBytes}
			removed={result.removedFiles}
			unchanged={result.unchangedFiles}
		/>

		{#if result.cleanupWarnings.length > 0}
			<InfoBox type="warning" class="mt-4">
				<div class="min-w-0 grow">
					<div class="font-medium">
						{m.deploymentResult_cleanupTitle({ count: result.cleanupWarnings.length })}
					</div>
					<p class="mt-1 text-sm">
						{m.deploymentResult_cleanupContent()}
					</p>
					<details class="mt-2 text-sm">
						<summary class="cursor-pointer font-medium">{m.deploymentResult_showDetails()}</summary>
						<ul class="mt-2 max-h-40 overflow-auto pl-5">
							{#each result.cleanupWarnings as warning}
								<li class="list-disc py-0.5 wrap-anywhere">{warning}</li>
							{/each}
						</ul>
					</details>
				</div>
			</InfoBox>
		{/if}

		<div class="mt-5 flex justify-end">
			<Button onclick={() => (open = false)}>{m.deploymentResult_done()}</Button>
		</div>
	{/if}
</Dialog>
