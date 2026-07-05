<script lang="ts">
	import ContextMenuContent from '$lib/components/ui/ContextMenuContent.svelte';
	import * as api from '$lib/api';
	import { Backend, type ContextItem, type Mod, type ModId } from '$lib/types';
	import { shortenFileSize } from '$lib/util';
	import Icon from '@iconify/svelte';
	import { DropdownMenu } from 'bits-ui';
	import DropdownArrow from '$lib/components/ui/DropdownArrow.svelte';
	import Spinner from '../ui/Spinner.svelte';
	import { m } from '$lib/paraglide/messages';
	import ConfirmDialog from '$lib/components/ui/ConfirmDialog.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Checkbox from '$lib/components/ui/Checkbox.svelte';
	import Info from '$lib/components/ui/Info.svelte';
	import Label from '$lib/components/ui/Label.svelte';

	type Props = {
		mod: Mod;
		locked: boolean;
		install: (mod: ModId) => void;
	};

	let { mod, locked, install }: Props = $props();

	let modId = $derived({
		packageUuid: mod.uuid,
		versionUuid: mod.versionUuid,
		backend: mod.backend
	});

	let versionsOpen = $state(false);
	let dialogOpen = $state(false);
	// svelte-ignore state_referenced_locally
	let installPayload: ModId = $state(modId);
	let downloadSize: number | null = $state(null);

	let loading = $state(false);
	let warnNoRemind = $state(false);

	let disabled = $derived(mod.isInstalled || locked || loading);

	let doInstall = async () => {
		if (warnNoRemind) {
			let prefs = await api.prefs.get();
			prefs.backendSkipConfirm = true;
			await api.prefs.set(prefs);
		}
		install(installPayload);
		loading = true;
		dialogOpen = false;
	};
	let tryInstall = async (modId: ModId) => {
		installPayload = modId;
		if (mod.backend !== Backend.Thunderstore && !(await api.prefs.get()).backendSkipConfirm) {
			dialogOpen = true;
		} else {
			doInstall();
		}
	};

	let contextItems: ContextItem[] = $derived(
		mod.versions.map((version) => ({
			label: version.name,
			onclick: async () => tryInstall({
				packageUuid: mod.uuid,
				versionUuid: version.uuid,
				backend: mod.backend
			}),
		}))
	);

	$effect(() => {
		loading = false;
		api.profile.install.getDownloadSize(modId).then((size) => (downloadSize = size));
	});
</script>

<div class="mt-2 flex text-lg text-white">
	<button
		class="enabled:bg-accent-700 enabled:hover:bg-accent-600 disabled:bg-primary-700 disabled:text-primary-300 flex grow items-center justify-center gap-2 rounded-l-lg py-2 font-semibold disabled:cursor-not-allowed"
		onclick={() => tryInstall(modId)}
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

<ConfirmDialog title={m.otherServer_warn_title()} bind:open={dialogOpen}>
	{m.otherServer_warn_content()}
	<div class="my-5 flex items-center">
		<Checkbox id="neverwarninstall" bind:checked={warnNoRemind} />
		<label class="ml-3" for="neverwarninstall">
			{m.otherServer_warn_noremind()}
		</label>
	</div>

	{#snippet buttons()}
		<Button color="accent" icon="mdi:download" onclick={doInstall}>
			{m.installModButton_button_install()}
		</Button>
	{/snippet}
</ConfirmDialog>
