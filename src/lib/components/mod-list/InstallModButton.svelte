<script lang="ts">
	import ContextMenuContent from '$lib/components/ui/ContextMenuContent.svelte';
	import * as api from '$lib/api';
	import type { ContextItem, Mod, ModId } from '$lib/types';
	import { shortenFileSize } from '$lib/util';
	import Icon from '@iconify/svelte';
	import { DropdownMenu } from 'bits-ui';
	import DropdownArrow from '$lib/components/ui/DropdownArrow.svelte';
	import Spinner from '../ui/Spinner.svelte';
	import { m } from '$lib/paraglide/messages';

	type Props = {
		mod: Mod;
		locked: boolean;
		loading: boolean;
		install: (mod: ModId) => void;
	};

	let { mod, locked, loading = $bindable(), install }: Props = $props();

	let versionsOpen = $state(false);
	let downloadSize: number | null = $state(null);

	let disabled = $derived(mod.isInstalled || locked || loading);

	let modId = $derived({
		packageUuid: mod.uuid,
		versionUuid: mod.versionUuid,
		backend: mod.backend
	});

	let contextItems: ContextItem[] = $derived(
		mod.versions.map((version) => ({
			label: version.name,
			onclick: () => install({
				packageUuid: mod.uuid,
				versionUuid: version.uuid,
				backend: mod.backend
			}),
		}))
	);

	$effect(() => {
		api.profile.install.getDownloadSize(modId).then((size) => (downloadSize = size));
	});
</script>

<div class="mt-2 flex text-lg text-white">
	<button
		class="enabled:bg-accent-700 enabled:hover:bg-accent-600 disabled:bg-primary-700 disabled:text-primary-300 flex grow items-center justify-center gap-2 rounded-l-lg py-2 font-semibold disabled:cursor-not-allowed"
		onclick={() => install(modId)}
		{disabled}
	>
		{#if locked}
			{m.installModButton_button_locked()}
		{:else if mod.isInstalled}
			{m.installModButton_button_isInstalled()}
		{:else if loading}
			<Spinner />

			{m.installModButton_button_loading()}
		{:else}
			<Icon icon="mdi:download" class="align-middle text-xl" />
			{m.installModButton_button_install()}

			{#if downloadSize}
				({shortenFileSize(downloadSize)})
			{/if}
		{/if}
	</button>
	<DropdownMenu.Root bind:open={versionsOpen}>
		<DropdownMenu.Trigger
			class="enabled:bg-accent-700 enabled:hover:bg-accent-600 disabled:bg-primary-700 disabled:text-primary-300 ml-0.5 gap-2 rounded-r-lg px-1.5 py-2 text-2xl disabled:cursor-not-allowed"
			{disabled}
		>
			<DropdownArrow open={versionsOpen} class="text-white" />
		</DropdownMenu.Trigger>
		<ContextMenuContent
			type="dropdown"
			items={contextItems}
			class="max-h-90 overflow-y-auto text-base"
		/>
	</DropdownMenu.Root>
</div>
