<script lang="ts">
	import ContextMenuContent from '$lib/components/ui/ContextMenuContent.svelte';
	import * as api from '$lib/api';
	import type { ContextItem, Mod, ModId } from '$lib/types';
	import { shortenFileSize } from '$lib/util';
	import Icon from '@iconify/svelte';
	import { DropdownMenu } from 'bits-ui';
	import clsx from 'clsx';
	import DropdownArrow from '$lib/components/ui/DropdownArrow.svelte';

	type Props = {
		mod: Mod;
		locked: boolean;
		install: (mod: ModId) => void;
	};

	let { mod, locked, install }: Props = $props();

	let versionsOpen = $state(false);
	let downloadSize: number | null = $state(null);

	let disabled = $derived(mod.isInstalled || locked);

	let modId = $derived({
		packageUuid: mod.uuid,
		versionUuid: mod.versionUuid
	});

	let contextItems: ContextItem[] = $derived(
		mod.versions.map((version) => ({
			label: version.name,
			onclick: () =>
				install({
					packageUuid: mod.uuid,
					versionUuid: version.uuid
				})
		}))
	);

	$effect(() => {
		api.profile.install.getDownloadSize(modId).then((size) => (downloadSize = size));
	});
</script>

<div class="mt-2 flex text-lg text-white">
	<button
		class="enabled:bg-accent-600 enabled:hover:bg-accent-500 disabled:bg-primary-600 disabled:text-primary-300 flex grow items-center justify-center gap-2 rounded-l-lg py-2 font-semibold disabled:cursor-not-allowed"
		onclick={() => install(modId)}
		{disabled}
	>
		{#if locked}
			Profile locked
		{:else if mod.isInstalled}
			Already installed
		{:else}
			<Icon icon="mdi:download" class="align-middle text-xl" />
			Install

			{#if downloadSize}
				({shortenFileSize(downloadSize)})
			{/if}
		{/if}
	</button>
	<DropdownMenu.Root bind:open={versionsOpen}>
		<DropdownMenu.Trigger
			class="enabled:bg-accent-600 enabled:hover:bg-accent-500 disabled:bg-primary-600 disabled:text-primary-300 ml-0.5 gap-2 rounded-r-lg px-1.5 py-2 text-2xl disabled:cursor-not-allowed"
			disabled={mod.isInstalled || locked}
		>
			<DropdownArrow bind:open={versionsOpen} class="text-white" />
		</DropdownMenu.Trigger>
		<ContextMenuContent
			type="dropdown"
			style="light"
			items={contextItems}
			class="max-h-90 overflow-y-auto text-base"
		/>
	</DropdownMenu.Root>
</div>
