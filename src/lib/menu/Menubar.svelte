<script lang="ts">
	import Icon from '@iconify/svelte';
	import { Button, Menubar } from 'bits-ui';

	import MenubarTrigger from '$lib/menu/MenubarTrigger.svelte';
	import MenubarItem from '$lib/menu/MenubarItem.svelte';

	import { invokeCommand } from '$lib/invoke';

	import { open as shellOpen } from '@tauri-apps/plugin-shell';
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import ImportProfilePopup from '$lib/import/ImportProfilePopup.svelte';
	import ExportCodePopup from '$lib/import/ExportCodePopup.svelte';
	import { confirm, open } from '@tauri-apps/plugin-dialog';
	import type { ImportData } from '$lib/models';
	import ImportR2Popup from '$lib/import/ImportR2Popup.svelte';
	import { activeProfile, refreshProfiles } from '$lib/stores';
	import NewProfilePopup from './NewProfilePopup.svelte';
	import InputField from '$lib/components/InputField.svelte';
	import BigButton from '$lib/components/BigButton.svelte';
	import { capitalize } from '$lib/util';
	import Popup from '$lib/components/Popup.svelte';
	import { refreshUpdate } from './Updater.svelte';
	import MenubarSeparator from './MenubarSeparator.svelte';
	import hotkeys from 'hotkeys-js';
	import { onMount } from 'svelte';
	import { writeText } from '@tauri-apps/plugin-clipboard-manager';

	import { get } from 'svelte/store';
	import { T, t } from '$i18n';

	let importR2Open = false;
	let newProfileOpen = false;
	let exportCodePopup: ExportCodePopup;

	let importProfileOpen = false;
	let importProfileData: ImportData | null = null;

	let profileOperation: 'rename' | 'duplicate' = 'rename';
	let profileOperationName = '';
	let profileOperationOpen = false;
	let profileOperationInProgress = false;

	const appWindow = getCurrentWindow();

	async function importLocalMod() {
		let path = await open({
			title: t("Import mod file description"),
			filters: [{ name: 'Dll or zip', extensions: ['dll', 'zip'] }]
		});

		if (path === null) return;
		invokeCommand('import_local_mod', { path });

		activeProfile.update((profile) => profile);
	}

	async function importFile() {
		let path = await open({
			title: t("Import file description"),
			filters: [{ name: 'Profile file', extensions: ['r2z'] }]
		});

		if (path === null) return;
		let data = await invokeCommand<ImportData>('import_file', { path });

		importProfileData = data;
		importProfileOpen = true;
	}

	async function exportFile() {
		let dir = await open({
			directory: true,
			title: t('Export file description')
		});

		if (dir === null) return;
		invokeCommand('export_file', { dir });
	}

	async function setAllModsState(enable: boolean) {
		await invokeCommand('set_all_mods_state', { enable });
		activeProfile.update((profile) => profile);
	}

	function openProfileOperation(operation: 'rename' | 'duplicate') {
		profileOperation = operation;
		profileOperationName = $activeProfile?.name ?? 'Unknown';
		profileOperationOpen = true;
	}

	async function doProfileOperation() {
		if (profileOperationInProgress) return;

		profileOperationInProgress = true;

		try {
			if (profileOperation == 'rename') {
				await invokeCommand('rename_profile', { name: profileOperationName });
			} else if (profileOperation == 'duplicate') {
				await invokeCommand('duplicate_profile', { name: profileOperationName });
			}
		} catch (e) {
			profileOperationInProgress = false;
			throw e;
		}

		await refreshProfiles();
		profileOperationInProgress = false;
		profileOperationOpen = false;
	}

	async function uninstallDisabledMods() {
		let confirmed = await confirm('Are you sure you want to uninstall all disabled mods?');
		if (!confirmed) return;

		let removed = await invokeCommand<number>('remove_disabled_mods');
		activeProfile.update((profile) => {
			if (profile === null) return null;

			profile.modCount -= removed;
			return profile;
		});
	}

	async function zoom(value: { delta: number } | { factor: number }) {
		await invokeCommand('zoom_window', { value });
	}

	async function copyLaunchArgs() {
		let str = await invokeCommand<string>('get_launch_args');
		writeText(str);
	}

	onMount(() => {
		hotkeys('ctrl|+,ctrl|-,ctrl|0,ctrl|n,ctrl|d,f2', { splitKey: '|' }, (evt, handler) => {
			console.log(handler.key);
			switch (handler.key) {
				case 'ctrl|+':
					zoom({ delta: 0.25 });
					break;

				case 'ctrl|-':
					zoom({ delta: -0.25 });
					break;

				case 'ctrl|0':
					zoom({ factor: 1 });
					break;

				case 'ctrl|n':
					newProfileOpen = true;
					break;

				case 'ctrl|d':
					openProfileOperation('duplicate');
					break;

				case 'f2':
					openProfileOperation('rename');
					break;

				default:
					return true;
			}

			return false;
		});
	});
