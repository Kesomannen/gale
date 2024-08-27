<script lang="ts">
	import Icon from '@iconify/svelte';
	import { Button, Dialog, Menubar } from 'bits-ui';

	import MenubarTrigger from '$lib/menu/MenubarTrigger.svelte';
	import MenubarItem from '$lib/menu/MenubarItem.svelte';

	import { invokeCommand } from '$lib/invoke';

	import { open as shellOpen } from '@tauri-apps/plugin-shell';
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import ImportProfilePopup from '$lib/import/ImportProfilePopup.svelte';
	import ExportCodePopup from '$lib/import/ExportCodePopup.svelte';
	import { writeText } from '@tauri-apps/plugin-clipboard-manager';
	import { open } from '@tauri-apps/plugin-dialog';
	import type { ImportData } from '$lib/models';
	import ImportR2Popup from '$lib/import/ImportR2Popup.svelte';
	import { activeProfile, refreshProfiles } from '$lib/stores';
	import NewProfilePopup from './NewProfilePopup.svelte';
	import ConfirmPopup from '$lib/components/ConfirmPopup.svelte';
	import InputField from '$lib/components/InputField.svelte';
	import BigButton from '$lib/components/BigButton.svelte';
	import { capitalize } from '$lib/util';
	import Popup from '$lib/components/Popup.svelte';
	import { refreshUpdate } from './Updater.svelte';

	let importR2Open = false;
	let newProfileOpen = false;
	let exportPackOpen = false;
	let exportCodePopup: ExportCodePopup;

	let importProfileOpen = false;
	let importProfileData: ImportData | null = null;

	let profileOperation: 'rename' | 'duplicate' = 'rename';
	let profileOperationName = '';
	let profileOperationOpen = false;
	let profileOperationInProgress = false;

	const appWindow = getCurrentWindow();

	async function importModDir() {
		let path = await open({
			directory: true,
			title: 'Select the directory containing the mod package'
		});

		if (path === null) return;
		invokeCommand('import_local_mod', { path });
	}

	async function importModFile() {
		let response = await open({
			title: 'Select the mod file to import',
			filters: [{ name: 'Dll or zip', extensions: ['dll', 'zip'] }]
		});

		if (response === null) return;
		invokeCommand('import_local_mod', { path: response.path });
	}

	async function importFile() {
		let response = await open({
			title: 'Select the file to import',
			filters: [{ name: 'Profile file', extensions: ['r2z'] }]
		});

		if (!response) return;
		let data = await invokeCommand<ImportData>('import_file', { path: response.path });
		importProfileData = data;
		importProfileOpen = true;
	}

	async function exportFile() {
		let dir = await open({
			directory: true,
			title: 'Select the directory to export the profile to'
		});

		if (!dir) return;
		invokeCommand('export_file', { dir });
	}

	async function setAllModsState(enable: boolean) {
		await invokeCommand('set_all_mods_state', { enable });
		activeProfile.update((p) => p);
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
</script>

<div data-tauri-drag-region class="h-8 flex bg-gray-800 flex-shrink-0">
	<!-- Fix for top border not being draggable -->
	<div data-tauri-drag-region class="fixed top-0 left-0 w-full h-[1px] z-50" />

	<Menubar.Root class="py-1 flex items-center">
		<img src="favicon.png" alt="Gale logo" class="ml-4 mr-2 h-5 w-5 opacity-50" />
		<Menubar.Menu>
			<MenubarTrigger>File</MenubarTrigger>
			<Menubar.Content
				class="bg-gray-800 shadow-xl flex-col flex gap-0.5 py-1 mt-0.5 rounded-lg border border-gray-600"
			>
				<MenubarItem on:click={() => invokeCommand('open_profile_dir')}
					>Open profile directory</MenubarItem
				>
				<MenubarItem on:click={() => invokeCommand('open_bepinex_log')}>Open game logs</MenubarItem>
				<MenubarItem on:click={() => invokeCommand('open_gale_log')}>Open gale logs</MenubarItem>
				<Menubar.Separator class="w-full h-[1px] bg-gray-600 my-0.5" />
				<MenubarItem on:click={() => invokeCommand('clear_download_cache', { soft: true })}
					>Clear unused mod cache</MenubarItem
				>
				<MenubarItem on:click={() => invokeCommand('clear_download_cache', { soft: false })}
					>Clear all cached mods</MenubarItem
				>
				<Menubar.Separator class="w-full h-[1px] bg-gray-600 my-0.5" />
				<MenubarItem on:click={() => invokeCommand('trigger_mod_fetching')}>Fetch mods</MenubarItem>
			</Menubar.Content>
		</Menubar.Menu>
		<Menubar.Menu>
			<MenubarTrigger>Profile</MenubarTrigger>
			<Menubar.Content
				class="bg-gray-800 shadow-xl flex-col flex gap-0.5 py-1 mt-0.5 rounded-lg border border-gray-600"
			>
				<MenubarItem on:click={() => (newProfileOpen = true)}>Create new profile</MenubarItem>
				<MenubarItem on:click={() => openProfileOperation('rename')}
					>Rename active profile</MenubarItem
				>
				<MenubarItem on:click={() => openProfileOperation('duplicate')}
					>Duplicate active profile</MenubarItem
				>
				<Menubar.Separator class="w-full h-[1px] bg-gray-600 my-0.5" />
				<MenubarItem on:click={() => invokeCommand('copy_dependency_strings')}
					>Copy mod list</MenubarItem
				>
				<MenubarItem on:click={() => invokeCommand('copy_debug_info')}>Copy debug info</MenubarItem>
				<MenubarItem on:click={() => invokeCommand('copy_launch_args')}
					>Copy launch arguments</MenubarItem
				>
				<Menubar.Separator class="w-full h-[1px] bg-gray-600 my-0.5" />
				<MenubarItem on:click={() => setAllModsState(true)}>Enable all mods</MenubarItem>
				<MenubarItem on:click={() => setAllModsState(false)}>Disable all mods</MenubarItem>
			</Menubar.Content>
		</Menubar.Menu>
		<Menubar.Menu>
			<MenubarTrigger>Import</MenubarTrigger>
			<Menubar.Content
				class="bg-gray-800 shadow-xl flex-col flex gap-0.5 py-1 mt-0.5 rounded-lg border border-gray-600"
			>
				<MenubarItem on:click={() => (importProfileOpen = true)}>...profile from code</MenubarItem>
				<MenubarItem on:click={importFile}>...profile from file</MenubarItem>
				<MenubarItem on:click={importModFile}>...local mod from file</MenubarItem>
				<MenubarItem on:click={importModDir}>...local mod from directory</MenubarItem>
				<MenubarItem on:click={() => (importR2Open = true)}>...profiles from r2modman</MenubarItem>
			</Menubar.Content>
		</Menubar.Menu>
		<Menubar.Menu>
			<MenubarTrigger>Export</MenubarTrigger>
			<Menubar.Content
				class="bg-gray-800 shadow-xl flex-col flex gap-0.5 py-1 mt-0.5 rounded-lg border border-gray-600"
			>
				<MenubarItem on:click={() => exportCodePopup.open()}>...profile as code</MenubarItem>
				<MenubarItem on:click={exportFile}>...profile as file</MenubarItem>
			</Menubar.Content>
		</Menubar.Menu>
		<Menubar.Menu>
			<MenubarTrigger>Help</MenubarTrigger>
			<Menubar.Content
				class="bg-gray-800 shadow-xl flex-col flex gap-0.5 py-1 mt-0.5 rounded-lg border border-gray-600"
			>
				<MenubarItem on:click={refreshUpdate}>Check for app updates</MenubarItem>
				<MenubarItem on:click={() => shellOpen('https://github.com/Kesomannen/ModManager/issues/')}
					>Report a bug</MenubarItem
				>
				<MenubarItem
					on:click={() => {
						shellOpen('https://discord.gg/lcmod');
					}}>Join LC modding server</MenubarItem
				>
				<MenubarItem
					on:click={() => {
						shellOpen('https://discord.com/channels/1168655651455639582/1246088342458863618');
					}}>Open discord thread</MenubarItem
				>
			</Menubar.Content>
		</Menubar.Menu>
	</Menubar.Root>

	<Button.Root class="px-3 py-1.5 hover:bg-gray-700 ml-auto group" on:click={appWindow.minimize}>
		<Icon icon="mdi:minimize" class="text-gray-500 group-hover:text-white" />
	</Button.Root>
	<Button.Root class="px-3 py-1.5 hover:bg-gray-700 group" on:click={appWindow.toggleMaximize}>
		<Icon icon="mdi:maximize" class="text-gray-500 group-hover:text-white" />
	</Button.Root>
	<Button.Root class="px-3 py-1.5 hover:bg-red-700 group" on:click={appWindow.close}>
		<Icon icon="mdi:close" class="text-gray-500 group-hover:text-white" />
	</Button.Root>
</div>

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
		<p class="mt-3 text-slate-400 text-sm">
			This process might take up to a minute depending on the size of the profile, please be
			patient.
		</p>
	{/if}
	<div class="flex ml-auto justify-end gap-2 mt-2">
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
				<Icon icon="mdi:loading" class="animate-spin text-lg my-1" />
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
