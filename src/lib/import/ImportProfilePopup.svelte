<script lang="ts">
	import Popup from '$lib/components/Popup.svelte';
	import TabsMenu from '$lib/components/TabsMenu.svelte';

	import { Tabs } from 'bits-ui';

	import { invokeCommand } from '$lib/invoke';
	import type { ImportData } from '$lib/models';
	import Icon from '@iconify/svelte';
	import { readText } from '@tauri-apps/plugin-clipboard-manager';
	import { confirm } from '@tauri-apps/plugin-dialog';
	import InputField from '$lib/components/InputField.svelte';
	import { profiles, refreshProfiles } from '$lib/stores';
	import BigButton from '$lib/components/BigButton.svelte';
	import Label from '$lib/components/Label.svelte';
	import Dropdown from '$lib/components/Dropdown.svelte';
	import ModCardList from '$lib/modlist/ModCardList.svelte';

	import { t, T } from '$i18n';

	export let open: boolean;
	export let data: ImportData | null;

	let key: string;
	let name: string;
	let loading: boolean;
	let mode: 'new' | 'overwrite' = 'new';

	$: if (open) {
		getKeyFromClipboard();
	}

	$: if (mode === 'overwrite' && isAvailable(name)) {
		name = profiles[0].name;
	}

	$: nameAvailable = mode === 'overwrite' || isAvailable(name);

	async function getKeyFromClipboard() {
		key = (await readText()) ?? '';
	}

	async function submitKey() {
		loading = true;
		try {
			data = await invokeCommand<ImportData>('import_code', { key: key.trim() });
			name = data.name;
			mode = isAvailable(name) ? 'new' : 'overwrite';
			console.log(data);
		} catch (e) {
			open = false;
		} finally {
			loading = false;
		}
	}

	async function importData() {
		if (!data) return;

		data.name = name;

		if (mode === 'overwrite') {
			let confirmed = await confirm(T('Import profile override description', {"name": data.name}), {
				title: t('Import profile override')
			});

			if (!confirmed) return;
		}

		invokeCommand('import_data', { data }).then(refreshProfiles);
		data = null;
		open = false;
	}

	function isAvailable(name: string) {
		return !profiles.some((profile) => profile.name === name);
	}
</script>

<Popup title={t('Import profile')} large={data !== null} bind:open onClose={() => (data = null)}>
	{#if data}
		<TabsMenu
			bind:value={mode}
			options={[
				{ value: 'new', label: t('Create new') },
				{ value: 'overwrite', label: t('Overwrite existing') }
			]}
		>
			<Tabs.Content value="new">
				<div class="flex items-center">
					<Label text="{t('Profile name')}" />

					<InputField bind:value={name} class="w-full" />
				</div>

				{#if !nameAvailable}
					<div class="text-md mt-1 flex items-center gap-1 font-bold text-red-400">
						<div class="w-[30%] min-w-52" />
						<Icon icon="mdi:error" class="text-lg" />
						{T('Profile name exists', {"name": name})}
					</div>
				{/if}
			</Tabs.Content>

			<Tabs.Content value="overwrite">
				<div class="flex items-center">
					<Label text="{t('Choose profile')}" />

					<Dropdown
						class="flex-grow"
						items={profiles.map((profile) => profile.name)}
						avoidCollisions={false}
						bind:selected={name}
					/>
				</div>
			</Tabs.Content>
		</TabsMenu>

		{#if data.modNames}
			<h3 class="mt-2 text-lg font-semibold text-white">{T('Many mods to install', {"length": data.mods.length})}</h3>
			<ModCardList names={data.modNames} class="mt-2 max-h-[50vh] flex-shrink flex-grow" />
		{/if}

		<div class="mt-2 flex w-full items-center justify-end gap-2 text-slate-400">
			<BigButton
				color="gray"
				on:click={() => {
					open = false;
					data = null;
				}}>{t("Cancel")}</BigButton
			>
			<BigButton disabled={!nameAvailable || loading} on:click={importData}>{t("Import")}</BigButton>
		</div>
	{:else}
		<div class="mt-1 flex gap-2">
			<div class="flex-grow">
				<InputField bind:value={key} class="w-full" size="lg" placeholder="{t('Enter import code')}" />
			</div>

			<BigButton on:click={submitKey} disabled={loading}>
				{#if loading}
					<Icon icon="mdi:loading" class="animate-spin" />
				{:else}
					{t("Import")}
				{/if}
			</BigButton>
		</div>
	{/if}
</Popup>