</script>

<div data-tauri-drag-region class="flex h-8 flex-shrink-0 bg-gray-800">
	<Menubar.Root class="flex items-center py-1">
		<img src="favicon.png" alt="Gale logo" class="ml-4 mr-2 h-5 w-5 opacity-50" />
		<Menubar.Menu>
			<MenubarTrigger>{t("File")}</MenubarTrigger>
			<Menubar.Content
				class="mt-0.5 flex flex-col gap-0.5 rounded-lg border border-gray-600 bg-gray-800 py-1 shadow-xl"
			>
				<MenubarItem
					on:click={() => invokeCommand('open_profile_dir')}
					text={t('Open profile directory')}
				/>
				<MenubarItem on:click={() => invokeCommand('open_bepinex_log')} text={t('Open game logs')} />
				<MenubarItem on:click={() => invokeCommand('open_gale_log')} text={t('Open gale logs')} />
				<MenubarSeparator />
				<MenubarItem
					on:click={() => invokeCommand('clear_download_cache', { soft: true })}
					text={t('Clear unused mod cache')}
				/>
				<MenubarItem
					on:click={() => invokeCommand('clear_download_cache', { soft: false })}
					text={t('Clear all cached mods')}
				/>
				<MenubarSeparator />
				<MenubarItem on:click={() => invokeCommand('trigger_mod_fetching')} text={t('Fetch mods')} />
			</Menubar.Content>
		</Menubar.Menu>
		<Menubar.Menu>
			<MenubarTrigger>{t("Profile")}</MenubarTrigger>
			<Menubar.Content
				class="mt-0.5 flex flex-col gap-0.5 rounded-lg border border-gray-600 bg-gray-800 py-1 shadow-xl"
			>
				<MenubarItem
					on:click={() => (newProfileOpen = true)}
					text={t('Create new profile')}
					key="Ctrl+N"
				/>
				<MenubarItem
					on:click={() => openProfileOperation('rename')}
					text={t('Rename active profile')}
					key="F2"
				/>
				<MenubarItem
					on:click={() => openProfileOperation('duplicate')}
					text={t('Duplicate active profile')}
					key="Ctrl+D"
				/>
				<MenubarSeparator />
				<MenubarItem
					on:click={() => invokeCommand('copy_dependency_strings')}
					text={t('Copy mod list')}
				/>
				<MenubarItem on:click={() => invokeCommand('copy_debug_info')} text={t('Copy debug info')} />
				<MenubarItem on:click={copyLaunchArgs} text={t('Copy launch arguments')} />
				<MenubarSeparator />
				<MenubarItem on:click={() => setAllModsState(true)} text={t('Enable all mods')} />
				<MenubarItem on:click={() => setAllModsState(false)} text={t('Disable all mods')} />
				<MenubarItem on:click={uninstallDisabledMods} text={t('Uninstall disabled mods')} />
			</Menubar.Content>
		</Menubar.Menu>
		<Menubar.Menu>
			<MenubarTrigger>{t("Import")}</MenubarTrigger>
			<Menubar.Content
				class="mt-0.5 flex flex-col gap-0.5 rounded-lg border border-gray-600 bg-gray-800 py-1 shadow-xl"
			>
				<MenubarItem on:click={() => (importProfileOpen = true)} text={t('Import profile from code')} />
				<MenubarItem on:click={importFile} text={t('Import profile from file')} />
				<MenubarItem on:click={importLocalMod} text={t('Import local mod')} />
				<MenubarItem on:click={() => (importR2Open = true)} text={t('Import profiles from r2modman')} />
			</Menubar.Content>
		</Menubar.Menu>
		<Menubar.Menu>
			<MenubarTrigger>{t("Export")}</MenubarTrigger>
			<Menubar.Content
				class="mt-0.5 flex flex-col gap-0.5 rounded-lg border border-gray-600 bg-gray-800 py-1 shadow-xl"
			>
				<MenubarItem on:click={() => exportCodePopup.open()} text={t('Export profile as code')} />
				<MenubarItem on:click={exportFile} text={t('Export profile as file')}/>
			</Menubar.Content>
		</Menubar.Menu>
		<Menubar.Menu>
			<MenubarTrigger>{t("Window")}</MenubarTrigger>
			<Menubar.Content
				class="mt-0.5 flex flex-col gap-0.5 rounded-lg border border-gray-600 bg-gray-800 py-1 shadow-xl"
			>
				<MenubarItem on:click={appWindow.minimize} text={t("Minimize")} />
				<MenubarItem on:click={appWindow.toggleMaximize} text={t("Maximize")} />
				<MenubarItem on:click={appWindow.close} text={t("Close")} />
				<MenubarSeparator />
				<MenubarItem
					on:click={() => invokeCommand('zoom_window', { value: { delta: 0.25 } })}
					text={t("Zoom in")}
					key="Ctrl++"
				/>
				<MenubarItem
					on:click={() => invokeCommand('zoom_window', { value: { delta: -0.25 } })}
					text={t("Zoom out")}
					key="Ctrl+-"
				/>
				<MenubarItem
					on:click={() => invokeCommand('zoom_window', { value: { factor: 1 } })}
					text={t("Reset zoom")}
					key="Ctrl+0"
				/>
			</Menubar.Content>
		</Menubar.Menu>
		<Menubar.Menu>
			<MenubarTrigger>{t("Help")}</MenubarTrigger>
			<Menubar.Content
				class="mt-0.5 flex flex-col gap-0.5 rounded-lg border border-gray-600 bg-gray-800 py-1 shadow-xl"
			>
				<MenubarItem on:click={refreshUpdate} text={t('Check for app updates')} />
				<MenubarItem
					on:click={() => shellOpen('https://github.com/Kesomannen/ModManager/issues/')}
					text={t('Report a bug')}
				/>
				<MenubarItem
					on:click={() => shellOpen('https://discord.gg/lcmod')}
					text={t('Join LC modding server')}
				/>
				<MenubarItem
					on:click={() =>
						shellOpen('https://discord.com/channels/1168655651455639582/1246088342458863618')}
					text={t('Open discord thread')}
				/>
			</Menubar.Content>
		</Menubar.Menu>
	</Menubar.Root>

	<Button.Root class="group ml-auto px-3 py-1.5 hover:bg-gray-700" on:click={appWindow.minimize}>
		<Icon icon="mdi:minimize" class="text-gray-500 group-hover:text-white" />
	</Button.Root>
	<Button.Root class="group px-3 py-1.5 hover:bg-gray-700" on:click={appWindow.toggleMaximize}>
		<Icon icon="mdi:maximize" class="text-gray-500 group-hover:text-white" />
	</Button.Root>
	<Button.Root class="group px-3 py-1.5 hover:bg-red-700" on:click={appWindow.close}>
		<Icon icon="mdi:close" class="text-gray-500 group-hover:text-white" />
	</Button.Root>
