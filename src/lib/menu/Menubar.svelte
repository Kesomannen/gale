<script lang="ts">
	import Icon from '@iconify/svelte';
	import { Button, Menubar } from 'bits-ui';

	import MenubarTrigger from '$lib/menu/MenubarTrigger.svelte';
	import MenubarItem from '$lib/menu/MenubarItem.svelte';
	import PreferencesPopup from '$lib/prefs/PrefsPopup.svelte';

	import ExportPackPopup from '$lib/import/ExportPackPopup.svelte';
	import { invokeCommand } from '$lib/invoke';

	import { open } from '@tauri-apps/api/shell';
	import { appWindow } from '@tauri-apps/api/window';
	import ImportProfilePopup from '$lib/import/ImportProfilePopup.svelte';
	import ExportCodePopup from '$lib/import/ExportCodePopup.svelte';
	import { dialog } from '@tauri-apps/api';
	import type { ImportData } from '$lib/models';
	import { fly } from 'svelte/transition';

	let preferencesOpen = false;
	let exportPackOpen = false;
	let exportCodePopup: ExportCodePopup;

	let importProfileOpen = false;
	let importProfileData: ImportData | undefined = undefined;

	async function importLocal() {
		let path = await dialog.open({
			directory: true,
			title: 'Select the root of the mod to import'
		});

		if (!path) return;
		invokeCommand('import_local_mod', { path });
	}

	async function importFile() {
		let path = await dialog.open({
			title: 'Select the file to import',
			filters: [{ name: 'Profile file', extensions: ['r2z'] }]
		});

		if (!path) return;
		let data = await invokeCommand<ImportData>('import_file', { path });
		importProfileData = data;
		importProfileOpen = true;
	}

	async function exportFile() {
		let dir = await dialog.open({
			directory: true,
			title: 'Select the directory to export the profile to'
		});

		if (!dir) return;
		invokeCommand('export_file', { dir });
	}
</script>

<div data-tauri-drag-region class="h-8 flex bg-gray-800 flex-shrink-0">
	<Menubar.Root class="py-1 flex items-center">
		<img src="favicon.png" alt="Gale logo" class="ml-4 mr-2 h-5 w-5 opacity-50" />
		<Menubar.Menu>
			<MenubarTrigger>File</MenubarTrigger>
			<Menubar.Content
				class="bg-gray-800 shadow-xl flex-col flex gap-0.5 py-1 mt-0.5 rounded-lg border border-gray-600"
			>
				<MenubarItem onClick={() => invokeCommand('reveal_profile_dir')}
					>Show profile in explorer</MenubarItem
				>
				<MenubarItem onClick={() => invokeCommand('open_logs')}>Open game logs</MenubarItem>
				<Menubar.Separator class="w-full h-[1px] bg-gray-600 my-0.5" />
				<MenubarItem onClick={() => invokeCommand('clear_download_cache', { soft: true })}
					>Clear unused mod cache</MenubarItem
				>
				<MenubarItem onClick={() => invokeCommand('clear_download_cache', { soft: false })}
					>Clear all cached mods</MenubarItem
				>
				<Menubar.Separator class="w-full h-[1px] bg-gray-600 my-0.5" />
				<MenubarItem onClick={() => (preferencesOpen = true)}>Settings</MenubarItem>
			</Menubar.Content>
		</Menubar.Menu>
		<Menubar.Menu>
			<MenubarTrigger>Import</MenubarTrigger>
			<Menubar.Content
				class="bg-gray-800 shadow-xl flex-col flex gap-0.5 py-1 mt-0.5 rounded-lg border border-gray-600"
			>
				<MenubarItem onClick={() => (importProfileOpen = true)}>...profile from code</MenubarItem>
				<MenubarItem onClick={importFile}>...profile from file</MenubarItem>
				<MenubarItem onClick={importLocal}>...local mod</MenubarItem>
			</Menubar.Content>
		</Menubar.Menu>
		<Menubar.Menu>
			<MenubarTrigger>Export</MenubarTrigger>
			<Menubar.Content
				class="bg-gray-800 shadow-xl flex-col flex gap-0.5 py-1 mt-0.5 rounded-lg border border-gray-600"
			>
				<MenubarItem onClick={() => exportCodePopup.open()}>...profile as code</MenubarItem>
				<MenubarItem onClick={exportFile}>...profile as file</MenubarItem>
				<MenubarItem onClick={() => (exportPackOpen = true)}>...profile as modpack</MenubarItem>
			</Menubar.Content>
		</Menubar.Menu>
		<Menubar.Menu>
			<MenubarTrigger>Help</MenubarTrigger>
			<Menubar.Content
				class="bg-gray-800 shadow-xl flex-col flex gap-0.5 py-1 mt-0.5 rounded-lg border border-gray-600"
			>
				<MenubarItem onClick={() => open('https://github.com/Kesomannen/ModManager/issues/')}
					>Report a bug</MenubarItem
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

<PreferencesPopup bind:open={preferencesOpen} />
<ExportPackPopup bind:isOpen={exportPackOpen} />
<ImportProfilePopup bind:open={importProfileOpen} bind:data={importProfileData} />
<ExportCodePopup bind:this={exportCodePopup} />
