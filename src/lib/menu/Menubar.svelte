<script lang="ts">
	import { run } from 'svelte/legacy';

	import { onMount } from 'svelte';
	import Icon from '@iconify/svelte';
	import { Menubar } from 'bits-ui';

	import MenubarItem from '$lib/menu/MenubarItem.svelte';

	import InputField from '$lib/components/InputField.svelte';
	import Button from '$lib/components/Button.svelte';
	import Popup from '$lib/components/Popup.svelte';

	import ImportR2Popup from '$lib/import/ImportR2Popup.svelte';
	import ExportCodePopup from '$lib/import/ExportCodePopup.svelte';
	import ImportProfilePopup from '$lib/import/ImportProfilePopup.svelte';

	import AboutPopup from './AboutPopup.svelte';
	import MenubarMenu from './MenubarMenu.svelte';
	import NewProfilePopup from './NewProfilePopup.svelte';
	import MenubarSeparator from './MenubarSeparator.svelte';

	import { capitalize, fileToBase64, shortenFileSize } from '$lib/util';
	import { activeProfile, refreshProfiles } from '$lib/stores';
	import { invokeCommand } from '$lib/invoke';
	import type { ImportData } from '$lib/types';
	import { useNativeMenu } from '$lib/theme';

	import { confirm, open } from '@tauri-apps/plugin-dialog';
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import { open as shellOpen } from '@tauri-apps/plugin-shell';
	import { writeText } from '@tauri-apps/plugin-clipboard-manager';
	import { pushInfoToast } from '$lib/toast';
	import { Menu, MenuItem, PredefinedMenuItem, Submenu } from '@tauri-apps/api/menu';

	let importR2Open = $state(false);
	let newProfileOpen = $state(false);

	let exportCodePopup: ExportCodePopup;
	let importProfilePopup: ImportProfilePopup;

	let profileOperation: 'rename' | 'duplicate' = $state('rename');
	let profileOperationName = $state('');
	let profileOperationOpen = $state(false);
	let profileOperationInProgress = $state(false);

	let aboutOpen = $state(false);

	let menu: Menu | null = $state(null);

	const submenus = [
		{
			text: 'File',
			items: [
				{
					text: 'Open profile folder',
					onclick: () => invokeCommand('open_profile_dir')
				},
				{
					text: 'Open game folder',
					onclick: () => invokeCommand('open_game_dir')
				},
				'',
				{
					text: 'Open game log',
					onclick: () => invokeCommand('open_game_log')
				},
				{
					text: 'Open Gale log',
					onclick: () => invokeCommand('open_gale_log')
				},
				'',
				{
					text: 'Clear mod cache',
					onclick: () => clearModCache(false)
				},
				{
					text: 'Clear unused mod cache',
					onclick: () => clearModCache(true)
				},
				{
					text: 'Fetch mods',
					onclick: () => invokeCommand('trigger_mod_fetch')
				}
			]
		},
		{
			text: 'Profile',
			items: [
				{
					text: 'Create new profile',
					accelerator: 'Ctrl+N',
					onclick: () => (newProfileOpen = true)
				},
				{
					text: 'Rename profile',
					accelerator: 'F2',
					onclick: () => openProfileOperation('rename')
				},
				{
					text: 'Duplicate profile',
					accelerator: 'Ctrl+D',
					onclick: () => openProfileOperation('duplicate')
				},
				'',
				{
					text: 'Copy mod list',
					onclick: copyModList
				},
				{
					text: 'Copy debug info',
					onclick: copyDebugInfo
				},
				{
					text: 'Copy launch arguments',
					onclick: copyLaunchArgs
				},
				'',
				{
					text: 'Enable all mods',
					onclick: () => setAllModsState(true)
				},
				{
					text: 'Disable all mods',
					onclick: () => setAllModsState(false)
				},
				{
					text: 'Uninstall disabled mods',
					onclick: uninstallDisabledMods
				},
				'',
				{
					text: 'Create desktop shortcut',
					onclick: createDesktopShotcut
				}
			]
		},
		{
			text: 'Import',
			items: [
				{
					text: '...profile from code',
					onclick: () => importProfilePopup.openForCode()
				},
				{
					text: '...profile from file',
					onclick: browseImportFile
				},
				{
					text: '...local mod',
					onclick: importLocalMod
				},
				{
					text: '...profiles from r2modman',
					onclick: () => (importR2Open = true)
				}
			]
		},
		{
			text: 'Export',
			items: [
				{
					text: '...profile as code',
					onclick: () => exportCodePopup.open()
				},
				{
					text: '...profile as file',
					onclick: exportFile
				}
			]
		},
		{
			text: 'Window',
			items: [
				{
					text: 'Zoom in',
					accelerator: 'Ctrl++',
					onclick: () => invokeCommand('zoom_window', { value: { delta: 0.25 } })
				},
				{
					text: 'Zoom out',
					accelerator: 'Ctrl+-',
					onclick: () => invokeCommand('zoom_window', { value: { delta: -0.25 } })
				},
				{
					text: 'Reset zoom',
					accelerator: 'Ctrl+0',
					onclick: () => invokeCommand('zoom_window', { value: { factor: 1 } })
				}
			]
		},
		{
			text: 'Help',
			items: [
				{
					text: 'Report a bug',
					onclick: () => shellOpen('https://github.com/Kesomannen/ModManager/issues/')
				},
				{
					text: 'Join discord server',
					onclick: () => shellOpen('https://discord.gg/sfuWXRfeTt')
				},
				{
					text: 'About Gale',
					onclick: () => (aboutOpen = true)
				}
			]
		}
	];

	const appWindow = getCurrentWindow();

	async function importLocalMod() {
		let path = await open({
			title: 'Select the mod file to import',
			filters: [{ name: 'Dll or zip', extensions: ['dll', 'zip'] }]
		});

		if (path === null) return;
		await invokeCommand('import_local_mod', { path });
		await refreshProfiles();

		pushInfoToast({
			message: 'Imported local mod into profile.'
		});
	}

	async function browseImportFile() {
		let path = await open({
			title: 'Select the file to import',
			filters: [{ name: 'Profile file', extensions: ['r2z'] }]
		});

		if (path === null) return;
		let data = await invokeCommand<ImportData>('read_profile_file', { path });
		importProfilePopup.openFor({ type: 'normal', ...data });
	}

	async function exportFile() {
		let dir = await open({
			directory: true,
			title: 'Select the folder to export the profile to'
		});

		if (dir === null) return;
		invokeCommand('export_file', { dir });
	}

	async function setAllModsState(enable: boolean) {
		let count = await invokeCommand<number>('set_all_mods_state', { enable });

		pushInfoToast({
			message: `${enable ? 'Enabled' : 'Disabled'} ${count} mods.`
		});

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
				pushInfoToast({
					message: `Renamed profile to ${profileOperationName}.`
				});
			} else if (profileOperation == 'duplicate') {
				await invokeCommand('duplicate_profile', { name: profileOperationName });
				pushInfoToast({
					message: `Duplicated profile to ${profileOperationName}.`
				});
			}
		} catch (e) {
			profileOperationInProgress = false;
			throw e;
		}

		await refreshProfiles();
		profileOperationInProgress = false;
		profileOperationOpen = false;
	}

	async function createDesktopShotcut() {
		await invokeCommand('create_desktop_shortcut');

		pushInfoToast({
			message: `Created desktop shortcut for ${$activeProfile?.name}.`
		});
	}

	async function uninstallDisabledMods() {
		let confirmed = await confirm('Are you sure you want to uninstall all disabled mods?');
		if (!confirmed) return;

		let count = await invokeCommand<number>('remove_disabled_mods');

		pushInfoToast({
			message: `Uninstalled ${count} disabled mods.`
		});

		await refreshProfiles();
	}

	async function zoom(value: { delta: number } | { factor: number }) {
		await invokeCommand('zoom_window', { value });
	}

	async function copyLaunchArgs() {
		let str = await invokeCommand<string>('get_launch_args');
		await writeText(str);

		pushInfoToast({
			message: `Copied launch arguments to clipboard.`
		});
	}

	async function clearModCache(soft: boolean) {
		if (!soft) {
			let result = await confirm(
				"Are you sure you want to delete all cached mods? This could potentially double the disk space used by installed mods. Only proceed if you know what you're doing!"
			);

			if (!result) return;
		}

		let size = await invokeCommand<number>('clear_download_cache', { soft });
		pushInfoToast({
			message: `Deleted${soft ? ' unused' : ''} mod cache (cleared ${shortenFileSize(size)}).`
		});
	}

	async function copyModList() {
		await invokeCommand('copy_dependency_strings');
		pushInfoToast({
			message: 'Copied mod list to clipboard.'
		});
	}

	async function copyDebugInfo() {
		await invokeCommand('copy_debug_info');
		pushInfoToast({
			message: 'Copied debug info to clipboard.'
		});
	}

	async function handleFileDrop(evt: DragEvent) {
		evt.preventDefault();
		if (evt.dataTransfer === null) return;

		let file: File | null;
		if (evt.dataTransfer.items) {
			let files = [...evt.dataTransfer.items].filter((item) => item.kind == 'file');
			if (files.length === 0) return;
			file = files[0].getAsFile();
		} else {
			file = [...evt.dataTransfer.items][0];
		}

		if (file === null) return;
		let base64 = await fileToBase64(file);

		if (file.name.endsWith('.r2z')) {
			let data = await invokeCommand<ImportData>('read_profile_base64', { base64 });
			importProfilePopup.openFor({ type: 'normal', ...data });
		} else if (file.name.endsWith('.zip')) {
			await invokeCommand('import_local_mod_base64', { base64 });
			await refreshProfiles();

			pushInfoToast({
				message: 'Imported local mod into profile.'
			});
		}
	}

	run(() => {
		if (menu != null) {
			appWindow.setDecorations($useNativeMenu);

			if ($useNativeMenu) {
				menu.setAsAppMenu();
			} else {
				Menu.new().then((menu) => menu.setAsAppMenu());
			}

			localStorage.setItem('useNativeMenu', $useNativeMenu.toString());
		}
	});

	const hotkeys: { [key: string]: () => void } = {
		'+': () => zoom({ delta: 0.25 }),
		'-': () => zoom({ delta: -0.25 }),
		'0': () => zoom({ factor: 1 }),
		n: () => (newProfileOpen = true),
		d: () => openProfileOperation('duplicate')
	};

	onMount(async () => {
		document.onkeydown = ({ key, ctrlKey }) => {
			if ($useNativeMenu) return;

			if (key === 'F2') {
				openProfileOperation('rename');
				return;
			}

			if (!ctrlKey) return;

			let hotkey = hotkeys[key];
			if (hotkey !== undefined) hotkey();
		};

		let separator = await PredefinedMenuItem.new({
			item: 'Separator'
		});

		let nativeMenus = await Promise.all(
			submenus.map(
				async (menu) =>
					await Submenu.new({
						text: menu.text,
						items: await Promise.all(
							menu.items.map(async (item) =>
								typeof item === 'string'
									? separator
									: await MenuItem.new({
											action: item.onclick,
											...item
										})
							)
						)
					})
			)
		);

		menu = await Menu.new({
			items: nativeMenus
		});
	});