</div>

<Popup
	title="{t(`${profileOperation} profile`)}"
	canClose={!profileOperationInProgress}
	bind:open={profileOperationOpen}
>
	<p class="mb-1 text-slate-300">
		{profileOperation == 'duplicate'
			? t('Duplicate active profile description')
			: t('Rename active profile description')}
	</p>
	<InputField
		bind:value={profileOperationName}
		placeholder="{t('Enter name')}"
		size="lg"
		class="w-full"
		on:submit={doProfileOperation}
	/>
	{#if profileOperation == 'duplicate'}
		<p class="mt-2 text-sm text-slate-400">
			{t('Duplicate active profile processing')}
		</p>
	{/if}
	<div class="ml-auto mt-2 flex justify-end gap-2">
		{#if !profileOperationInProgress}
			<BigButton color="gray" on:click={() => (profileOperationOpen = false)}>{t("Cancel")}</BigButton>
		{/if}
		<BigButton
			color="green"
			fontWeight="medium"
			disabled={profileOperationInProgress}
			on:click={doProfileOperation}
		>
			{#if profileOperationInProgress}
				<Icon icon="mdi:loading" class="my-1 animate-spin text-lg" />
			{:else}
				{t(profileOperation)}
			{/if}
		</BigButton>
	</div>
</Popup>

<NewProfilePopup bind:open={newProfileOpen} />
<ImportProfilePopup bind:open={importProfileOpen} bind:data={importProfileData} />
<ExportCodePopup bind:this={exportCodePopup} />
<ImportR2Popup bind:open={importR2Open} />
