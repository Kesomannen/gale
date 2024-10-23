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
	import AboutPopup from './AboutPopup.svelte';
	import MenubarSeparator from './MenubarSeparator.svelte';
	import InputField from '$lib/components/InputField.svelte';
	import BigButton from '$lib/components/BigButton.svelte';
	import Popup from '$lib/components/Popup.svelte';
	import { capitalize } from '$lib/util';
	import hotkeys from 'hotkeys-js';
	import { onMount } from 'svelte';
	import { writeText } from '@tauri-apps/plugin-clipboard-manager';

	let importR2Open = false;
	let newProfileOpen = false;
	let exportCodePopup: ExportCodePopup;

	let importProfileOpen = false;
	let importProfileData: ImportData | null = null;

	let profileOperation: 'rename' | 'duplicate' = 'rename';
	let profileOperationName = '';
	let profileOperationOpen = false;
	let profileOperationInProgress = false;

	let aboutOpen = false;

	const appWindow = getCurrentWindow();

	async function importLocalMod() {
		let path = await open({
			title: 'Select the mod file to import',
			filters: [{ name: 'Dll or zip', extensions: ['dll', 'zip'] }]
		});

		if (path === null) return;
		await invokeCommand('import_local_mod', { path });
		await refreshProfiles();
	}

	async function importFile() {
		let path = await open({
			title: 'Select the file to import',
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
			title: 'Select the directory to export the profile to'
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

		await invokeCommand<number>('remove_disabled_mods');
		await refreshProfiles();
	}

	async function zoom(value: { delta: number } | { factor: number }) {
		await invokeCommand('zoom_window', { value });
	}

	async function copyLaunchArgs() {
		let str = await invokeCommand<string>('get_launch_args');
		writeText(str);
	}

	async function clearModCache() {
		let result = await confirm(
			"Are you sure you want to delete all cached mods? This could potentially double the disk space used by installed mods. Only proceed if you know what you're doing!",
			{
				title: 'Clear mod cache'
			}
		);

		if (!result) return;

		await invokeCommand('clear_download_cache', { soft: false });
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

<header data-tauri-drag-region class="flex h-8 flex-shrink-0 bg-gray-800">
	<Menubar.Root class="flex items-center py-1">
		<img src="favicon.png" alt="Gale logo" class="ml-4 mr-2 h-5 w-5 opacity-50" />
		<Menubar.Menu>
			<MenubarTrigger>File</MenubarTrigger>
			<Menubar.Content
				class="mt-0.5 flex flex-col gap-0.5 rounded-lg border border-gray-600 bg-gray-800 py-1 shadow-xl"
			>
				<MenubarItem
					on:click={() => invokeCommand('open_profile_dir')}
					text="Open profile directory"
				/>
				<MenubarItem on:click={() => invokeCommand('open_bepinex_log')} text="Open BepInEx log" />
				<MenubarItem on:click={() => invokeCommand('open_gale_log')} text="Open Gale log" />
				<MenubarSeparator />
				<MenubarItem on:click={clearModCache} text="Clear mod cache" />
				<MenubarItem
					on:click={() => invokeCommand('clear_download_cache', { soft: true })}
					text="Clear unused mod cache"
				/>
				<MenubarItem on:click={() => invokeCommand('trigger_mod_fetching')} text="Fetch mods" />
			</Menubar.Content>
		</Menubar.Menu>
		<Menubar.Menu>
			<MenubarTrigger>Profile</MenubarTrigger>
			<Menubar.Content
				class="mt-0.5 flex flex-col gap-0.5 rounded-lg border border-gray-600 bg-gray-800 py-1 shadow-xl"
			>
				<MenubarItem
					on:click={() => (newProfileOpen = true)}
					text="Create new profile"
					key="Ctrl N"
				/>
				<MenubarItem
					on:click={() => openProfileOperation('rename')}
					text="Rename active profile"
					key="F2"
				/>
				<MenubarItem
					on:click={() => openProfileOperation('duplicate')}
					text="Duplicate active profile"
					key="Ctrl D"
				/>
				<MenubarSeparator />
				<MenubarItem
					on:click={() => invokeCommand('copy_dependency_strings')}
					text="Copy mod list"
				/>
				<MenubarItem on:click={() => invokeCommand('copy_debug_info')} text="Copy debug info" />
				<MenubarItem on:click={copyLaunchArgs} text="Copy launch arguments" />
				<MenubarSeparator />
				<MenubarItem on:click={() => setAllModsState(true)} text="Enable all mods" />
				<MenubarItem on:click={() => setAllModsState(false)} text="Disable all mods" />
				<MenubarItem on:click={uninstallDisabledMods} text="Uninstall disabled mods" />
			</Menubar.Content>
		</Menubar.Menu>
		<Menubar.Menu>
			<MenubarTrigger>Import</MenubarTrigger>
			<Menubar.Content
				class="mt-0.5 flex flex-col gap-0.5 rounded-lg border border-gray-600 bg-gray-800 py-1 shadow-xl"
			>
				<MenubarItem on:click={() => (importProfileOpen = true)} text="...profile from code" />
				<MenubarItem on:click={importFile} text="...profile from file" />
				<MenubarItem on:click={importLocalMod} text="...local mod" />
				<MenubarItem on:click={() => (importR2Open = true)} text="...profiles from r2modman" />
			</Menubar.Content>
		</Menubar.Menu>
		<Menubar.Menu>
			<MenubarTrigger>Export</MenubarTrigger>
			<Menubar.Content
				class="mt-0.5 flex flex-col gap-0.5 rounded-lg border border-gray-600 bg-gray-800 py-1 shadow-xl"
			>
				<MenubarItem on:click={() => exportCodePopup.open()} text="...profile as code" />
				<MenubarItem on:click={exportFile} text="...profile as file" />
			</Menubar.Content>
		</Menubar.Menu>
		<Menubar.Menu>
			<MenubarTrigger>Window</MenubarTrigger>
			<Menubar.Content
				class="mt-0.5 flex flex-col gap-0.5 rounded-lg border border-gray-600 bg-gray-800 py-1 shadow-xl"
			>
				<MenubarItem on:click={appWindow.minimize} text="Minimize" />
				<MenubarItem on:click={appWindow.toggleMaximize} text="Maximize" />
				<MenubarItem on:click={appWindow.close} text="Close" />
				<MenubarSeparator />
				<MenubarItem
					on:click={() => invokeCommand('zoom_window', { value: { delta: 0.25 } })}
					text="Zoom in"
					key="Ctrl +"
				/>
				<MenubarItem
					on:click={() => invokeCommand('zoom_window', { value: { delta: -0.25 } })}
					text="Zoom out"
					key="Ctrl -"
				/>
				<MenubarItem
					on:click={() => invokeCommand('zoom_window', { value: { factor: 1 } })}
					text="Reset zoom"
					key="Ctrl 0"
				/>
			</Menubar.Content>
		</Menubar.Menu>
		<Menubar.Menu>
			<MenubarTrigger>Help</MenubarTrigger>
			<Menubar.Content
				class="mt-0.5 flex flex-col gap-0.5 rounded-lg border border-gray-600 bg-gray-800 py-1 shadow-xl"
			>
				<MenubarItem
					on:click={() => shellOpen('https://github.com/Kesomannen/ModManager/issues/')}
					text="Report a bug"
				/>
				<MenubarItem
					on:click={() => shellOpen('https://discord.gg/lcmod')}
					text="Join LC modding server"
				/>
				<MenubarItem
					on:click={() =>
						shellOpen('https://discord.com/channels/1168655651455639582/1246088342458863618')}
					text="Open discord thread"
				/>
				<MenubarItem on:click={() => (aboutOpen = true)} text="About Gale" />
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
</header>

<Popup
	title="{capitalize(profileOperation)} profile"
	canClose={!profileOperationInProgress}
	bind:open={profileOperationOpen}
>
	<p class="mb-1 text-slate-300">
		{profileOperation == 'duplicate'
			? 'Enter a name for the duplicated profile:'
			: 'Enter a new name for the profile:'}
	</p>
	<InputField
		bind:value={profileOperationName}
		placeholder="Enter name..."
		size="lg"
		class="w-full"
		on:submit={doProfileOperation}
	/>
	{#if profileOperation == 'duplicate'}
		<p class="mt-2 text-sm text-slate-400">
			This process might take up to a minute depending on the size of the profile, please be
			patient.
		</p>
	{/if}
	<div class="ml-auto mt-2 flex justify-end gap-2">
		{#if !profileOperationInProgress}
			<BigButton color="gray" on:click={() => (profileOperationOpen = false)}>Cancel</BigButton>
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
				{capitalize(profileOperation)}
			{/if}
		</BigButton>
	</div>
</Popup>

<NewProfilePopup bind:open={newProfileOpen} />
<ImportProfilePopup bind:open={importProfileOpen} bind:data={importProfileData} />
<ExportCodePopup bind:this={exportCodePopup} />
<ImportR2Popup bind:open={importR2Open} />
<AboutPopup bind:open={aboutOpen} />
