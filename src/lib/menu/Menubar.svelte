<script lang="ts">
	import Icon from '@iconify/svelte';
	import { Button, Menubar } from 'bits-ui';

	import MenubarTrigger from '$lib/menu/MenubarTrigger.svelte';
	import MenubarItem from '$lib/menu/MenubarItem.svelte';
	import NewProfilePopup from '$lib/menu/NewProfilePopup.svelte';
	import PreferencesPopup from '$lib/preferences/PreferencesPopup.svelte';

	import { open } from '$lib/util';
	import ExportPackPopup from '$lib/import/ExportPackPopup.svelte';
	import { invokeCommand } from '$lib/error';
	import { appWindow } from '@tauri-apps/api/window';

	let newProfileOpen = false;
	let preferencesOpen = false;
	let exportPackOpen = false;
</script>

<div data-tauri-drag-region class="h-9 flex bg-gray-800 flex-shrink-0">
	<Menubar.Root class="pl-4 py-1">
		<Menubar.Menu>
			<MenubarTrigger>File</MenubarTrigger>
			<Menubar.Content
				class="bg-gray-800 shadow-xl flex-col flex gap-0.5 p-2 mt-0.5 rounded-lg border border-gray-600"
			>
				<MenubarItem onClick={() => (newProfileOpen = true)}>New profile</MenubarItem>
				<MenubarItem onClick={() => invokeCommand('reveal_project_dir')}
					>Show profile in explorer</MenubarItem
				>
				<Menubar.Separator class="w-full h-[1px] bg-gray-600 my-2" />
				<MenubarItem onClick={() => invokeCommand('clear_download_cache')}
					>Clear download cache</MenubarItem
				>
				<Menubar.Separator class="w-full h-[1px] bg-gray-600 my-2" />
				<MenubarItem onClick={() => (preferencesOpen = true)}>Preferences</MenubarItem>
				<MenubarItem onClick={window.close}>Quit</MenubarItem>
			</Menubar.Content>
		</Menubar.Menu>
		<Menubar.Menu>
			<MenubarTrigger>Import</MenubarTrigger>
		</Menubar.Menu>
		<Menubar.Menu>
			<MenubarTrigger>Export</MenubarTrigger>
			<Menubar.Content
				class="bg-gray-800 shadow-xl flex-col flex gap-0.5 p-1 mt-0.5 rounded-lg border border-gray-600"
			>
				<MenubarItem onClick={() => (exportPackOpen = true)}>Profile as modpack</MenubarItem>
			</Menubar.Content>
		</Menubar.Menu>
		<Menubar.Menu>
			<MenubarTrigger>Help</MenubarTrigger>
			<Menubar.Content
				class="bg-gray-800 shadow-xl flex-col flex gap-0.5 p-1 mt-0.5 rounded-lg border border-gray-600"
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

<NewProfilePopup bind:open={newProfileOpen} />
<PreferencesPopup bind:open={preferencesOpen} />
<ExportPackPopup bind:isOpen={exportPackOpen} />