</script>

<svelte:body
	ondragenter={(evt) => evt.preventDefault()}
	ondragover={(evt) => evt.preventDefault()}
	ondrop={handleFileDrop}
/>

<header
	data-tauri-drag-region
	class="bg-primary-800 flex h-8 shrink-0"
	class:hidden={$useNativeMenu && false}
>
	<Menubar.Root class="flex items-center py-1">
		<img src="favicon.png" alt="Gale logo" class="mr-2 ml-4 h-5 w-5 opacity-50" />
		{#each submenus as submenu}
			<MenubarMenu label={submenu.text}>
				{#each submenu.items as item}
					{#if typeof item === 'string'}
						<MenubarSeparator />
					{:else}
						<MenubarItem onclick={item.onclick} text={item.text} />
					{/if}
				{/each}
			</MenubarMenu>
		{/each}
	</Menubar.Root>

	<button class="group hover:bg-primary-700 ml-auto px-3 py-1.5" onclick={appWindow.minimize}>
		<Icon icon="mdi:minimize" class="text-primary-500 group-hover:text-white" />
	</button>
	<button class="group hover:bg-primary-700 px-3 py-1.5" onclick={appWindow.toggleMaximize}>
		<Icon icon="mdi:maximize" class="text-primary-500 group-hover:text-white" />
	</button>
	<button class="group px-3 py-1.5 hover:bg-red-700" onclick={appWindow.close}>
		<Icon icon="mdi:close" class="text-primary-500 group-hover:text-white" />
	</button>
</header>

<Popup
	title="{capitalize(profileOperation)} profile"
	canClose={!profileOperationInProgress}
	bind:open={profileOperationOpen}
>
	<p class="text-primary-300 mb-1">
		{profileOperation == 'duplicate'
			? 'Enter a name for the duplicated profile:'
			: 'Enter a new name for the profile:'}
	</p>
	<InputField
		bind:value={profileOperationName}
		placeholder="Enter name..."
		size="lg"
		class="w-full"
		onsubmit={doProfileOperation}
	/>
	<div class="mt-2 ml-auto flex justify-end gap-2">
		{#if !profileOperationInProgress}
			<Button color="primary" onclick={() => (profileOperationOpen = false)}>Cancel</Button>
		{/if}
		<Button
			color="accent"
			fontWeight="medium"
			disabled={profileOperationInProgress}
			onclick={doProfileOperation}
		>
			{#if profileOperationInProgress}
				<Icon icon="mdi:loading" class="my-1 animate-spin text-lg" />
			{:else}
				{capitalize(profileOperation)}
			{/if}
		</Button>
	</div>
</Popup>

<AboutPopup bind:open={aboutOpen} />
<ImportR2Popup bind:open={importR2Open} />
<NewProfilePopup bind:open={newProfileOpen} />
<ExportCodePopup bind:this={exportCodePopup} />
<ImportProfilePopup bind:this={importProfilePopup} />
